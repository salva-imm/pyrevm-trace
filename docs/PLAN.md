# pyrevm-trace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a pure Python package (`pyrevm-trace`) that exposes EVM simulation, state override, and gas profiling via both a sync and async API — with FastAPI/other framework integration left to downstream consumers.

**Architecture:** Rust crate (`src/`) wraps REVM via PyO3, returning raw dicts. The Python layer wraps those dicts in Pydantic models and exposes a sync `Simulator` class plus an `AsyncSimulator` that offloads CPU work to `asyncio.to_thread`. No web framework dependency.

**Tech Stack:** Rust + REVM 14 + PyO3 0.22 + maturin 1.x (build), Python + Pydantic v2 (models), pytest + pytest-asyncio (tests)

---

## File Structure

```
pyrevm-trace/
├── Cargo.toml                          # Rust cdylib for PyO3
├── pyproject.toml                      # maturin build backend, no web deps
├── src/
│   ├── lib.rs                          # PyO3 module, registers EVMSimulator
│   ├── types.rs                        # Address / U256 parse helpers
│   ├── executor.rs                     # EVMSimulator pyclass + simulate()
│   ├── tracer.rs                       # CallTracer Inspector (call tree)
│   └── gas_profiler.rs                 # GasProfiler Inspector (opcode steps)
├── python/
│   └── pyrevm_trace/
│       ├── __init__.py                 # Public re-exports
│       ├── models.py                   # Pydantic models (SimulationResult, etc.)
│       ├── simulator.py               # Sync Simulator class
│       └── async_simulator.py         # AsyncSimulator (asyncio.to_thread wrapper)
└── tests/
    ├── test_simulation.py             # Sync simulation + state management
    ├── test_trace.py                  # Call trace tests
    ├── test_gas_profile.py            # Opcode gas profiling tests
    └── test_async.py                  # AsyncSimulator tests
```

---

## Task 1: Project Scaffolding (maturin + Cargo)

**Files:**
- Create: `Cargo.toml`
- Modify: `pyproject.toml`
- Create: `src/lib.rs`, `src/types.rs`, `src/executor.rs`, `src/tracer.rs`, `src/gas_profiler.rs`
- Create: `python/pyrevm_trace/__init__.py`
- Delete: `main.py`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "pyrevm-trace"
version = "0.1.0"
edition = "2021"

[lib]
name = "_pyrevm_trace"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }
revm = { version = "14", default-features = false, features = ["std"] }

[profile.release]
lto = true
codegen-units = 1
opt-level = 3
```

- [ ] **Step 2: Overwrite pyproject.toml**

```toml
[project]
name = "pyrevm-trace"
version = "0.1.0"
description = "Python bindings for fast EVM tracing using REVM"
readme = "README.md"
requires-python = ">=3.12"
dependencies = [
    "pydantic>=2.0",
]

