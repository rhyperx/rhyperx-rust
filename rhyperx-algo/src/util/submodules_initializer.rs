#![cfg(feature = "bindings")]

/// This allows to import like this: from module.submodule import something
use pyo3::{
    Bound, PyResult,
    types::{PyAnyMethods, PyModule, PyModuleMethods},
};

fn _inner_init(module: &Bound<'_, PyModule>, path: &str) -> PyResult<()> {
    let py = module.py();
    let sys_modules = py.import("sys")?.getattr("modules")?;

    for submod_name in module.index()? {
        let submod_name = submod_name.extract::<String>()?;
        let submod = module.getattr(&submod_name)?;
        let modpath = format!("{path}.{submod_name}");
        if let Ok(submod) = submod.cast::<PyModule>() {
            _inner_init(submod, &modpath)?;
            sys_modules.set_item(&modpath, submod)?;
        }
    }
    Ok(())
}

pub trait PyModuleSubmoduleExt {
    fn init_submodules(&self) -> pyo3::PyResult<()>;
}

impl PyModuleSubmoduleExt for pyo3::Bound<'_, pyo3::types::PyModule> {
    fn init_submodules(&self) -> pyo3::PyResult<()> {
        let mod_path = self.name()?.extract::<String>()?;

        // handles re export of top level
        let mod_path = match mod_path.split_once('.') {
            Some(parts) => parts.0,
            None => mod_path.as_str(),
        };

        _inner_init(self, mod_path)?;
        Ok(())
    }
}
