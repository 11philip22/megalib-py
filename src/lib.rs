use ::megalib::{Node, NodeType, RegistrationState, Session};
use pyo3::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A file or folder node in MEGA.
///
/// Attributes:
///     name: File/folder name
///     handle: Unique MEGA handle
///     size: Size in bytes (0 for folders)
///     timestamp: Unix timestamp of last modification
///     is_file: True if this is a file
///     is_folder: True if this is a folder
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

/// Registration state for two-step account creation.
///
/// Use `serialize()` to save state, `deserialize()` to restore it.
#[pyclass]
struct MegaRegistrationState {
    inner: RegistrationState,
}

#[pymethods]
impl MegaRegistrationState {
    /// Serialize state to JSON string for storage.
    fn serialize(&self) -> String {
        self.inner.serialize()
    }

    /// Restore state from a JSON string.
    #[staticmethod]
    fn deserialize(s: String) -> PyResult<Self> {
        let state = RegistrationState::deserialize(&s)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(MegaRegistrationState { inner: state })
    }
}

/// Information about a public file link.
///
/// Attributes:
///     name: File name
///     size: Size in bytes
///     handle: MEGA handle
#[pyclass]
struct MegaPublicFile {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    size: u64,
    #[pyo3(get)]
    handle: String,
}

/// Authenticated MEGA session for file operations.
///
/// Create a session using `login()` or `load()`, then call `refresh()` to
/// fetch your file tree before performing operations.
///
/// Example:
///     session = await MegaSession.login("user@example.com", "password")
///     await session.refresh()
///     files = await session.list("/")
#[pyclass]
struct MegaSession {
    inner: Arc<Mutex<Session>>,
}

#[pymethods]
impl MegaSession {
    /// Login to MEGA with email and password.
    ///
    /// Args:
    ///     email: Your MEGA account email
    ///     password: Your MEGA account password
    ///     proxy: Optional HTTP/SOCKS5 proxy URL (e.g., "http://proxy:8080")
    ///
    /// Returns:
    ///     Authenticated MegaSession
    ///
    /// Raises:
    ///     ValueError: If login fails (wrong credentials, etc.)
    #[staticmethod]
    fn login(
        py: Python<'_>,
        email: String,
        password: String,
        proxy: Option<String>,
    ) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let res = if let Some(p) = proxy {
                Session::login_with_proxy(&email, &password, &p).await
            } else {
                Session::login(&email, &password).await
            };