[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[tool.maturin]
features = ["pyo3/extension-module"]
python-source = "python"
module-name = "pyrevm_trace._pyrevm_trace"

[project.optional-dependencies]
dev = ["pytest", "pytest-asyncio"]
```

- [ ] **Step 3: Create src/lib.rs**

```rust
use pyo3::prelude::*;

mod executor;
mod gas_profiler;
mod tracer;
mod types;

#[pymodule]
fn _pyrevm_trace(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<executor::EVMSimulator>()?;
    Ok(())
}
```

- [ ] **Step 4: Create placeholder Rust source files**

`src/types.rs`:
```rust
// placeholder
```

`src/tracer.rs`:
```rust
// placeholder
```

`src/gas_profiler.rs`:
```rust
// placeholder
```

`src/executor.rs` (minimal compilable stub):
```rust
use pyo3::prelude::*;

#[pyclass]
pub struct EVMSimulator {}

#[pymethods]
impl EVMSimulator {
    #[new]
    pub fn new() -> Self { EVMSimulator {} }
}
```

- [ ] **Step 5: Create python/pyrevm_trace/__init__.py**

```bash
mkdir -p python/pyrevm_trace
```

```python
from pyrevm_trace._pyrevm_trace import EVMSimulator

__all__ = ["EVMSimulator"]
```

- [ ] **Step 6: Verify build**

```bash
pip install maturin
maturin develop
python -c "from pyrevm_trace import EVMSimulator; print('OK')"
```

Expected: `OK`

- [ ] **Step 7: Remove main.py and commit**

```bash
git rm main.py
git add Cargo.toml pyproject.toml src/ python/ .gitignore
git commit -m "chore: scaffold PyO3+maturin project structure"
```

---

## Task 2: Rust Address and U256 Helpers

**Files:**
- Modify: `src/types.rs`

- [ ] **Step 1: Write src/types.rs with unit tests**

```rust
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use revm::primitives::{Address, U256};
use std::str::FromStr;

pub fn parse_address(addr: &str) -> PyResult<Address> {
    let addr = addr.trim_start_matches("0x");
    Address::from_str(addr)
        .map_err(|e| PyValueError::new_err(format!("Invalid address '{}': {}", addr, e)))
}

pub fn address_to_hex(addr: Address) -> String {
    format!("0x{addr:x}")
}

pub fn py_int_to_u256(val: u128) -> U256 {
    U256::from(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_address() {
        assert!(parse_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").is_ok());
    }

    #[test]
    fn test_parse_without_prefix() {
        assert!(parse_address("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045").is_ok());
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_address("not_an_address").is_err());
    }

    #[test]
    fn test_u256_from_u128() {
        let val: u128 = 10u128.pow(18);
        assert_eq!(py_int_to_u256(val), U256::from(val));
    }
}
```

- [ ] **Step 2: Run Rust tests**

```bash
cargo test types
```

Expected: 4 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/types.rs
git commit -m "feat: add address and U256 parse helpers"
```

---

## Task 3: EVM State Management

**Files:**
- Modify: `src/executor.rs`
- Create: `tests/test_simulation.py`

- [ ] **Step 1: Write failing Python tests**

`tests/test_simulation.py`:
```python
import pytest
from pyrevm_trace import EVMSimulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"


def test_set_balance():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)


def test_set_code():
    sim = EVMSimulator(chain_id=1)
    # PUSH1 1, PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN — returns uint256(1)
    sim.set_code(CONTRACT, bytes.fromhex("600160005260206000f3"))


def test_set_storage():
    sim = EVMSimulator(chain_id=1)
    sim.set_storage(CONTRACT, slot=0, value=42)


def test_invalid_address_raises():
    sim = EVMSimulator(chain_id=1)
    with pytest.raises(ValueError):
        sim.set_balance("not_an_address", 100)
```

- [ ] **Step 2: Run to confirm AttributeError (expected)**

```bash
pytest tests/test_simulation.py -v 2>&1 | head -20
```

Expected: `AttributeError: 'EVMSimulator' object has no attribute 'set_balance'`

- [ ] **Step 3: Implement state methods in src/executor.rs**

```rust
use pyo3::prelude::*;
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{AccountInfo, Bytecode, Bytes, U256, KECCAK_EMPTY},
};
use crate::types::{parse_address, py_int_to_u256};

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
        EVMSimulator { chain_id, db: CacheDB::new(EmptyDB::default()) }
    }

    pub fn set_balance(&mut self, address: &str, balance: u128) -> PyResult<()> {
        let addr = parse_address(address)?;
        // Preserve existing code if any
        let (code_hash, code) = self.db
            .load_account(addr)
            .ok()
            .and_then(|acc| acc.info.code.clone().map(|c| (acc.info.code_hash, Some(c))))
            .unwrap_or((KECCAK_EMPTY, None));
        self.db.insert_account_info(addr, AccountInfo {
            balance: py_int_to_u256(balance),
            nonce: 0,
            code_hash,
            code,
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
}
```

- [ ] **Step 4: Rebuild and run tests**

```bash
maturin develop && pytest tests/test_simulation.py -v
```

Expected: 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/executor.rs src/types.rs tests/test_simulation.py
git commit -m "feat: EVM state management (set_balance, set_code, set_storage)"
```

---

## Task 4: Basic EVM Transaction Simulation

**Files:**
- Modify: `src/executor.rs` (add `simulate` method)

- [ ] **Step 1: Write failing tests**

Append to `tests/test_simulation.py`:
```python
RETURN_ONE = bytes.fromhex("600160005260206000f3")  # returns uint256(1)
REVERT_ALL = bytes.fromhex("60006000fd")             # PUSH1 0, PUSH1 0, REVERT


def test_simulate_success():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(
        {"from": SENDER, "to": CONTRACT, "data": b"", "gas_limit": 100_000},
        with_trace=False,
        with_gas_profile=False,
    )
    assert result["success"] is True
    assert result["gas_used"] > 0
    assert result["output"][-1] == 1  # last byte of ABI-encoded uint256(1)


def test_simulate_revert():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, REVERT_ALL)

    result = sim.simulate(
        {"from": SENDER, "to": CONTRACT, "data": b"", "gas_limit": 100_000},
        with_trace=False,
        with_gas_profile=False,
    )
    assert result["success"] is False
    assert result["gas_used"] > 0


def test_simulate_eth_transfer():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    recipient = "0xAb5801a7D398351b8bE11C439e05C5B3259aeC9B"

    result = sim.simulate(
        {"from": SENDER, "to": recipient, "value": 10**17, "gas_limit": 21_000},
        with_trace=False,
        with_gas_profile=False,
    )
    assert result["success"] is True
    assert result["gas_used"] == 21_000
```

- [ ] **Step 2: Run to confirm AttributeError**

```bash
pytest tests/test_simulation.py::test_simulate_success -v 2>&1 | head -10
```

Expected: `AttributeError: 'EVMSimulator' object has no attribute 'simulate'`

- [ ] **Step 3: Add simulate to src/executor.rs**

Add these use statements at the top of `src/executor.rs`:
```rust
use pyo3::types::{PyBytes, PyDict, PyList};
use pyo3::Python;
use revm::{
    primitives::{ExecutionResult, Output, TxKind},
    Evm,
};
```

Add `simulate` inside the `#[pymethods]` impl block:
```rust
#[pyo3(signature = (tx, with_trace = false, with_gas_profile = false))]
pub fn simulate(
    &mut self,
    py: Python<'_>,
    tx: &Bound<'_, PyDict>,
    with_trace: bool,
    with_gas_profile: bool,
) -> PyResult<PyObject> {
    let from: String = tx
        .get_item("from")?
        .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("'from' is required"))?
        .extract()?;
    let to_str: Option<String> = tx.get_item("to").ok().flatten()
        .map(|v| v.extract()).transpose()?;
    let data: Vec<u8> = tx.get_item("data").ok().flatten()
        .map(|v| v.extract()).transpose()?.unwrap_or_default();
    let value: u128 = tx.get_item("value").ok().flatten()
        .map(|v| v.extract()).transpose()?.unwrap_or(0);
    let gas_limit: u64 = tx.get_item("gas_limit").ok().flatten()
        .map(|v| v.extract()).transpose()?.unwrap_or(30_000_000);
    let gas_price: u128 = tx.get_item("gas_price").ok().flatten()
        .map(|v| v.extract()).transpose()?.unwrap_or(1);

    let caller = parse_address(&from)?;
    let transact_to = match to_str {
        Some(ref addr) => TxKind::Call(parse_address(addr)?),
        None => TxKind::Create,
    };
    let chain_id = self.chain_id;

    // Build shared tx setup closure
    let data_bytes = Bytes::from(data);
    let setup = |tx_env: &mut revm::primitives::TxEnv| {
        tx_env.caller = caller;
        tx_env.transact_to = transact_to.clone();
        tx_env.value = U256::from(value);
        tx_env.data = data_bytes.clone();
        tx_env.gas_limit = gas_limit;
        tx_env.gas_price = U256::from(gas_price);
    };

    // Execute (tracing wired in Tasks 5 & 6; no-op paths here)
    let (exec_result, maybe_trace, gas_steps) = {
        let result = Evm::builder()
            .with_db(&mut self.db)
            .modify_cfg_env(|cfg| { cfg.chain_id = chain_id; })
            .modify_tx_env(setup)
            .build()
            .transact_commit()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;
        (result, None::<crate::tracer::CallFrame>, vec![])
    };

    Self::build_result_dict(py, exec_result, maybe_trace, gas_steps)
}
```

Add a private helper **outside** the `#[pymethods]` block:
```rust
impl EVMSimulator {
    fn build_result_dict(
        py: Python<'_>,
        result: ExecutionResult,
        maybe_trace: Option<crate::tracer::CallFrame>,
        gas_steps: Vec<crate::gas_profiler::GasStep>,
    ) -> PyResult<PyObject> {
        let out = PyDict::new(py);
        match result {
            ExecutionResult::Success { gas_used, output, logs, .. } => {
                out.set_item("success", true)?;
                out.set_item("gas_used", gas_used)?;
                let raw = match output {
                    Output::Call(b) => b.to_vec(),
                    Output::Create(b, _) => b.to_vec(),
                };
                out.set_item("output", PyBytes::new(py, &raw))?;
                let py_logs = PyList::empty(py);
                for log in &logs {
                    let ld = PyDict::new(py);
                    ld.set_item("address", format!("{:?}", log.address))?;
                    let topics: Vec<String> =
                        log.data.topics().iter().map(|t| format!("{:?}", t)).collect();
                    ld.set_item("topics", topics)?;
                    ld.set_item("data", PyBytes::new(py, &log.data.data))?;
                    py_logs.append(ld)?;
                }
                out.set_item("logs", py_logs)?;
            }
            ExecutionResult::Revert { gas_used, output } => {
                out.set_item("success", false)?;
                out.set_item("gas_used", gas_used)?;
                out.set_item("output", PyBytes::new(py, &output))?;
                out.set_item("logs", PyList::empty(py))?;
            }
            ExecutionResult::Halt { reason, gas_used } => {
                out.set_item("success", false)?;
                out.set_item("gas_used", gas_used)?;
                out.set_item("output", PyBytes::new(py, &[]))?;
                out.set_item("halt_reason", format!("{:?}", reason))?;
                out.set_item("logs", PyList::empty(py))?;
            }
        }
        // trace and gas steps (populated in Tasks 5 & 6)
        match maybe_trace {
            Some(ref frame) => out.set_item("call_trace", call_frame_to_py(py, frame)?)?,
            None => out.set_item("call_trace", py.None())?,
        }
        let py_steps = PyList::empty(py);
        for step in &gas_steps {
            let s = PyDict::new(py);
            s.set_item("pc", step.pc)?;
            s.set_item("opcode", step.opcode_name.as_str())?;
            s.set_item("opcode_byte", step.opcode)?;
            s.set_item("gas_remaining", step.gas_remaining)?;
            s.set_item("gas_cost", step.gas_cost)?;
            py_steps.append(s)?;
        }
        out.set_item("gas_steps", py_steps)?;
        Ok(out.into())
    }
}

fn call_frame_to_py(py: Python<'_>, frame: &crate::tracer::CallFrame) -> PyResult<PyObject> {
    let d = PyDict::new(py);
    d.set_item("call_type", &frame.call_type)?;
    d.set_item("from", crate::types::address_to_hex(frame.from))?;
    d.set_item("to", crate::types::address_to_hex(frame.to))?;
    d.set_item("value", frame.value.to_string())?;
    d.set_item("gas_limit", frame.gas_limit)?;
    d.set_item("gas_used", frame.gas_used)?;
    d.set_item("input", PyBytes::new(py, &frame.input))?;
    d.set_item("output", PyBytes::new(py, &frame.output))?;
    d.set_item("success", frame.success)?;
    let subcalls = PyList::empty(py);
    for child in &frame.subcalls {
        subcalls.append(call_frame_to_py(py, child)?)?;
    }
    d.set_item("subcalls", subcalls)?;
    Ok(d.into())
}
```

Also add stub types so `src/executor.rs` compiles before Tasks 5 & 6 fill in the modules:

`src/tracer.rs` (minimal stub):
```rust
use revm::primitives::{Address, Bytes, U256};

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub call_type: String,
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas_limit: u64,
    pub input: Bytes,
    pub output: Bytes,
    pub gas_used: u64,
    pub success: bool,
    pub subcalls: Vec<CallFrame>,
}
```

`src/gas_profiler.rs` (minimal stub):
```rust
#[derive(Debug, Clone)]
pub struct GasStep {
    pub pc: usize,
    pub opcode: u8,
    pub opcode_name: String,
    pub gas_remaining: u64,
    pub gas_cost: u64,
}
```

- [ ] **Step 4: Rebuild and run tests**

```bash
maturin develop && pytest tests/test_simulation.py -v
```

Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/executor.rs src/tracer.rs src/gas_profiler.rs
git commit -m "feat: basic EVM transaction simulation"
```

---

## Task 5: Call Tree Tracer (Inspector)

**Files:**
- Modify: `src/tracer.rs` (full Inspector impl)
- Modify: `src/executor.rs` (use tracer when `with_trace=True`)
- Create: `tests/test_trace.py`

- [ ] **Step 1: Write failing test**

`tests/test_trace.py`:
```python
from pyrevm_trace import EVMSimulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"
RETURN_ONE = bytes.fromhex("600160005260206000f3")


def test_call_trace_shape():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(
        {"from": SENDER, "to": CONTRACT, "data": b"", "gas_limit": 100_000},
        with_trace=True,
        with_gas_profile=False,
    )
    assert result["success"] is True
    trace = result["call_trace"]
    assert trace is not None
    assert trace["from"].lower() == SENDER.lower()
    assert trace["to"].lower() == CONTRACT.lower()
    assert trace["success"] is True
    assert "gas_used" in trace
    assert isinstance(trace["subcalls"], list)


def test_no_trace_when_disabled():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(
        {"from": SENDER, "to": CONTRACT, "data": b"", "gas_limit": 100_000},
        with_trace=False,
        with_gas_profile=False,
    )
    assert result["call_trace"] is None
```

- [ ] **Step 2: Run to confirm failure**

```bash
maturin develop && pytest tests/test_trace.py::test_call_trace_shape -v 2>&1 | head -15
```

Expected: `AssertionError: assert None is not None` (trace disabled in Task 4 stub).

- [ ] **Step 3: Implement CallTracer Inspector in src/tracer.rs**

Replace the stub content with:
```rust
use revm::{
    interpreter::{CallInputs, CallOutcome, CallScheme},
    primitives::{Address, Bytes, U256},
    Database, EvmContext, Inspector,
};

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub call_type: String,
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas_limit: u64,
    pub input: Bytes,
    pub output: Bytes,
    pub gas_used: u64,
    pub success: bool,
    pub subcalls: Vec<CallFrame>,
}

