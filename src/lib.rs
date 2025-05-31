#[macro_use]
mod macros;
pub mod error;
pub mod container;
pub mod image;
pub mod network;
pub mod volume;
pub mod stack;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

use docker_api::models::{PingInfo, SystemDataUsage200Response, SystemInfo, SystemVersion};
use docker_api::Docker;

use pythonize::pythonize;

use std::sync::OnceLock;
use tokio::runtime::Runtime;

use container::Pyo3Containers;
use image::Pyo3Images;
use network::Pyo3Networks;
use volume::Pyo3Volumes;
use stack::{Pyo3Stack, Service};
use error::DockerPyo3Error;

#[cfg(unix)]
static SYSTEM_DEFAULT_URI: &str = "unix:///var/run/docker.sock";

#[cfg(not(unix))]
static SYSTEM_DEFAULT_URI: &str = "tcp://localhost:2375";

// Shared runtime for async operations with optimizations
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4) // Optimized for Docker operations
            .thread_name("docker-pyo3")
            .thread_stack_size(3 * 1024 * 1024) // 3MB stack for async tasks
            .enable_all() // Enable all tokio features
            .build()
            .expect("Failed to create optimized tokio runtime")
    })
}

/// Health check for the runtime
pub fn runtime_health_check() -> bool {
    match RUNTIME.get() {
        Some(runtime) => {
            // Simple check to see if runtime is responsive
            runtime.block_on(async {
                tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    async { true }
                ).await.unwrap_or(false)
            })
        },
        None => false,
    }
}

#[pyclass(name = "Docker")]
#[derive(Clone, Debug)]
pub struct Pyo3Docker(pub Docker);

#[pymethods]
impl Pyo3Docker {
    #[new]
    #[pyo3(signature = ( uri = SYSTEM_DEFAULT_URI))]
    fn py_new(uri: &str) -> PyResult<Self> {
        let docker = Docker::new(uri)
            .map_err(|e| DockerPyo3Error::Connection(format!(
                "Failed to connect to Docker daemon at '{}': {}", uri, e
            )))?;
        
        // Test the connection by pinging the daemon with a timeout
        let test_client = Pyo3Docker(docker.clone());
        
        // Use a shorter timeout for connection testing
        use std::time::Duration;
        use tokio::time::timeout;
        
        let ping_future = async {
            timeout(Duration::from_secs(5), test_client.0.ping()).await
        };
        
        match get_runtime().block_on(ping_future) {
            Ok(Ok(_)) => Ok(Pyo3Docker(docker)),
            Ok(Err(e)) => {
                let error_msg = e.to_string();
                if error_msg.contains("connection") || error_msg.contains("refused") || error_msg.contains("timeout") {
                    Err(DockerPyo3Error::Connection(format!(
                        "Cannot reach Docker daemon at '{}': {}", uri, e
                    )).into())
                } else {
                    Err(DockerPyo3Error::from(e).into())
                }
            }
            Err(_) => Err(DockerPyo3Error::Connection(format!(
                "Docker connection timeout: Failed to connect to '{}' within 5 seconds", uri
            )).into()),
        }
    }

    fn version(&self) -> PyResult<Py<PyAny>> {
        __version(self.clone())
            .map(|version| pythonize_this!(version))
            .map_err(|e| DockerPyo3Error::from(e).into())
    }

    fn info(&self) -> PyResult<Py<PyAny>> {
        __info(self.clone())
            .map(|info| pythonize_this!(info))
            .map_err(|e| DockerPyo3Error::from(e).into())
    }

    fn ping(&self) -> PyResult<Py<PyAny>> {
        __ping(self.clone())
            .map(|ping| pythonize_this!(ping))
            .map_err(|e| DockerPyo3Error::from(e).into())
    }