            let session =
                res.map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            Ok(MegaSession {
                inner: Arc::new(Mutex::new(session)),
            })
        })
    }

    /// Refresh the file tree from the server.
    ///
    /// Must be called after login before using list(), stat(), etc.
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

    /// Get information about a file or folder.
    ///
    /// Args:
    ///     path: Path to the file/folder (e.g., "/Root/Documents")
    ///
    /// Returns:
    ///     MegaNode if found, None otherwise
    fn stat<'p>(&self, py: Python<'p>, path: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = inner.lock().await;
            match session.stat(&path) {
                Some(node) => Ok(Some(MegaNode::from(node))),
                None => Ok(None),
            }
        })
    }

    /// List files in a directory.
    ///
    /// Args:
    ///     path: Path to list (e.g., "/", "/Root/Documents")
    ///     recursive: If True, list all descendants recursively
    ///
    /// Returns:
    ///     List of MegaNode objects
    #[pyo3(signature = (path, recursive = false))]
    fn list<'p>(&self, py: Python<'p>, path: String, recursive: bool) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = inner.lock().await;
            let nodes = session
                .list(&path, recursive)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            let py_nodes: Vec<MegaNode> = nodes.iter().map(|n| MegaNode::from(*n)).collect();
            Ok(py_nodes)
        })
    }

    /// Get storage quota information.
    ///
    /// Returns:
    ///     Tuple of (total_bytes, used_bytes)
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

    /// Create a new directory.
    ///
    /// Args:
    ///     path: Full path for the new directory (e.g., "/Root/NewFolder")
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

    /// Rename a file or folder.
    ///
    /// Args:
    ///     path: Path to the item to rename
    ///     new_name: New name (not a path, just the filename)
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

    /// Move a file or folder to a new location.
    ///
    /// Args:
    ///     source: Path to the item to move
    ///     dest: Path to the destination folder
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

    /// Delete a file or folder.
    ///
    /// Args:
    ///     path: Path to the item to delete
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

    /// Create a public download link for a file.
    ///
    /// Args:
    ///     path: Path to the file to export
    ///
    /// Returns:
    ///     Public URL string
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

    /// Upload a file to MEGA.
    ///
    /// Args:
    ///     local_path: Path to local file
    ///     remote_path: Destination folder on MEGA
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

    /// Download a file from MEGA.
    ///
    /// Args:
    ///     remote_path: Path to file on MEGA
    ///     local_path: Destination path on local disk
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

    /// Get the user's email address.
    ///
    /// Returns:
    ///     User's email address as a string
    fn get_email<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = inner.lock().await;
            Ok(session.email.clone())
        })
    }

    /// Get the user's display name.
    ///
    /// Returns:
    ///     User's display name as a string
    fn get_name<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = inner.lock().await;
            Ok(session.name.clone())
        })
    }

    /// Get the user's MEGA handle (unique ID).
    ///
    /// Returns:
    ///     User's MEGA handle as a string
    fn get_handle<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = inner.lock().await;
            Ok(session.user_handle.clone())
        })
    }

    /// Set number of parallel transfer workers.
    ///
    /// Higher values speed up large file transfers.
    ///
    /// Args:
    ///     workers: Number of parallel transfer workers
    fn set_workers<'p>(&self, py: Python<'p>, workers: usize) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session.set_workers(workers);
            Ok(())
        })
    }

    /// Enable/disable resume for interrupted downloads.
    ///
    /// Args:
    ///     enabled: True to enable, False to disable
    fn set_resume<'p>(&self, py: Python<'p>, enabled: bool) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session.set_resume(enabled);
            Ok(())
        })
    }

    /// Enable/disable thumbnail generation on upload.
    ///
    /// Args:
    ///     enabled: True to enable, False to disable
    fn enable_previews<'p>(&self, py: Python<'p>, enabled: bool) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session.enable_previews(enabled);
            Ok(())
        })
    }

    /// Share a folder with another user.
    ///
    /// Args:
    ///     path: Path to folder to share
    ///     email: Email of user to share with
    ///     access_level: 0=read, 1=write, 2=full
    fn share_folder<'p>(
        &self,
        py: Python<'p>,
        path: String,
        email: String,
        access_level: i32,
    ) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session
                .share_folder(&path, &email, access_level)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    /// List all contacts (users you've shared with).
    ///
    /// Returns:
    ///     List of MegaNode objects representing contacts
    fn list_contacts<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = inner.lock().await;
            let contacts = session.list_contacts();
            let py_contacts: Vec<MegaNode> = contacts.iter().map(|n| MegaNode::from(*n)).collect();
            Ok(py_contacts)
        })
    }

    /// Save session to file for later restoration.
    ///
    /// The saved file contains encrypted credentials - keep it secure!
    ///
    /// Args:
    ///     path: Path to save session file
    fn save<'p>(&self, py: Python<'p>, path: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let session = inner.lock().await;
            session
                .save(&path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            Ok(())
        })
    }

    /// Change the user's password.
    ///
    /// Args:
    ///     new_password: New password for the account
    fn change_password<'p>(&self, py: Python<'p>, new_password: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session
                .change_password(&new_password)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    /// Download a file to a specific file path.
    ///
    /// Args:
    ///     remote_path: Path to file on MEGA
    ///     local_path: Destination path on local disk
    fn download_to_file<'p>(
        &self,
        py: Python<'p>,
        remote_path: String,
        local_path: String,
    ) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            let node = session.stat(&remote_path).cloned();

            if let Some(node) = node {
                session
                    .download_to_file(&node, &local_path)
                    .await
                    .map_err(|e| {
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

    /// Upload a file resumable.
    ///
    /// Args:
    ///     local_path: Path to local file
    ///     remote_path: Destination folder on MEGA
    fn upload_resumable<'p>(
        &self,
        py: Python<'p>,
        local_path: String,
        remote_path: String,
    ) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut session = inner.lock().await;
            session
                .upload_resumable(&local_path, &remote_path)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok("Upload complete")
        })
    }

    /// Load a saved session from a file.
    ///
    /// Args:
    ///     path: Path to saved session file
    ///
    /// Returns:
    ///     MegaSession if loaded, None if file not found
    #[staticmethod]
    fn load(py: Python<'_>, path: String) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match Session::load(&path)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?
            {
                Some(session) => Ok(Some(MegaSession {
                    inner: Arc::new(Mutex::new(session)),
                })),
                None => Ok(None),
            }
        })
    }
}