pub struct CallTracer {
    stack: Vec<CallFrame>,
    pub root: Option<CallFrame>,
}

impl CallTracer {
    pub fn new() -> Self {
        CallTracer { stack: Vec::new(), root: None }
    }
}

impl<DB: Database> Inspector<DB> for CallTracer {
    fn call(
        &mut self,
        _ctx: &mut EvmContext<DB>,
        inputs: &mut CallInputs,
    ) -> Option<CallOutcome> {
        let call_type = match inputs.scheme {
            CallScheme::Call => "CALL",
            CallScheme::StaticCall => "STATICCALL",
            CallScheme::DelegateCall => "DELEGATECALL",
            CallScheme::CallCode => "CALLCODE",
        };
        self.stack.push(CallFrame {
            call_type: call_type.to_string(),
            from: inputs.caller,
            to: inputs.bytecode_address,
            value: inputs.call_value(),
            gas_limit: inputs.gas_limit,
            input: inputs.input.clone(),
            output: Bytes::new(),
            gas_used: 0,
            success: false,
            subcalls: Vec::new(),
        });
        None
    }

    fn call_end(
        &mut self,
        _ctx: &mut EvmContext<DB>,
        _inputs: &CallInputs,
        outcome: CallOutcome,
    ) -> CallOutcome {
        if let Some(mut frame) = self.stack.pop() {
            frame.success = outcome.result.is_success();
            frame.output = outcome.result.output.clone();
            frame.gas_used = frame.gas_limit.saturating_sub(outcome.gas().remaining());
            if let Some(parent) = self.stack.last_mut() {
                parent.subcalls.push(frame);
            } else {
                self.root = Some(frame);
            }
        }
        outcome
    }
}
```

- [ ] **Step 4: Wire tracer into src/executor.rs**

Add imports:
```rust
use crate::tracer::CallTracer;
use revm::inspector_handle_register;
```

In `simulate`, replace the no-op `(exec_result, maybe_trace, gas_steps)` block with:
```rust
let (exec_result, maybe_trace, gas_steps) = if with_trace {
    let mut tracer = CallTracer::new();
    let result = Evm::builder()
        .with_db(&mut self.db)
        .with_external_context(&mut tracer)
        .append_handler_register(inspector_handle_register)
        .modify_cfg_env(|cfg| { cfg.chain_id = chain_id; })
        .modify_tx_env(setup)
        .build()
        .transact_commit()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;
    (result, tracer.root, vec![])
} else {
    let result = Evm::builder()
        .with_db(&mut self.db)
        .modify_cfg_env(|cfg| { cfg.chain_id = chain_id; })
        .modify_tx_env(setup)
        .build()
        .transact_commit()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;
    (result, None, vec![])
};
```

- [ ] **Step 5: Rebuild and run tests**

```bash
maturin develop && pytest tests/test_simulation.py tests/test_trace.py -v
```

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/tracer.rs src/executor.rs tests/test_trace.py
git commit -m "feat: call tree tracer via REVM Inspector"
```

