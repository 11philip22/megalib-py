use ::megalib::{Node, NodeType, RegistrationState, Session};
use pyo3::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[pyclass]
#[derive(Clone)]
struct MegaNode {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    handle: String,
    #[pyo3(get)]
    size: u64,
    #[pyo3(get)]
    timestamp: i64,
    #[pyo3(get)]
    is_file: bool,
    #[pyo3(get)]
    is_folder: bool,
}

impl From<&Node> for MegaNode {
    fn from(n: &Node) -> Self {
        MegaNode {
            name: n.name.clone(),
            handle: n.handle.clone(),
            size: n.size,
            timestamp: n.timestamp,
            is_file: n.node_type == NodeType::File,
            is_folder: n.node_type.is_container(),
        }
    }
}

#[pyclass]
struct MegaRegistrationState {
    inner: RegistrationState,
}

#[pymethods]
impl MegaRegistrationState {
    fn serialize(&self) -> String {
        self.inner.serialize()
    }

    #[staticmethod]
    fn deserialize(s: String) -> PyResult<Self> {
        let state = RegistrationState::deserialize(&s)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(MegaRegistrationState { inner: state })
    }
}

#[pyclass]
struct MegaPublicFile {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    size: u64,
    #[pyo3(get)]
    handle: String,
}

#[pyclass]
struct MegaSession {
    inner: Arc<Mutex<Session>>,
}

#[pymethods]
impl MegaSession {
    #[staticmethod]
    fn login(py: Python<'_>, email: String, password: String) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = Session::login(&email, &password)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            Ok(MegaSession {
                inner: Arc::new(Mutex::new(session)),
            })
        })
    }

    fn refresh<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session
                .refresh()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn list<'p>(&self, py: Python<'p>, path: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = inner.lock().await;
            let nodes = session
                .list(&path, false)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let py_nodes: Vec<MegaNode> = nodes.iter().map(|n| MegaNode::from(*n)).collect();
            Ok(py_nodes)
        })
    }

    fn quota<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            let q = session
                .quota()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Ok((q.total, q.used))
        })
    }

    fn mkdir<'p>(&self, py: Python<'p>, path: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session
                .mkdir(&path)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn rename<'p>(&self, py: Python<'p>, path: String, new_name: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session
                .rename(&path, &new_name)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn mv<'p>(&self, py: Python<'p>, source: String, dest: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session
                .mv(&source, &dest)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn rm<'p>(&self, py: Python<'p>, path: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session
                .rm(&path)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn export<'p>(&self, py: Python<'p>, path: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            let url = session
                .export(&path)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(url)
        })
    }

    fn upload<'p>(
        &self,
        _py: Python<'p>,
        local_path: String,
        remote_path: String,
    ) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(_py, async move {
            let mut session = inner.lock().await;
            session
                .upload(local_path, &remote_path)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok("Upload complete")
        })
    }

    fn download<'p>(
        &self,
        _py: Python<'p>,
        remote_path: String,
        local_path: String,
    ) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(_py, async move {
            let mut session = inner.lock().await;
            let node = session.stat(&remote_path).cloned();

            if let Some(node) = node {
                let file = std::fs::File::create(&local_path)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
                let mut writer = std::io::BufWriter::new(file);

                session.download(&node, &mut writer).await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;
                Ok("Download complete")
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
                    "File not found on Mega",
                ))
            }
        })
    }

    fn get_email<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = inner.lock().await;
            Ok(session.email.clone())
        })
    }
}

#[pyfunction]
fn register(py: Python<'_>, email: String, password: String, name: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let state = ::megalib::register(&email, &password, &name)
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(MegaRegistrationState { inner: state })
    })
}

#[pyfunction]
fn verify_registration<'p>(
    py: Python<'p>,
    state: &MegaRegistrationState,
    signup_key: String,
) -> PyResult<&'p PyAny> {
    let state_inner = state.inner.clone();
    pyo3_asyncio::tokio::future_into_py(py, async move {
        ::megalib::verify_registration(&state_inner, &signup_key)
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(())
    })
}

#[pyfunction]
fn get_public_file_info(py: Python<'_>, url: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let info = ::megalib::get_public_file_info(&url)
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(MegaPublicFile {
            name: info.name,
            size: info.size,
            handle: info.handle,
        })
    })
}

#[pyfunction]
fn download_public_file(py: Python<'_>, url: String, local_path: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let file = std::fs::File::create(&local_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        let mut writer = std::io::BufWriter::new(file);

        ::megalib::download_public_file(&url, &mut writer)
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok("Download complete")
    })
}

#[pymodule]
#[pyo3(name = "megalib")]
fn megalib_backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MegaSession>()?;
    m.add_class::<MegaNode>()?;
    m.add_class::<MegaRegistrationState>()?;
    m.add_class::<MegaPublicFile>()?;
    m.add_function(wrap_pyfunction!(register, m)?)?;
    m.add_function(wrap_pyfunction!(verify_registration, m)?)?;
    m.add_function(wrap_pyfunction!(get_public_file_info, m)?)?;
    m.add_function(wrap_pyfunction!(download_public_file, m)?)?;
    Ok(())
}
