use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(tesserae, TesseraeError, PyException);

#[pymodule]
mod tesserae {
    use pyo3::prelude::*;
    use pyo3::types::PyList;
    use std::path::PathBuf;

    #[pymodule_export]
    use super::TesseraeError;

    fn err(e: tess::Error) -> PyErr {
        TesseraeError::new_err(e.to_string())
    }

    fn to_py(py: Python<'_>, v: tess::Resolved) -> PyResult<Py<PyAny>> {
        use tess::Resolved::*;
        Ok(match v {
            Bool(b) => b.into_pyobject(py)?.to_owned().into_any().unbind(),
            Int(n) => n.into_pyobject(py)?.into_any().unbind(),
            Float(f) => f.into_pyobject(py)?.into_any().unbind(),
            Str(s) | Colour(s) => s.into_pyobject(py)?.into_any().unbind(),
            Port(p) => p.into_pyobject(py)?.into_any().unbind(),
            Size(n) => n.into_pyobject(py)?.into_any().unbind(),
            Duration(d) => d.into_pyobject(py)?.into_any().unbind(), // -> timedelta
            List(items) => {
                let items = items
                    .into_iter()
                    .map(|i| to_py(py, i))
                    .collect::<PyResult<Vec<_>>>()?;
                PyList::new(py, items)?.into_any().unbind()
            }
            _ => unreachable!("Resolved is non_exhaustive; handle new variants here"),
        })
    }

    #[pyclass(frozen)]
    struct Config(tess::Config);

    #[pymethods]
    impl Config {
        fn keys(&self) -> Vec<String> {
            self.0.keys().into_iter().map(String::from).collect()
        }
        fn get(&self, py: Python<'_>, key: &str) -> PyResult<Py<PyAny>> {
            to_py(py, self.0.get(key).map_err(err)?)
        }
        fn line(&self, key: &str) -> PyResult<usize> {
            self.0.line(key).map_err(err)
        }
        fn __getitem__(&self, py: Python<'_>, key: &str) -> PyResult<Py<PyAny>> {
            self.get(py, key)
        }
        fn __contains__(&self, key: &str) -> bool {
            self.0.line(key).is_ok()
        }
    }

    #[pyfunction]
    #[pyo3(signature = (path, env=None))]
    fn load(path: PathBuf, env: Option<&str>) -> PyResult<Config> {
        tess::load(&path, env).map(Config).map_err(err)
    }

    #[pyfunction]
    #[pyo3(signature = (src, env=None))]
    fn loads(src: &str, env: Option<&str>) -> PyResult<Config> {
        tess::load_str(src, env).map(Config).map_err(err)
    }
}