---

## Task 6: Gas Profiler (Opcode-Level)

**Files:**
- Modify: `src/gas_profiler.rs` (full Inspector impl)
- Modify: `src/executor.rs` (use profiler when `with_gas_profile=True`)
- Create: `tests/test_gas_profile.py`

- [ ] **Step 1: Write failing test**

`tests/test_gas_profile.py`:
```python
from pyrevm_trace import EVMSimulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"
RETURN_ONE = bytes.fromhex("600160005260206000f3")


def test_gas_steps_returned():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(
        {"from": SENDER, "to": CONTRACT, "data": b"", "gas_limit": 100_000},
        with_trace=False,
        with_gas_profile=True,
    )
    assert result["success"] is True
    steps = result["gas_steps"]
    assert len(steps) > 0
    step = steps[0]
    assert {"pc", "opcode", "gas_remaining", "gas_cost"} <= step.keys()


def test_gas_opcode_names():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(
        {"from": SENDER, "to": CONTRACT, "data": b"", "gas_limit": 100_000},
        with_trace=False,
        with_gas_profile=True,
    )
    opcodes = [s["opcode"] for s in result["gas_steps"]]
    # RETURN_ONE bytecode runs PUSH1, PUSH1, MSTORE, PUSH1, PUSH1, RETURN
    assert "PUSH1" in opcodes
    assert "MSTORE" in opcodes
    assert "RETURN" in opcodes


def test_no_gas_steps_when_disabled():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = sim.simulate(
        {"from": SENDER, "to": CONTRACT, "data": b"", "gas_limit": 100_000},
        with_trace=False,
        with_gas_profile=False,
    )
    assert result["gas_steps"] == []
```