/// Start the registration process for a new MEGA account.
///
/// Args:
///     email: Email address for the new account
///     password: Password for the new account
///     name: Display name for the account
///
/// Returns:
///     MegaRegistrationState to save and use with verify_registration()
#[pyfunction]
fn register(py: Python<'_>, email: String, password: String, name: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let state = ::megalib::register(&email, &password, &name)
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(MegaRegistrationState { inner: state })
    })
}

/// Complete registration using the signup key from email.
///
/// Args:
///     state: MegaRegistrationState from register()
///     signup_key: The confirmation key from the verification email
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

/// Get info about a public file without downloading.
///
/// Args:
///     url: MEGA public link (e.g., "https://mega.nz/file/...")
///
/// Returns:
///     MegaPublicFile with name, size, and handle
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

/// Download a file from a public MEGA link.
///
/// Args:
///     url: MEGA public link
///     local_path: Destination path on local disk
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

/// A public folder for browsing shared folders without login.
///
/// Created via open_folder(). Use list() to browse, download() to get files.
#[pyclass]
struct MegaPublicFolder {
    inner: Arc<::megalib::public::PublicFolder>,
}

#[pymethods]
impl MegaPublicFolder {
    /// List files in a path within the public folder.
    fn list<'p>(&self, py: Python<'p>, path: String) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let nodes = inner.list(&path, false);
            let py_nodes: Vec<MegaNode> = nodes.iter().map(|n| MegaNode::from(*n)).collect();
            Ok(py_nodes)
        })
    }

    /// Download a file from the public folder.
    fn download<'p>(
        &self,
        _py: Python<'p>,
        remote_path: String,
        local_path: String,
    ) -> PyResult<&'p PyAny> {
        let inner = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(_py, async move {
            let node = inner.stat(&remote_path).cloned();

            if let Some(node) = node {
                let file = std::fs::File::create(&local_path)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
                let mut writer = std::io::BufWriter::new(file);

                inner.download(&node, &mut writer).await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;
                Ok("Download complete")
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
                    "File not found in public folder",
                ))
            }
        })
    }
}

/// Open a public folder from a MEGA folder link.
///
/// Args:
///     url: MEGA folder link (e.g., "https://mega.nz/folder/...")
///
/// Returns:
///     MegaPublicFolder for browsing and downloading
#[pyfunction]
fn open_folder(py: Python<'_>, url: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let folder = ::megalib::public::open_folder(&url)
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(MegaPublicFolder {
            inner: Arc::new(folder),
        })
    })
}

#[pymodule]
#[pyo3(name = "megalib")]
fn megalib_backend(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<MegaSession>()?;
    m.add_class::<MegaNode>()?;
    m.add_class::<MegaRegistrationState>()?;
    m.add_class::<MegaPublicFile>()?;
    m.add_class::<MegaPublicFolder>()?;
    m.add_function(wrap_pyfunction!(register, m)?)?;
    m.add_function(wrap_pyfunction!(verify_registration, m)?)?;
    m.add_function(wrap_pyfunction!(get_public_file_info, m)?)?;
    m.add_function(wrap_pyfunction!(download_public_file, m)?)?;
    m.add_function(wrap_pyfunction!(open_folder, m)?)?;
    Ok(())
}
