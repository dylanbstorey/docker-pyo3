#[macro_use]
mod macros;
pub mod container;
pub mod image;
pub mod network;
pub mod volume;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

use docker_api::models::{PingInfo, SystemDataUsage200Response, SystemInfo, SystemVersion};
use docker_api::Docker;

use pythonize::pythonize;

use container::Pyo3Containers;
use image::Pyo3Images;
use network::Pyo3Networks;
use volume::Pyo3Volumes;

#[cfg(unix)]
static SYSTEM_DEFAULT_URI: &str = "unix:///var/run/docker.sock";

#[cfg(not(unix))]
static SYSTEM_DEFAULT_URI: &str = "tcp://localhost:2375";

/// Docker client for interacting with the Docker daemon.
///
/// Examples:
///     >>> docker = Docker()  # Connect to default socket
///     >>> docker = Docker("unix:///var/run/docker.sock")
///     >>> docker = Docker("tcp://localhost:2375")
#[pyclass(name = "Docker")]
#[derive(Clone, Debug)]
pub struct Pyo3Docker(pub Docker);

#[pymethods]
impl Pyo3Docker {
    #[new]
    #[pyo3(signature = ( uri = SYSTEM_DEFAULT_URI))]
    /// Create a new Docker client.
    ///
    /// Args:
    ///     uri: URI to connect to the Docker daemon. Defaults to the system default
    ///          (unix:///var/run/docker.sock on Unix, tcp://localhost:2375 on Windows).
    ///
    /// Returns:
    ///     Docker client instance
    fn py_new(uri: &str) -> Self {
        Pyo3Docker(Docker::new(uri).unwrap())
    }

    /// Get Docker version information.
    ///
    /// Returns:
    ///     dict: Version information including API version, OS, architecture, etc.
    fn version(&self) -> Py<PyAny> {
        let sv = __version(self.clone());
        pythonize_this!(sv)
    }

    /// Get Docker system information.
    ///
    /// Returns:
    ///     dict: System information including containers count, images count, storage driver, etc.
    fn info(&self) -> Py<PyAny> {
        let si = __info(self.clone());
        pythonize_this!(si)
    }

    /// Ping the Docker daemon to verify connectivity.
    ///
    /// Returns:
    ///     dict: Ping response from the daemon
    fn ping(&self) -> Py<PyAny> {
        let pi = __ping(self.clone());
        pythonize_this!(pi)
    }

    /// Get data usage information for Docker objects.
    ///
    /// Returns:
    ///     dict: Data usage statistics for containers, images, volumes, and build cache
    fn data_usage(&self) -> Py<PyAny> {
        let du = __data_usage(self.clone());
        pythonize_this!(du)
    }

    /// Get a Containers interface for managing containers.
    ///
    /// Returns:
    ///     Containers: Interface for container operations
    fn containers(&'_ self) -> Pyo3Containers {
        Pyo3Containers::new(self.clone())
    }

    /// Get an Images interface for managing images.
    ///
    /// Returns:
    ///     Images: Interface for image operations
    fn images(&'_ self) -> Pyo3Images {
        Pyo3Images::new(self.clone())
    }

    /// Get a Networks interface for managing networks.
    ///
    /// Returns:
    ///     Networks: Interface for network operations
    fn networks(&'_ self) -> Pyo3Networks {
        Pyo3Networks::new(self.clone())
    }

    /// Get a Volumes interface for managing volumes.
    ///
    /// Returns:
    ///     Volumes: Interface for volume operations
    fn volumes(&'_ self) -> Pyo3Volumes {
        Pyo3Volumes::new(self.clone())
    }
}

#[tokio::main]
async fn __version(docker: Pyo3Docker) -> SystemVersion {
    let version = docker.0.version().await;
    version.unwrap()
}

#[tokio::main]
async fn __info(docker: Pyo3Docker) -> SystemInfo {
    let info = docker.0.info().await;
    info.unwrap()
}

#[tokio::main]
async fn __ping(docker: Pyo3Docker) -> PingInfo {
    let ping = docker.0.ping().await;
    ping.unwrap()
}

#[tokio::main]
async fn __data_usage(docker: Pyo3Docker) -> SystemDataUsage200Response {
    let du = docker.0.data_usage().await;
    du.unwrap()
}

/// A Python module implemented in Rust.
#[pymodule]
pub fn docker_pyo3(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Docker>()?;

    m.add_wrapped(wrap_pymodule!(image::image))?;
    m.add_wrapped(wrap_pymodule!(container::container))?;
    m.add_wrapped(wrap_pymodule!(network::network))?;
    m.add_wrapped(wrap_pymodule!(volume::volume))?;

    let sys = PyModule::import(_py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.downcast_into()?;
    sys_modules.set_item("docker_pyo3.image", m.getattr("image")?)?;
    sys_modules.set_item("docker_pyo3.container", m.getattr("container")?)?;
    sys_modules.set_item("docker_pyo3.network", m.getattr("network")?)?;
    sys_modules.set_item("docker_pyo3.volume", m.getattr("volume")?)?;

    Ok(())
}