- [ ] **Step 2: Run to confirm failure**

```bash
pytest tests/test_gas_profile.py::test_gas_steps_returned -v 2>&1 | head -10
```

Expected: `AssertionError: assert [] != []` (empty steps).

- [ ] **Step 3: Implement GasProfiler in src/gas_profiler.rs**

Replace stub with:
```rust
use revm::{
    interpreter::Interpreter,
    Database, EvmContext, Inspector,
};

#[derive(Debug, Clone)]
pub struct GasStep {
    pub pc: usize,
    pub opcode: u8,
    pub opcode_name: String,
    pub gas_remaining: u64,
    pub gas_cost: u64,
}

pub struct GasProfiler {
    pub steps: Vec<GasStep>,
    prev_gas: u64,
}

impl GasProfiler {
    pub fn new() -> Self {
        GasProfiler { steps: Vec::new(), prev_gas: 0 }
    }
}

impl<DB: Database> Inspector<DB> for GasProfiler {
    fn step(&mut self, interp: &mut Interpreter, _ctx: &mut EvmContext<DB>) {
        let gas_remaining = interp.gas.remaining();
        let gas_cost = self.prev_gas.saturating_sub(gas_remaining);
        let opcode = interp.current_opcode();
        let opcode_name = revm::interpreter::opcode::OpCode::new(opcode)
            .map(|op| op.as_str().to_string())
            .unwrap_or_else(|| format!("0x{:02x}", opcode));
        self.steps.push(GasStep { pc: interp.program_counter(), opcode, opcode_name, gas_remaining, gas_cost });
        self.prev_gas = gas_remaining;
    }
}
```

