use pyo3::prelude::*;

#[pyclass]
pub struct EVMSimulator {}

#[pymethods]
impl EVMSimulator {
    #[new]
    pub fn new() -> Self { EVMSimulator {} }
}
