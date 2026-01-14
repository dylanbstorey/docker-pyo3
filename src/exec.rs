use docker_api::models::ExecInspect200Response;
use docker_api::opts::ExecResizeOpts;
use docker_api::Exec;
use pyo3::exceptions;
use pyo3::prelude::*;
use pythonize::pythonize;

use crate::Pyo3Docker;

#[pymodule]
pub fn exec(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Exec>()?;
    Ok(())
}

/// Represents a Docker exec instance for running commands in containers.
///
/// Use this to inspect or resize an exec session that was created with
/// Container.exec_create().
#[pyclass(name = "Exec")]
pub struct Pyo3Exec {
    exec: Exec,
    exec_id: String,
}

#[pymethods]
impl Pyo3Exec {
    #[new]
    /// Get an existing exec instance by ID.
    ///
    /// Args:
    ///     docker: Docker client instance
    ///     id: Exec instance ID
    ///
    /// Returns:
    ///     Exec: Exec instance
    pub fn new(docker: Pyo3Docker, id: &str) -> Self {
        Pyo3Exec {
            exec: Exec::get(docker.0, id),
            exec_id: id.to_string(),
        }
    }

    /// Get the exec instance ID.
    ///
    /// Returns:
    ///     str: Exec instance ID
    pub fn id(&self) -> String {
        self.exec_id.clone()
    }

    /// Inspect the exec instance to get detailed information.
    ///
    /// Returns information about the exec instance including its running state,
    /// exit code, process config, and container ID.
    ///
    /// Returns:
    ///     dict: Detailed exec information including:
    ///         - id: Exec instance ID
    ///         - running: Whether the exec is running
    ///         - exit_code: Exit code (if completed)
    ///         - process_config: Command and environment
    ///         - container_id: ID of the container
    ///
    /// Raises:
    ///     SystemError: If inspect fails
    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __exec_inspect(&self.exec);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Resize the TTY session for this exec instance.
    ///
    /// This only works if the exec was created with tty=True.
    /// Use this to adjust terminal dimensions for interactive sessions.
    ///
    /// Args:
    ///     width: New terminal width in columns
    ///     height: New terminal height in rows
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If resize fails (e.g., exec not created with TTY)
    #[pyo3(signature = (width, height))]
    pub fn resize(&self, width: u64, height: u64) -> PyResult<()> {
        let opts = ExecResizeOpts::builder()
            .width(width)
            .height(height)
            .build();
        let rv = __exec_resize(&self.exec, &opts);
        match rv {
            Ok(_) => Ok(()),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __exec_inspect(exec: &Exec) -> Result<ExecInspect200Response, docker_api::Error> {
    exec.inspect().await
}

#[tokio::main]
async fn __exec_resize(exec: &Exec, opts: &ExecResizeOpts) -> Result<(), docker_api::Error> {
    exec.resize(opts).await
}