- [ ] **Step 4: Wire profiler into src/executor.rs**

Add import:
```rust
use crate::gas_profiler::GasProfiler;
```

Replace the `with_trace` branch block in `simulate` with a 3-way match:
```rust
let (exec_result, maybe_trace, gas_steps) = match (with_trace, with_gas_profile) {
    (true, _) => {
        let mut tracer = CallTracer::new();
        let result = Evm::builder()
            .with_db(&mut self.db)
            .with_external_context(&mut tracer)
            .append_handler_register(inspector_handle_register)
            .modify_cfg_env(|cfg| { cfg.chain_id = chain_id; })
            .modify_tx_env(setup)
            .build()
            .transact_commit()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;
        (result, tracer.root, vec![])
    }
    (false, true) => {
        let mut profiler = GasProfiler::new();
        let result = Evm::builder()
            .with_db(&mut self.db)
            .with_external_context(&mut profiler)
            .append_handler_register(inspector_handle_register)
            .modify_cfg_env(|cfg| { cfg.chain_id = chain_id; })
            .modify_tx_env(setup)
            .build()
            .transact_commit()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;
        (result, None, profiler.steps)
    }
    (false, false) => {
        let result = Evm::builder()
            .with_db(&mut self.db)
            .modify_cfg_env(|cfg| { cfg.chain_id = chain_id; })
            .modify_tx_env(setup)
            .build()
            .transact_commit()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;
        (result, None, vec![])
    }
};
```

- [ ] **Step 5: Rebuild and run all tests so far**

```bash
maturin develop && pytest tests/ -v
```

Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/gas_profiler.rs src/executor.rs tests/test_gas_profile.py
git commit -m "feat: opcode-level gas profiler"
```

---

## Task 7: Python Pydantic Models

**Files:**
- Create: `python/pyrevm_trace/models.py`

- [ ] **Step 1: Write failing test**

Append to `tests/test_simulation.py`:
```python
from pyrevm_trace.models import SimulationResult

RETURN_ONE = bytes.fromhex("600160005260206000f3")


def test_pydantic_wraps_raw_result():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, RETURN_ONE)

    raw = sim.simulate(
        {"from": SENDER, "to": CONTRACT, "data": b"", "gas_limit": 100_000},
        with_trace=True,
        with_gas_profile=False,
    )
    result = SimulationResult.model_validate(raw)
    assert result.success is True
    assert result.gas_used > 0
    assert result.call_trace is not None
    assert result.call_trace.from_addr.startswith("0x")
    assert isinstance(result.call_trace.subcalls, list)


def test_pydantic_gas_steps():
    sim = EVMSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, RETURN_ONE)

    raw = sim.simulate(
        {"from": SENDER, "to": CONTRACT, "data": b"", "gas_limit": 100_000},
        with_trace=False,
        with_gas_profile=True,
    )
    result = SimulationResult.model_validate(raw)
    assert len(result.gas_steps) > 0
    assert result.gas_steps[0].opcode != ""
```

- [ ] **Step 2: Run to confirm ImportError**

```bash
pytest tests/test_simulation.py::test_pydantic_wraps_raw_result -v 2>&1 | head -10
```

Expected: `ImportError: cannot import name 'SimulationResult'`

- [ ] **Step 3: Create python/pyrevm_trace/models.py**

```python
from __future__ import annotations

from pydantic import BaseModel, Field


class Log(BaseModel):
    address: str
    topics: list[str]
    data: bytes


class GasStep(BaseModel):
    pc: int
    opcode: str
    opcode_byte: int
    gas_remaining: int
    gas_cost: int


class CallFrame(BaseModel):
    call_type: str
    from_addr: str = Field(alias="from")
    to_addr: str = Field(alias="to")
    value: str
    gas_limit: int
    gas_used: int
    input: bytes
    output: bytes
    success: bool
    subcalls: list[CallFrame] = []

    model_config = {"populate_by_name": True}


class SimulationResult(BaseModel):
    success: bool
    gas_used: int
    output: bytes
    logs: list[Log] = []
    call_trace: CallFrame | None = None
    gas_steps: list[GasStep] = []
    halt_reason: str | None = None
```

- [ ] **Step 4: Run tests**

```bash
pytest tests/test_simulation.py -k "pydantic" -v
```

Expected: 2 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add python/pyrevm_trace/models.py tests/test_simulation.py
git commit -m "feat: Pydantic models for simulation results"
```

---

## Task 8: Sync and Async Python Clients

**Files:**
- Create: `python/pyrevm_trace/simulator.py`
- Create: `python/pyrevm_trace/async_simulator.py`
- Modify: `python/pyrevm_trace/__init__.py`
- Create: `tests/test_async.py`

The sync client is a thin typed wrapper. The async client offloads CPU work with `asyncio.to_thread` so it doesn't block the event loop — the right pattern for CPU-bound work in async frameworks.

