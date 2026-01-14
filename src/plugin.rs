//! Docker Plugin management support.
//!
//! This module provides types and functions for managing Docker plugins.

use crate::Pyo3Docker;
use docker_api::opts::PluginListOpts;
use docker_api::plugin::{Plugin, Plugins};
use pyo3::exceptions;
use pyo3::prelude::*;
use pythonize::pythonize;

#[pymodule]
pub fn plugin(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Plugins>()?;
    m.add_class::<Pyo3Plugin>()?;
    Ok(())
}

/// Interface for managing Docker plugins collection.
///
/// Docker plugins extend the capabilities of the Docker daemon.
/// Swarm mode must be enabled for some plugin operations.
#[derive(Debug)]
#[pyclass(name = "Plugins")]
pub struct Pyo3Plugins {
    plugins: Plugins,
}

/// Represents an individual Docker plugin.
///
/// Provides methods to inspect, enable, disable, and manage plugins.
#[derive(Debug)]
#[pyclass(name = "Plugin")]
pub struct Pyo3Plugin {
    plugin: Plugin,
}

#[pymethods]
impl Pyo3Plugins {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Plugins {
            plugins: Plugins::new(docker.0),
        }
    }

    /// Get a specific plugin by name.
    ///
    /// Args:
    ///     name: Plugin name (e.g., "vieux/sshfs:latest")
    ///
    /// Returns:
    ///     Plugin: Plugin instance for further operations
    pub fn get(&self, name: &str) -> Pyo3Plugin {
        Pyo3Plugin {
            plugin: self.plugins.get(name),
        }
    }

    /// List all installed plugins.
    ///
    /// Returns:
    ///     list[dict]: List of plugin information dictionaries including:
    ///         - id: Plugin ID
    ///         - name: Plugin name
    ///         - enabled: Whether the plugin is enabled
    ///         - config: Plugin configuration
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __plugins_list(&self.plugins, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// List plugins filtered by capability.
    ///
    /// Args:
    ///     capability: Filter plugins by capability (e.g., "volumedriver", "networkdriver")
    ///
    /// Returns:
    ///     list[dict]: List of matching plugin information dictionaries
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn list_by_capability(&self, capability: &str) -> PyResult<Py<PyAny>> {
        let opts = PluginListOpts::builder()
            .filter([docker_api::opts::PluginFilter::Capability(
                capability.to_string(),
            )])
            .build();
        let rv = __plugins_list(&self.plugins, &opts);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// List only enabled plugins.
    ///
    /// Returns:
    ///     list[dict]: List of enabled plugin information dictionaries
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn list_enabled(&self) -> PyResult<Py<PyAny>> {
        let opts = PluginListOpts::builder()
            .filter([docker_api::opts::PluginFilter::Enable])
            .build();
        let rv = __plugins_list(&self.plugins, &opts);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// List only disabled plugins.
    ///
    /// Returns:
    ///     list[dict]: List of disabled plugin information dictionaries
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn list_disabled(&self) -> PyResult<Py<PyAny>> {
        let opts = PluginListOpts::builder()
            .filter([docker_api::opts::PluginFilter::Disable])
            .build();
        let rv = __plugins_list(&self.plugins, &opts);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __plugins_list(
    plugins: &Plugins,
    opts: &PluginListOpts,
) -> Result<Vec<docker_api::models::Plugin>, docker_api::Error> {
    plugins.list(opts).await
}

#[pymethods]
impl Pyo3Plugin {
    #[new]
    pub fn new(docker: Pyo3Docker, name: &str) -> Self {
        Pyo3Plugin {
            plugin: Plugin::new(docker.0, name),
        }
    }

    /// Get the plugin name.
    ///
    /// Returns:
    ///     str: Plugin name
    pub fn name(&self) -> String {
        self.plugin.name().to_string()
    }

    /// Inspect the plugin to get detailed information.
    ///
    /// Returns:
    ///     dict: Detailed plugin information including:
    ///         - id: Plugin ID
    ///         - name: Plugin name
    ///         - enabled: Whether the plugin is enabled
    ///         - settings: Plugin settings (mounts, env, args, devices)
    ///         - plugin_reference: Plugin reference
    ///         - config: Full plugin configuration
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __plugin_inspect(&self.plugin);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Enable the plugin.
    ///
    /// Enables a previously disabled plugin so it can be used by Docker.
    ///
    /// Args:
    ///     timeout: Timeout in seconds to wait for enable (optional)
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the plugin cannot be enabled
    #[pyo3(signature = (timeout=None))]
    pub fn enable(&self, timeout: Option<u64>) -> PyResult<()> {
        let rv = __plugin_enable(&self.plugin, timeout);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Disable the plugin.
    ///
    /// Disables a running plugin. The plugin must be disabled before it can be removed.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the plugin cannot be disabled
    pub fn disable(&self) -> PyResult<()> {
        let rv = __plugin_disable(&self.plugin);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Remove (delete) the plugin.
    ///
    /// Removes the plugin from Docker. The plugin must be disabled first.
    /// Use force_remove() to remove an enabled plugin.
    ///
    /// Returns:
    ///     dict: Information about the removed plugin
    ///
    /// Raises:
    ///     SystemError: If the plugin cannot be removed (e.g., still enabled)
    pub fn remove(&self) -> PyResult<Py<PyAny>> {
        let rv = __plugin_delete(&self.plugin);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Forcefully remove the plugin.
    ///
    /// Removes the plugin from Docker, even if it's currently enabled.
    ///
    /// Returns:
    ///     dict: Information about the removed plugin
    ///
    /// Raises:
    ///     SystemError: If the plugin cannot be removed
    pub fn force_remove(&self) -> PyResult<Py<PyAny>> {
        let rv = __plugin_force_delete(&self.plugin);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Push the plugin to a registry.
    ///
    /// Pushes the plugin to the registry specified in the plugin name.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the push fails
    pub fn push(&self) -> PyResult<()> {
        let rv = __plugin_push(&self.plugin);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Create a plugin from a tar archive.
    ///
    /// Creates a new plugin from a tar archive containing the plugin's
    /// rootfs directory and config.json manifest.
    ///
    /// Args:
    ///     path: Path to the tar archive containing plugin rootfs and manifest
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If plugin creation fails
    pub fn create(&self, path: &str) -> PyResult<()> {
        let rv = __plugin_create(&self.plugin, path);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __plugin_inspect(
    plugin: &Plugin,
) -> Result<docker_api::models::Plugin, docker_api::Error> {
    plugin.inspect().await
}

#[tokio::main]
async fn __plugin_enable(plugin: &Plugin, timeout: Option<u64>) -> Result<(), docker_api::Error> {
    plugin.enable(timeout).await
}

#[tokio::main]
async fn __plugin_disable(plugin: &Plugin) -> Result<(), docker_api::Error> {
    plugin.disable().await
}

#[tokio::main]
async fn __plugin_delete(plugin: &Plugin) -> Result<docker_api::models::Plugin, docker_api::Error> {
    plugin.delete().await
}

#[tokio::main]
async fn __plugin_force_delete(
    plugin: &Plugin,
) -> Result<docker_api::models::Plugin, docker_api::Error> {
    plugin.force_delete().await
}

#[tokio::main]
async fn __plugin_push(plugin: &Plugin) -> Result<(), docker_api::Error> {
    plugin.push().await
}

#[tokio::main]
async fn __plugin_create(plugin: &Plugin, path: &str) -> Result<(), docker_api::Error> {
    plugin.create(path).await
}
