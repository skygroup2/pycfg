mod cfg;
mod block;

use pyo3::prelude::*;

use cfg::{Edges, CFGEvm};
use block::{InstructionBlock, StackInfo};

/// A Python module implemented in Rust.
#[pymodule]
fn pycfg(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CFGEvm>()?;
    m.add_class::<Edges>()?;
    m.add_class::<InstructionBlock>()?;
    m.add_class::<StackInfo>()?;
    Ok(())
}