- [ ] **Step 1: Write failing tests**

Append to `tests/test_simulation.py`:
```python
from pyrevm_trace.simulator import Simulator


def test_sync_simulator():
    sim = Simulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, bytes.fromhex("600160005260206000f3"))

    result = sim.simulate(
        from_addr=SENDER,
        to=CONTRACT,
        gas_limit=100_000,
        with_trace=True,
    )
    assert result.success is True
    assert result.call_trace is not None
```

`tests/test_async.py`:
```python
import pytest
import pytest_asyncio
from pyrevm_trace.async_simulator import AsyncSimulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"
RETURN_ONE = bytes.fromhex("600160005260206000f3")

pytestmark = pytest.mark.asyncio


async def test_async_simulate_success():
    sim = AsyncSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, RETURN_ONE)

    result = await sim.simulate(
        from_addr=SENDER,
        to=CONTRACT,
        gas_limit=100_000,
        with_trace=True,
    )
    assert result.success is True
    assert result.call_trace is not None


async def test_async_simulate_concurrent():
    """Two independent sims running concurrently should not interfere."""
    import asyncio

    sim1 = AsyncSimulator(chain_id=1)
    sim1.set_balance(SENDER, 10**18)
    sim1.set_code(CONTRACT, RETURN_ONE)

    sim2 = AsyncSimulator(chain_id=1)
    sim2.set_balance(SENDER, 10**18)
    sim2.set_code(CONTRACT, RETURN_ONE)

    r1, r2 = await asyncio.gather(
        sim1.simulate(from_addr=SENDER, to=CONTRACT, gas_limit=100_000),
        sim2.simulate(from_addr=SENDER, to=CONTRACT, gas_limit=100_000),
    )
    assert r1.success is True
    assert r2.success is True
```

- [ ] **Step 2: Run to confirm ImportError**

```bash
pip install pytest-asyncio
pytest tests/test_async.py -v 2>&1 | head -10
```

Expected: `ImportError: cannot import name 'AsyncSimulator'`

- [ ] **Step 3: Create python/pyrevm_trace/simulator.py**

```python
from __future__ import annotations

from pyrevm_trace._pyrevm_trace import EVMSimulator as _RustEVM
from pyrevm_trace.models import SimulationResult


class Simulator:
    """Synchronous EVM simulator. Thread-safe per instance (each holds its own state DB)."""

    def __init__(self, chain_id: int = 1) -> None:
        self._evm = _RustEVM(chain_id=chain_id)

    def set_balance(self, address: str, balance: int) -> None:
        self._evm.set_balance(address, balance)

    def set_code(self, address: str, bytecode: bytes) -> None:
        self._evm.set_code(address, bytecode)

    def set_storage(self, address: str, slot: int, value: int) -> None:
        self._evm.set_storage(address, slot, value)

    def simulate(
        self,
        from_addr: str,
        to: str | None = None,
        data: bytes = b"",
        value: int = 0,
        gas_limit: int = 30_000_000,
        gas_price: int = 1,
        with_trace: bool = False,
        with_gas_profile: bool = False,
    ) -> SimulationResult:
        tx: dict = {
            "from": from_addr,
            "data": data,
            "value": value,
            "gas_limit": gas_limit,
            "gas_price": gas_price,
        }
        if to is not None:
            tx["to"] = to
        raw = self._evm.simulate(tx, with_trace=with_trace, with_gas_profile=with_gas_profile)
        return SimulationResult.model_validate(raw)
```

- [ ] **Step 4: Create python/pyrevm_trace/async_simulator.py**

```python
from __future__ import annotations

import asyncio
from functools import partial

from pyrevm_trace.simulator import Simulator
from pyrevm_trace.models import SimulationResult


class AsyncSimulator:
    """
    Async EVM simulator.

    Wraps Simulator and offloads CPU-bound execution to a thread pool via
    asyncio.to_thread so the event loop is never blocked. Each instance holds
    independent EVM state — safe to use concurrently across tasks.
    """

    def __init__(self, chain_id: int = 1) -> None:
        self._sim = Simulator(chain_id=chain_id)

    def set_balance(self, address: str, balance: int) -> None:
        self._sim.set_balance(address, balance)

    def set_code(self, address: str, bytecode: bytes) -> None:
        self._sim.set_code(address, bytecode)

    def set_storage(self, address: str, slot: int, value: int) -> None:
        self._sim.set_storage(address, slot, value)

    async def simulate(
        self,
        from_addr: str,
        to: str | None = None,
        data: bytes = b"",
        value: int = 0,
        gas_limit: int = 30_000_000,
        gas_price: int = 1,
        with_trace: bool = False,
        with_gas_profile: bool = False,
    ) -> SimulationResult:
        fn = partial(
            self._sim.simulate,
            from_addr=from_addr,
            to=to,
            data=data,
            value=value,
            gas_limit=gas_limit,
            gas_price=gas_price,
            with_trace=with_trace,
            with_gas_profile=with_gas_profile,
        )
        return await asyncio.to_thread(fn)
```

