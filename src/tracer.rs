use revm::{
    context_interface::ContextTr,
    inspector::Inspector,
    interpreter::{CallInputs, CallOutcome, interpreter::EthInterpreter},
    primitives::{Address, Bytes, U256},
};

pub struct CallFrame {
    pub from: Address,
    pub to: Address,
    pub calldata: Bytes,
    pub value: U256,
    pub gas_limit: u64,
    pub success: bool,
    pub output: Bytes,
    pub calls: Vec<CallFrame>,
}

pub struct CallTracer {
    stack: Vec<CallFrame>,
    pub root: Option<CallFrame>,
}

impl CallTracer {
    pub fn new() -> Self {
        Self { stack: Vec::new(), root: None }
    }
}

impl<CTX: ContextTr> Inspector<CTX, EthInterpreter> for CallTracer {
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        let calldata = inputs.input.bytes(context);
        self.stack.push(CallFrame {
            from: inputs.caller,
            to: inputs.target_address,
            calldata,
            value: inputs.value.get(),
            gas_limit: inputs.gas_limit,
            success: false,
            output: Bytes::new(),
            calls: Vec::new(),
        });
        None
    }

    fn call_end(&mut self, _context: &mut CTX, _inputs: &CallInputs, outcome: &mut CallOutcome) {
        if let Some(mut frame) = self.stack.pop() {
            frame.success = outcome.result.result.is_ok();
            frame.output = outcome.result.output.clone();
            if let Some(parent) = self.stack.last_mut() {
                parent.calls.push(frame);
            } else {
                self.root = Some(frame);
            }
        }
    }
}
