use std::collections::HashMap;

use crate::Pyo3Docker;
use docker_api::config::{ConfigCreateOpts, ConfigListOpts};
use docker_api::{Config, Configs};
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::pythonize;

#[pymodule]
pub fn config(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Configs>()?;
    m.add_class::<Pyo3Config>()?;
    Ok(())
}

/// Interface for managing Docker Swarm configs collection.
///
/// Configs are non-sensitive configuration data that can be mounted into containers.
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Configs")]
pub struct Pyo3Configs {
    configs: Configs,
}

/// Represents an individual Docker Swarm config.
///
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Config")]
pub struct Pyo3Config {
    config: Config,
}

#[pymethods]
impl Pyo3Configs {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Configs {
            configs: Configs::new(docker.0),
        }
    }

    /// Get a specific config by ID or name.
    ///
    /// Args:
    ///     id: Config ID or name
    ///
    /// Returns:
    ///     Config: Config instance
    pub fn get(&self, id: &str) -> Pyo3Config {
        Pyo3Config {
            config: self.configs.get(id),
        }
    }

    /// List all configs in the swarm.
    ///
    /// Returns:
    ///     list[dict]: List of config information dictionaries
    ///
    /// Raises:
    ///     SystemError: If the operation fails (e.g., swarm not initialized)
    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __configs_list(&self.configs, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Create a new config in the swarm.
    ///
    /// Args:
    ///     name: Name of the config
    ///     data: Config data (will be base64 encoded automatically)
    ///     labels: Labels as dict (e.g., {"app": "web"})
    ///
    /// Returns:
    ///     Config: Created config instance
    ///
    /// Raises:
    ///     SystemError: If the config cannot be created
    #[pyo3(signature = (name, data, labels=None))]
    pub fn create(
        &self,
        name: &str,
        data: &str,
        labels: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Pyo3Config> {
        let mut opts = ConfigCreateOpts::new(name, data);

        if let Some(labels_dict) = labels {
            let labels_map: HashMap<String, String> = labels_dict.extract().unwrap();
            for (k, v) in labels_map {
                opts = opts.add_label(k, v);
            }
        }

        let rv = __configs_create(&self.configs, &opts);

        match rv {
            Ok(config) => Ok(Pyo3Config { config }),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __configs_list(
    configs: &Configs,
    opts: &ConfigListOpts,
) -> Result<Vec<docker_api::models::Config>, docker_api::Error> {
    configs.list(opts).await
}

#[tokio::main]
async fn __configs_create(
    configs: &Configs,
    opts: &ConfigCreateOpts,
) -> Result<Config, docker_api::Error> {
    configs.create(opts).await
}

#[pymethods]
impl Pyo3Config {
    #[new]
    pub fn new(docker: Pyo3Docker, id: &str) -> Self {
        Pyo3Config {
            config: Config::new(docker.0, id),
        }
    }

    /// Get the config ID.
    ///
    /// Returns:
    ///     str: Config ID
    pub fn id(&self) -> String {
        self.config.name().to_string()
    }

    /// Inspect the config to get detailed information.
    ///
    /// Returns:
    ///     dict: Config metadata including ID, version, created/updated times, and labels
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __config_inspect(&self.config);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Delete the config from the swarm.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the config cannot be deleted
    pub fn delete(&self) -> PyResult<()> {
        let rv = __config_delete(&self.config);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __config_inspect(
    config: &Config,
) -> Result<docker_api::models::Config, docker_api::Error> {
    config.inspect().await
}

#[tokio::main]
async fn __config_delete(config: &Config) -> Result<(), docker_api::Error> {
    config.delete().await
}