- [ ] **Step 5: Update python/pyrevm_trace/__init__.py**

```python
from pyrevm_trace._pyrevm_trace import EVMSimulator
from pyrevm_trace.simulator import Simulator
from pyrevm_trace.async_simulator import AsyncSimulator

__all__ = ["EVMSimulator", "Simulator", "AsyncSimulator"]
```

- [ ] **Step 6: Add pytest-asyncio config to pyproject.toml**

Append to `pyproject.toml`:
```toml
[tool.pytest.ini_options]
asyncio_mode = "auto"
```

- [ ] **Step 7: Run all tests**

```bash
maturin develop && pytest tests/ -v
```

Expected: All tests PASS.

- [ ] **Step 8: Commit**

```bash
git add python/pyrevm_trace/simulator.py python/pyrevm_trace/async_simulator.py \
        python/pyrevm_trace/__init__.py tests/test_simulation.py tests/test_async.py \
        pyproject.toml
git commit -m "feat: sync Simulator and async AsyncSimulator Python clients"
```

---

## Task 9: README and Cleanup

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Write README.md**

```markdown
# pyrevm-trace

Fast EVM simulation and tracing for Python, powered by [REVM](https://github.com/bluealloy/revm) via PyO3.

## Install

Requires Rust toolchain (`rustup`) and `maturin`:

```bash
pip install maturin
maturin develop          # builds Rust extension in-place (dev mode)
pip install -e ".[dev]"  # install Python deps + test deps
```

## Quick start — sync

```python
from pyrevm_trace.simulator import Simulator

sim = Simulator(chain_id=1)
sim.set_balance("0xYourAddress...", 10**18)
sim.set_code("0xContract...", bytes.fromhex("600160005260206000f3"))

result = sim.simulate(
    from_addr="0xYourAddress...",
    to="0xContract...",
    gas_limit=100_000,
    with_trace=True,
)
print(result.success, result.gas_used)
print(result.call_trace)
```

## Quick start — async

```python
import asyncio
from pyrevm_trace.async_simulator import AsyncSimulator

async def main():
    sim = AsyncSimulator(chain_id=1)
    sim.set_balance("0xYourAddress...", 10**18)
    sim.set_code("0xContract...", bytes.fromhex("600160005260206000f3"))

    result = await sim.simulate(
        from_addr="0xYourAddress...",
        to="0xContract...",
        gas_limit=100_000,
        with_gas_profile=True,
    )
    for step in result.gas_steps:
        print(step.pc, step.opcode, step.gas_cost)

asyncio.run(main())
```

## API

### `Simulator` / `AsyncSimulator`

| Method | Description |
|--------|-------------|
| `set_balance(addr, wei)` | Override account balance |
| `set_code(addr, bytecode)` | Deploy bytecode at address |
| `set_storage(addr, slot, value)` | Set storage slot |
| `simulate(from_addr, to, data, value, gas_limit, gas_price, with_trace, with_gas_profile)` | Execute tx, return `SimulationResult` |

### `SimulationResult`

```python
result.success      # bool
result.gas_used     # int
result.output       # bytes
result.logs         # list[Log]
result.call_trace   # CallFrame | None   (with_trace=True)
result.gas_steps    # list[GasStep]      (with_gas_profile=True)
```
```

- [ ] **Step 2: Run final test suite**

```bash
pytest tests/ -v
```

Expected: All tests PASS with zero failures.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: README with sync/async usage examples"
```

---

## Verification

```bash
# Build
maturin develop

# Full test suite
pytest tests/ -v

# Manual smoke test
python - <<'EOF'
import asyncio
from pyrevm_trace.async_simulator import AsyncSimulator

SENDER = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
CONTRACT = "0x1234567890123456789012345678901234567890"

async def main():
    sim = AsyncSimulator(chain_id=1)
    sim.set_balance(SENDER, 10**18)
    sim.set_code(CONTRACT, bytes.fromhex("600160005260206000f3"))
    r = await sim.simulate(SENDER, to=CONTRACT, gas_limit=100_000, with_trace=True)
    print("success:", r.success, "gas_used:", r.gas_used)
    print("trace from:", r.call_trace.from_addr, "→", r.call_trace.to_addr)

asyncio.run(main())
EOF
```

---

## Notes for Implementer

1. **REVM API is version-sensitive.** If `TxKind` doesn't exist, try `TransactTo`. Run `cargo doc --open` to check actual types in the pinned version.

2. **PyO3 0.22 uses `Bound<'_, T>`** for Python object refs. If examples online show `&PyDict`, they target older pyo3 — use `&Bound<'_, PyDict>` instead.

3. **`maturin develop` must be re-run after every Rust change** before running Python tests.

4. **`AsyncSimulator` is safe for concurrent use** because each instance has its own `EVMSimulator` (independent CacheDB). It is NOT safe to share one `AsyncSimulator` across concurrent tasks.

5. **`with_trace=True` takes priority** over `with_gas_profile=True` when both are set. Combining them requires a composite Inspector — leave as a future enhancement.
