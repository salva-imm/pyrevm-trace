use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use revm::{
    context::{Context, TxEnv},
    context_interface::ContextTr,
    context_interface::result::{ExecutionResult, Output},
    database::CacheDB,
    database_interface::EmptyDB,
    inspector::Inspector,
    interpreter::{interpreter::EthInterpreter, CallInputs, CallOutcome, Interpreter},
    primitives::{Bytes, TxKind, U256, KECCAK_EMPTY},
    state::{AccountInfo, Bytecode},
    InspectCommitEvm, MainBuilder, MainContext,
};
use crate::gas_profiler::GasProfiler;
use crate::tracer::{CallFrame, CallTracer};
use crate::types::{address_to_hex, parse_address, py_int_to_u256};

enum SimInspector {
    Trace(CallTracer),
    Gas(GasProfiler),
}

impl<CTX: ContextTr> Inspector<CTX, EthInterpreter> for SimInspector {
    fn call(&mut self, ctx: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        match self {
            Self::Trace(t) => t.call(ctx, inputs),
            Self::Gas(_) => None,
        }
    }

    fn call_end(&mut self, ctx: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        match self {
            Self::Trace(t) => t.call_end(ctx, inputs, outcome),
            Self::Gas(_) => {}
        }
    }

    fn step(&mut self, interp: &mut Interpreter<EthInterpreter>, ctx: &mut CTX) {
        match self {
            Self::Trace(_) => {}
            Self::Gas(g) => g.step(interp, ctx),
        }
    }

    fn step_end(&mut self, interp: &mut Interpreter<EthInterpreter>, ctx: &mut CTX) {
        match self {
            Self::Trace(_) => {}
            Self::Gas(g) => g.step_end(interp, ctx),
        }
    }
}

#[pyclass]
pub struct EVMSimulator {
    chain_id: u64,
    pub(crate) db: CacheDB<EmptyDB>,
}

#[pymethods]
impl EVMSimulator {
    #[new]
    #[pyo3(signature = (chain_id = 1))]
    pub fn new(chain_id: u64) -> Self {
        EVMSimulator { chain_id, db: CacheDB::default() }
    }

    pub fn set_balance(&mut self, address: &str, balance: u128) -> PyResult<()> {
        let addr = parse_address(address)?;
        let (code_hash, code) = self.db.cache.accounts.get(&addr)
            .and_then(|acc| acc.info.code.clone().map(|c| (acc.info.code_hash, Some(c))))
            .unwrap_or((KECCAK_EMPTY, None));
        self.db.insert_account_info(addr, AccountInfo {
            balance: py_int_to_u256(balance),
            nonce: 0,
            code_hash,
            code,
            ..Default::default()
        });
        Ok(())
    }

    pub fn set_code(&mut self, address: &str, bytecode: Vec<u8>) -> PyResult<()> {
        let addr = parse_address(address)?;
        let code = Bytecode::new_raw(Bytes::from(bytecode));
        let code_hash = code.hash_slow();
        self.db.insert_account_info(addr, AccountInfo {
            balance: U256::ZERO,
            nonce: 0,
            code_hash,
            code: Some(code),
            ..Default::default()
        });
        Ok(())
    }

    pub fn set_storage(&mut self, address: &str, slot: u64, value: u64) -> PyResult<()> {
        let addr = parse_address(address)?;
        self.db
            .insert_account_storage(addr, U256::from(slot), U256::from(value))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;
        Ok(())
    }

