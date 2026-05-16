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
