use std::error::Error;

pub mod cbs;
pub mod cetc;
pub mod forward;
pub mod kclist;

#[cfg(feature = "bindings")]
#[pyo3::pymodule(submodule)]
pub mod triangle {
    use std::error::Error;

    use anyhow::Result;
    use pyo3::pyfunction;
    use pyo3_stub_gen::reexport_module_members;

    #[pymodule_export]
    use super::cetc::cetc;

    #[pymodule_export]
    use super::forward::forward;

    #[pymodule_export]
    use super::kclist::kclist;

    reexport_module_members!("rust_core.triangle" from "rust_core._core.triangle");
}

// impl From<Box<dyn Error>> for pyo3::PyErr {
//     fn from(err: Box<dyn Error>) -> Self {
//         pyo3::exceptions::PyRuntimeError::new_err(err.to_string())
//     }
// }