    #[pyo3(signature = (caller, to, calldata, value, gas_limit, with_trace = false, with_gas_profile = false))]
    pub fn simulate(
        &mut self,
        py: Python<'_>,
        caller: &str,
        to: &str,
        calldata: Vec<u8>,
        value: u128,
        gas_limit: u64,
        with_trace: bool,
        with_gas_profile: bool,
    ) -> PyResult<PyObject> {
        let caller_addr = parse_address(caller)?;
        let to_addr = parse_address(to)?;

        let tx = TxEnv::builder()
            .caller(caller_addr)
            .kind(TxKind::Call(to_addr))
            .value(U256::from(value))
            .data(Bytes::from(calldata))
            .gas_limit(gas_limit)
            .build()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;

        let db = std::mem::replace(&mut self.db, CacheDB::default());

        let ctx = Context::mainnet()
            .modify_cfg_chained(|cfg| {
                cfg.chain_id = self.chain_id;
                cfg.disable_nonce_check = true;
            })
            .with_db(db);

        // with_trace takes priority; gas_profile only active when trace is off
        let mut inspector = if with_gas_profile && !with_trace {
            SimInspector::Gas(GasProfiler::new())
        } else {
            SimInspector::Trace(CallTracer::new())
        };
        let mut evm = ctx.build_mainnet_with_inspector(&mut inspector);

        let result = evm.inspect_tx_commit(tx)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)));

        self.db = std::mem::replace(
            &mut evm.ctx.journaled_state.database,
            CacheDB::default(),
        );

        let result = result?;
        let dict = self.result_to_py(py, result)?;
        let py_dict = dict.bind(py).downcast::<PyDict>().unwrap();

        match (with_trace, with_gas_profile, inspector) {
            (true, _, SimInspector::Trace(tracer)) => {
                if let Some(frame) = tracer.root {
                    py_dict.set_item("calls", frame_to_py(py, &frame)?)?;
                }
            }
            (false, true, SimInspector::Gas(profiler)) => {
                let steps: Vec<PyObject> = profiler.steps.iter().map(|s| {
                    let d = PyDict::new_bound(py);
                    d.set_item("pc", s.pc).unwrap();
                    d.set_item("op", s.op).unwrap();
                    d.set_item("gas_remaining", s.gas_remaining).unwrap();
                    d.set_item("gas_cost", s.gas_cost).unwrap();
                    d.into()
                }).collect();
                py_dict.set_item("gas_steps", steps)?;
            }
            _ => {}
        }

        Ok(dict)
    }
}

impl EVMSimulator {
    fn result_to_py(&self, py: Python<'_>, result: ExecutionResult) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        match result {
            ExecutionResult::Success { gas, output, logs, .. } => {
                dict.set_item("success", true)?;
                dict.set_item("gas_used", gas.tx_gas_used())?;
                let return_data = match output {
                    Output::Call(bytes) => bytes,
                    Output::Create(bytes, _) => bytes,
                };
                dict.set_item("return_data", PyBytes::new_bound(py, &return_data))?;
                let py_logs: Vec<PyObject> = logs.iter().map(|log| {
                    let d = PyDict::new_bound(py);
                    d.set_item("address", address_to_hex(log.address)).unwrap();
                    let topics: Vec<String> = log.topics().iter()
                        .map(|t| format!("0x{t:x}"))
                        .collect();
                    d.set_item("topics", topics).unwrap();
                    d.set_item("data", PyBytes::new_bound(py, &log.data.data)).unwrap();
                    d.into()
                }).collect();
                dict.set_item("logs", py_logs)?;
            }
            ExecutionResult::Revert { gas, output, .. } => {
                dict.set_item("success", false)?;
                dict.set_item("gas_used", gas.tx_gas_used())?;
                dict.set_item("return_data", PyBytes::new_bound(py, &output))?;
                dict.set_item("logs", Vec::<PyObject>::new())?;
            }
            ExecutionResult::Halt { gas, .. } => {
                dict.set_item("success", false)?;
                dict.set_item("gas_used", gas.tx_gas_used())?;
                dict.set_item("return_data", PyBytes::new_bound(py, b""))?;
                dict.set_item("logs", Vec::<PyObject>::new())?;
            }
        }
        Ok(dict.into())
    }
}

fn frame_to_py(py: Python<'_>, frame: &CallFrame) -> PyResult<PyObject> {
    let d = PyDict::new_bound(py);
    d.set_item("from", address_to_hex(frame.from))?;
    d.set_item("to", address_to_hex(frame.to))?;
    d.set_item("calldata", PyBytes::new_bound(py, &frame.calldata))?;
    d.set_item("value", frame.value.to_string())?;
    d.set_item("gas_limit", frame.gas_limit)?;
    d.set_item("success", frame.success)?;
    d.set_item("output", PyBytes::new_bound(py, &frame.output))?;
    let sub: Vec<PyObject> = frame.calls.iter()
        .map(|c| frame_to_py(py, c))
        .collect::<PyResult<_>>()?;
    d.set_item("calls", sub)?;
    Ok(d.into())
}
