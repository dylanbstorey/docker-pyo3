use pyo3::exceptions;
use pyo3::prelude::*;

/// Custom error types for docker-pyo3
#[derive(Debug)]
pub enum DockerPyo3Error {
    /// Docker API errors (network, daemon issues, etc.)
    DockerApi(docker_api::Error),
    /// Serialization/deserialization errors
    Serialization(String),
    /// Invalid parameters passed from Python
    InvalidParameter(String),
    /// I/O errors (file operations, etc.)
    Io(std::io::Error),
    /// Authentication/authorization errors
    Auth(String),
    /// Connection errors (daemon not available, etc.)
    Connection(String),
    /// Resource not found (container, image, etc.)
    NotFound(String),
    /// Resource already exists
    AlreadyExists(String),
    /// Operation not supported
    NotSupported(String),
}

impl std::fmt::Display for DockerPyo3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DockerPyo3Error::DockerApi(e) => write!(f, "Docker API error: {}", e),
            DockerPyo3Error::Serialization(e) => write!(f, "Serialization error: {}", e),
            DockerPyo3Error::InvalidParameter(e) => write!(f, "Invalid parameter: {}", e),
            DockerPyo3Error::Io(e) => write!(f, "I/O error: {}", e),
            DockerPyo3Error::Auth(e) => write!(f, "Authentication error: {}", e),
            DockerPyo3Error::Connection(e) => write!(f, "Connection error: {}", e),
            DockerPyo3Error::NotFound(e) => write!(f, "Resource not found: {}", e),
            DockerPyo3Error::AlreadyExists(e) => write!(f, "Resource already exists: {}", e),
            DockerPyo3Error::NotSupported(e) => write!(f, "Operation not supported: {}", e),
        }
    }
}

impl std::error::Error for DockerPyo3Error {}

impl From<docker_api::Error> for DockerPyo3Error {
    fn from(error: docker_api::Error) -> Self {
        DockerPyo3Error::DockerApi(error)
    }
}

impl From<std::io::Error> for DockerPyo3Error {
    fn from(error: std::io::Error) -> Self {
        DockerPyo3Error::Io(error)
    }
}

// Note: serde_json not available in dependencies
// impl From<serde_json::Error> for DockerPyo3Error {
//     fn from(error: serde_json::Error) -> Self {
//         DockerPyo3Error::Serialization(error.to_string())
//     }
// }

/// Convert DockerPyo3Error to appropriate Python exception
impl From<DockerPyo3Error> for PyErr {
    fn from(error: DockerPyo3Error) -> Self {
        match error {
            DockerPyo3Error::DockerApi(e) => {
                // Parse common Docker API errors and map to appropriate Python exceptions
                let error_msg = e.to_string();
                if error_msg.contains("404") || error_msg.contains("not found") || error_msg.contains("No such") {
                    exceptions::PyFileNotFoundError::new_err(format!("Docker resource not found: {}", e))
                } else if error_msg.contains("401") || error_msg.contains("403") {
                    exceptions::PyPermissionError::new_err(format!("Docker permission denied: {}", e))
                } else if error_msg.contains("409") || error_msg.contains("conflict") || error_msg.contains("already exists") {
                    exceptions::PyFileExistsError::new_err(format!("Docker conflict: {}", e))
                } else if error_msg.contains("connection") || error_msg.contains("timeout") || error_msg.contains("refused") || error_msg.contains("connect error") || error_msg.contains("Operation timed out") {
                    exceptions::PyConnectionError::new_err(format!("Docker connection error: {}", e))
                } else if error_msg.contains("400") || error_msg.contains("Bad Request") || error_msg.contains("invalid") {
                    exceptions::PyValueError::new_err(format!("Docker invalid request: {}", e))
                } else if error_msg.contains("500") || error_msg.contains("Internal Server Error") {
                    exceptions::PyRuntimeError::new_err(format!("Docker server error: {}", e))
                } else {
                    exceptions::PyRuntimeError::new_err(format!("Docker error: {}", e))
                }
            },
            DockerPyo3Error::InvalidParameter(msg) => {
                exceptions::PyValueError::new_err(msg)
            },
            DockerPyo3Error::Io(e) => {
                exceptions::PyIOError::new_err(format!("I/O error: {}", e))
            },
            DockerPyo3Error::Auth(msg) => {
                exceptions::PyPermissionError::new_err(msg)
            },
            DockerPyo3Error::Connection(msg) => {
                exceptions::PyConnectionError::new_err(msg)
            },
            DockerPyo3Error::NotFound(msg) => {
                exceptions::PyFileNotFoundError::new_err(msg)
            },
            DockerPyo3Error::AlreadyExists(msg) => {
                exceptions::PyFileExistsError::new_err(msg)
            },
            DockerPyo3Error::NotSupported(msg) => {
                exceptions::PyNotImplementedError::new_err(msg)
            },
            DockerPyo3Error::Serialization(msg) => {
                exceptions::PyValueError::new_err(format!("Data serialization error: {}", msg))
            },
        }
    }
}

/// Convenient macro for creating errors
#[macro_export]
macro_rules! docker_error {
    (InvalidParameter, $msg:expr) => {
        crate::error::DockerPyo3Error::InvalidParameter($msg.to_string())
    };
    (NotFound, $msg:expr) => {
        crate::error::DockerPyo3Error::NotFound($msg.to_string())
    };
    (AlreadyExists, $msg:expr) => {
        crate::error::DockerPyo3Error::AlreadyExists($msg.to_string())
    };
    (Connection, $msg:expr) => {
        crate::error::DockerPyo3Error::Connection($msg.to_string())
    };
    (Auth, $msg:expr) => {
        crate::error::DockerPyo3Error::Auth($msg.to_string())
    };
    (NotSupported, $msg:expr) => {
        crate::error::DockerPyo3Error::NotSupported($msg.to_string())
    };
}

/// Result type alias for convenience
pub type DockerResult<T> = Result<T, DockerPyo3Error>;

/// Convert DockerResult to PyResult
pub trait IntoPyResult<T> {
    fn into_py_result(self) -> PyResult<T>;
}

impl<T> IntoPyResult<T> for DockerResult<T> {
    fn into_py_result(self) -> PyResult<T> {
        self.map_err(PyErr::from)
    }
}