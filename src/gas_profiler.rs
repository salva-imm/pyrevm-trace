use revm::{
    context_interface::ContextTr,
    inspector::Inspector,
    interpreter::{interpreter::EthInterpreter, interpreter_types::Jumps, Interpreter},
};

pub struct GasStep {
    pub pc: usize,
    pub op: u8,
    pub gas_remaining: u64,
    pub gas_cost: u64,
}

pub struct GasProfiler {
    pub steps: Vec<GasStep>,
}

impl GasProfiler {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }
}

impl<CTX: ContextTr> Inspector<CTX, EthInterpreter> for GasProfiler {
    fn step(&mut self, interp: &mut Interpreter<EthInterpreter>, _context: &mut CTX) {
        self.steps.push(GasStep {
            pc: interp.bytecode.pc(),
            op: interp.bytecode.opcode(),
            gas_remaining: interp.gas.remaining(),
            gas_cost: 0,
        });
    }

    fn step_end(&mut self, interp: &mut Interpreter<EthInterpreter>, _context: &mut CTX) {
        if let Some(step) = self.steps.last_mut() {
            step.gas_cost = step.gas_remaining.saturating_sub(interp.gas.remaining());
        }
    }
}
