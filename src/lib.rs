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

#[pyclass(name = "Docker")]
#[derive(Clone, Debug)]
pub struct Pyo3Docker(pub Docker);

#[pymethods]
impl Pyo3Docker {
    #[new]
    #[pyo3(signature = ( uri = SYSTEM_DEFAULT_URI))]
    fn py_new(uri: &str) -> Self {
        Pyo3Docker(Docker::new(uri).unwrap())
    }

    fn version(&self) -> Py<PyAny> {
        let sv = __version(self.clone());
        pythonize_this!(sv)
    }

    fn info(&self) -> Py<PyAny> {
        let si = __info(self.clone());
        pythonize_this!(si)
    }

    fn ping(&self) -> Py<PyAny> {
        let pi = __ping(self.clone());
        pythonize_this!(pi)
    }

    fn data_usage(&self) -> Py<PyAny> {
        let du = __data_usage(self.clone());
        pythonize_this!(du)
    }

    fn containers(&'_ self) -> Pyo3Containers {
        Pyo3Containers::new(self.clone())
    }

    fn images(&'_ self) -> Pyo3Images {
        Pyo3Images::new(self.clone())
    }

    fn networks(&'_ self) -> Pyo3Networks {
        Pyo3Networks::new(self.clone())
    }

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
pub fn docker_pyo3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pyo3Docker>()?;

    m.add_wrapped(wrap_pymodule!(image::image))?;
    m.add_wrapped(wrap_pymodule!(container::container))?;
    m.add_wrapped(wrap_pymodule!(network::network))?;
    m.add_wrapped(wrap_pymodule!(volume::volume))?;

    let sys = PyModule::import(_py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("docker_pyo3.image", m.getattr("image")?)?;
    sys_modules.set_item("docker_pyo3.container", m.getattr("container")?)?;
    sys_modules.set_item("docker_pyo3.network", m.getattr("network")?)?;
    sys_modules.set_item("docker_pyo3.volume", m.getattr("volume")?)?;

    Ok(())
}