    fn data_usage(&self) -> PyResult<Py<PyAny>> {
        __data_usage(self.clone())
            .map(|data_usage| pythonize_this!(data_usage))
            .map_err(|e| DockerPyo3Error::from(e).into())
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

    /// Check if the internal async runtime is healthy
    fn runtime_health(&self) -> bool {
        runtime_health_check()
    }

    /// Check if Docker daemon is reachable
    fn daemon_health(&self) -> PyResult<bool> {
        // Quick ping to check daemon connectivity
        match __ping(self.clone()) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get Docker daemon URI that this client is connected to
    fn daemon_uri(&self) -> String {
        // This is a simple implementation - docker-api doesn't expose the URI directly
        // For Unix sockets, we assume the default path unless otherwise specified
        #[cfg(unix)]
        {
            SYSTEM_DEFAULT_URI.to_string()
        }
        #[cfg(not(unix))]
        {
            SYSTEM_DEFAULT_URI.to_string()
        }
    }

    /// Comprehensive health check for Docker client
    fn health_check(&self) -> PyResult<Py<PyAny>> {
        use serde::Serialize;
        
        #[derive(Serialize)]
        struct HealthInfo {
            runtime_healthy: bool,
            daemon_reachable: bool,
            overall_healthy: bool,
            daemon_uri: String,
        }
        
        let runtime_ok = self.runtime_health();
        let daemon_ok = self.daemon_health()?;
        
        let health_info = HealthInfo {
            runtime_healthy: runtime_ok,
            daemon_reachable: daemon_ok,
            overall_healthy: runtime_ok && daemon_ok,
            daemon_uri: self.daemon_uri(),
        };
        
        Ok(pythonize_this!(health_info))
    }
    
    // Phase 2.0 Stack Management Methods
    
    /// Create a new stack for multi-container applications
    fn create_stack(&self, name: String) -> Pyo3Stack {
        Pyo3Stack::new(self.clone(), name)
    }
    
    /// Create a new service for stack deployment
    fn create_service(&self, name: String) -> Service {
        Service::new(name)
    }
    
    /// Import a stack from a docker-compose.yml file
    fn import_stack_from_file(&self, file_path: String) -> PyResult<Pyo3Stack> {
        Pyo3Stack::from_file(self.clone(), file_path)
    }
    
    /// Import a stack from docker-compose YAML content
    fn import_stack_from_yaml(&self, yaml_content: String) -> PyResult<Pyo3Stack> {
        Pyo3Stack::from_yaml(self.clone(), yaml_content)
    }
}

fn __version(docker: Pyo3Docker) -> Result<SystemVersion, docker_api::Error> {
    get_runtime().block_on(docker.0.version())
}

fn __info(docker: Pyo3Docker) -> Result<SystemInfo, docker_api::Error> {
    get_runtime().block_on(docker.0.info())
}

fn __ping(docker: Pyo3Docker) -> Result<PingInfo, docker_api::Error> {
    get_runtime().block_on(docker.0.ping())
}

fn __data_usage(docker: Pyo3Docker) -> Result<SystemDataUsage200Response, docker_api::Error> {
    get_runtime().block_on(docker.0.data_usage())
}

/// Register all docker-pyo3 classes and modules into a PyModule.
/// This function can be used by other Rust crates to re-export docker-pyo3
/// functionality under their own namespace.
///
/// # Example
/// ```rust,ignore
/// use pyo3::prelude::*;
/// use docker_pyo3::register_module;
///
/// #[pymodule]
/// fn my_module(py: Python, m: &PyModule) -> PyResult<()> {
///     // Re-export docker-pyo3 under your module
///     docker_pyo3::register_module(py, m, "my_module")?;
///     
///     // Add your own additional classes/functions
///     // m.add_class::<MyClass>()?;
///     
///     Ok(())
/// }
/// ```
pub fn register_module(py: Python, m: &PyModule, module_name: &str) -> PyResult<()> {
    // Add main classes
    m.add_class::<Pyo3Docker>()?;
    m.add_class::<Pyo3Stack>()?;
    m.add_class::<Service>()?;

    // Add submodules
    m.add_wrapped(wrap_pymodule!(image::image))?;
    m.add_wrapped(wrap_pymodule!(container::container))?;
    m.add_wrapped(wrap_pymodule!(network::network))?;
    m.add_wrapped(wrap_pymodule!(volume::volume))?;

    // Register submodules in sys.modules with the provided module name
    let sys = PyModule::import(py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item(format!("{}.image", module_name), m.getattr("image")?)?;
    sys_modules.set_item(format!("{}.container", module_name), m.getattr("container")?)?;
    sys_modules.set_item(format!("{}.network", module_name), m.getattr("network")?)?;
    sys_modules.set_item(format!("{}.volume", module_name), m.getattr("volume")?)?;

    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
pub fn docker_pyo3(py: Python, m: &PyModule) -> PyResult<()> {
    register_module(py, m, "docker_pyo3")
}
