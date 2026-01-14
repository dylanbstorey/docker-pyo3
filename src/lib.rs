#[macro_use]
mod macros;
pub mod compose;
pub mod config;
pub mod container;
pub mod exec;
pub mod image;
pub mod network;
pub mod node;
pub mod plugin;
pub mod secret;
pub mod service;
pub mod task;
pub mod volume;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

use docker_api::models::{EventMessage, PingInfo, SystemInfo, SystemVersion};
use docker_api::opts::EventsOpts;
use docker_api::Docker;

use futures_util::stream::StreamExt;

use pythonize::pythonize;

use config::Pyo3Configs;
use container::Pyo3Containers;
use image::Pyo3Images;
use network::Pyo3Networks;
use node::Pyo3Nodes;
use plugin::Pyo3Plugins;
use secret::Pyo3Secrets;
use service::Pyo3Services;
use task::Pyo3Tasks;
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
    ///
    /// Note: Uses docker CLI to avoid Docker API v1.44+ VirtualSize compatibility issues.
    fn data_usage(&self) -> PyResult<Py<PyAny>> {
        let du = __data_usage_via_cli();
        match du {
            Ok(du) => Ok(pythonize_this!(du)),
            Err(e) => Err(pyo3::exceptions::PySystemError::new_err(e)),
        }
    }

    /// Get a stream of Docker events.
    ///
    /// Retrieves real-time events from the Docker daemon such as container starts,
    /// stops, image pulls, etc.
    ///
    /// Args:
    ///     limit: Maximum number of events to return (default: 100)
    ///
    /// Returns:
    ///     list[dict]: List of Docker events, each containing type, action, actor, time, etc.
    #[pyo3(signature = (limit=None))]
    fn events(&self, limit: Option<usize>) -> Py<PyAny> {
        let limit = limit.unwrap_or(100);
        let events = __events(self.clone(), limit);
        pythonize_this!(events)
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

    /// Get a Nodes interface for managing Swarm nodes.
    ///
    /// Swarm mode must be enabled for these operations to work.
    ///
    /// Returns:
    ///     Nodes: Interface for node operations
    fn nodes(&'_ self) -> Pyo3Nodes {
        Pyo3Nodes::new(self.clone())
    }

    /// Get a Services interface for managing Swarm services.
    ///
    /// Swarm mode must be enabled for these operations to work.
    ///
    /// Returns:
    ///     Services: Interface for service operations
    fn services(&'_ self) -> Pyo3Services {
        Pyo3Services::new(self.clone())
    }

    /// Get a Tasks interface for managing Swarm tasks.
    ///
    /// Tasks are individual units of work running on swarm nodes as part of a service.
    /// Swarm mode must be enabled for these operations to work.
    ///
    /// Returns:
    ///     Tasks: Interface for task operations
    fn tasks(&'_ self) -> Pyo3Tasks {
        Pyo3Tasks::new(self.clone())
    }

    /// Get a Secrets interface for managing Swarm secrets.
    ///
    /// Secrets are sensitive data that can be mounted into containers.
    /// Swarm mode must be enabled for these operations to work.
    ///
    /// Returns:
    ///     Secrets: Interface for secret operations
    fn secrets(&'_ self) -> Pyo3Secrets {
        Pyo3Secrets::new(self.clone())
    }

    /// Get a Configs interface for managing Swarm configs.
    ///
    /// Configs are non-sensitive configuration data that can be mounted into containers.
    /// Swarm mode must be enabled for these operations to work.
    ///
    /// Returns:
    ///     Configs: Interface for config operations
    fn configs(&'_ self) -> Pyo3Configs {
        Pyo3Configs::new(self.clone())
    }

    /// Get a Plugins interface for managing Docker plugins.
    ///
    /// Docker plugins extend the capabilities of the Docker daemon, providing
    /// additional volume drivers, network drivers, and other extensions.
    ///
    /// Returns:
    ///     Plugins: Interface for plugin operations
    fn plugins(&'_ self) -> Pyo3Plugins {
        Pyo3Plugins::new(self.clone())
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

/// Data usage item from docker CLI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DataUsageItem {
    #[serde(rename = "Type")]
    pub type_: String,
    pub total_count: String,
    pub active: String,
    pub size: String,
    pub reclaimable: String,
}

/// Data usage response compatible with Docker API v1.44+
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DataUsageCompat {
    pub images: Option<DataUsageItem>,
    pub containers: Option<DataUsageItem>,
    pub volumes: Option<DataUsageItem>,
    pub build_cache: Option<DataUsageItem>,
}

/// Get data usage via docker CLI to avoid VirtualSize issues.
fn __data_usage_via_cli() -> Result<DataUsageCompat, String> {
    use std::process::Command;

    let output = Command::new("docker")
        .args(["system", "df", "--format", "json"])
        .output()
        .map_err(|e| format!("Failed to execute docker: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "docker system df failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Docker outputs one JSON object per line (NDJSON format)
    let mut result = DataUsageCompat {
        images: None,
        containers: None,
        volumes: None,
        build_cache: None,
    };

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let item: DataUsageItem =
            serde_json::from_str(line).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        match item.type_.as_str() {
            "Images" => result.images = Some(item),
            "Containers" => result.containers = Some(item),
            "Local Volumes" => result.volumes = Some(item),
            "Build Cache" => result.build_cache = Some(item),
            _ => {}
        }
    }

    Ok(result)
}

#[tokio::main]
async fn __events(docker: Pyo3Docker, limit: usize) -> Vec<EventMessage> {
    let opts = EventsOpts::builder().build();
    let mut events_stream = docker.0.events(&opts);
    let mut events = Vec::new();

    // Use tokio timeout to avoid blocking indefinitely when no events
    let timeout_duration = std::time::Duration::from_secs(1);

    loop {
        let event_result = tokio::time::timeout(timeout_duration, events_stream.next()).await;
        match event_result {
            Ok(Some(Ok(event))) => {
                events.push(event);
                if events.len() >= limit {
                    break;
                }
            }
            Ok(Some(Err(_))) => break,
            Ok(None) => break,
            Err(_) => break, // Timeout - no more events available
        }
    }

    events
}

/// A Python module implemented in Rust.
#[pymodule]
pub fn docker_pyo3(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Docker>()?;

    m.add_wrapped(wrap_pymodule!(compose::compose))?;
    m.add_wrapped(wrap_pymodule!(config::config))?;
    m.add_wrapped(wrap_pymodule!(image::image))?;
    m.add_wrapped(wrap_pymodule!(container::container))?;
    m.add_wrapped(wrap_pymodule!(exec::exec))?;
    m.add_wrapped(wrap_pymodule!(network::network))?;
    m.add_wrapped(wrap_pymodule!(node::node))?;
    m.add_wrapped(wrap_pymodule!(plugin::plugin))?;
    m.add_wrapped(wrap_pymodule!(secret::secret))?;
    m.add_wrapped(wrap_pymodule!(service::service))?;
    m.add_wrapped(wrap_pymodule!(task::task))?;
    m.add_wrapped(wrap_pymodule!(volume::volume))?;

    let sys = PyModule::import(_py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.cast_into()?;
    sys_modules.set_item("docker_pyo3.compose", m.getattr("compose")?)?;
    sys_modules.set_item("docker_pyo3.config", m.getattr("config")?)?;
    sys_modules.set_item("docker_pyo3.image", m.getattr("image")?)?;
    sys_modules.set_item("docker_pyo3.container", m.getattr("container")?)?;
    sys_modules.set_item("docker_pyo3.exec", m.getattr("exec")?)?;
    sys_modules.set_item("docker_pyo3.network", m.getattr("network")?)?;
    sys_modules.set_item("docker_pyo3.node", m.getattr("node")?)?;
    sys_modules.set_item("docker_pyo3.plugin", m.getattr("plugin")?)?;
    sys_modules.set_item("docker_pyo3.secret", m.getattr("secret")?)?;
    sys_modules.set_item("docker_pyo3.service", m.getattr("service")?)?;
    sys_modules.set_item("docker_pyo3.task", m.getattr("task")?)?;
    sys_modules.set_item("docker_pyo3.volume", m.getattr("volume")?)?;

    Ok(())
}
