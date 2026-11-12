use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(tesserae, TesseraeError, PyException);

#[pymodule]
mod tesserae {
    use pyo3::exceptions::PyKeyError;
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyList};
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
            Duration(d) => d.into_pyobject(py)?.into_any().unbind(),
            List(items) => {
                let items = items
                    .into_iter()
                    .map(|i| to_py(py, i))
                    .collect::<PyResult<Vec<_>>>()?;
                PyList::new(py, items)?.into_any().unbind()
            }
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
            if self.0.line(key).is_ok() {
                return self.get(py, key);
            }
            match self.tree(py, Some(key))? {
                Some(section) => Ok(section.into_any().unbind()),
                None => Err(PyKeyError::new_err(key.to_string())),
            }
        }

        fn __contains__(&self, key: &str) -> bool {
            self.0.line(key).is_ok() || self.section_keys(Some(key)).next().is_some()
        }

        fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
            Ok(self.tree(py, None)?.unwrap_or_else(|| PyDict::new(py)))
        }
    }

    impl Config {
        fn section_keys<'a>(
            &'a self,
            section: Option<&str>,
        ) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
            let prefix = section.map(|s| format!("{s}/"));
            self.0
                .keys()
                .into_iter()
                .filter_map(move |key| match &prefix {
                    Some(p) => key.strip_prefix(p.as_str()).map(|rest| (key, rest)),
                    None => Some((key, key)),
                })
        }

        fn tree<'py>(
            &self,
            py: Python<'py>,
            section: Option<&str>,
        ) -> PyResult<Option<Bound<'py, PyDict>>> {
            let root = PyDict::new(py);
            let mut found = false;

            for (key, rest) in self.section_keys(section) {
                found = true;
                let mut parts: Vec<&str> = rest.split('/').collect();
                let last = parts.pop().expect("split always yields one part");

                let mut dict = root.clone();
                for part in parts {
                    dict = match dict.get_item(part)? {
                        Some(existing) => existing.cast_into::<PyDict>()?,
                        None => {
                            let child = PyDict::new(py);
                            dict.set_item(part, &child)?;
                            child
                        }
                    };
                }
                dict.set_item(last, self.get(py, key)?)?;
            }

            Ok(found.then_some(root))
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
