use std::collections::HashMap;

use crate::Pyo3Docker;
use docker_api::secret::{SecretCreateOpts, SecretListOpts};
use docker_api::{Secret, Secrets};
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::pythonize;

#[pymodule]
pub fn secret(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Secrets>()?;
    m.add_class::<Pyo3Secret>()?;
    Ok(())
}

/// Interface for managing Docker Swarm secrets collection.
///
/// Secrets are sensitive data that can be mounted into containers.
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Secrets")]
pub struct Pyo3Secrets {
    secrets: Secrets,
}

/// Represents an individual Docker Swarm secret.
///
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Secret")]
pub struct Pyo3Secret {
    secret: Secret,
}

#[pymethods]
impl Pyo3Secrets {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Secrets {
            secrets: Secrets::new(docker.0),
        }
    }

    /// Get a specific secret by ID or name.
    ///
    /// Args:
    ///     id: Secret ID or name
    ///
    /// Returns:
    ///     Secret: Secret instance
    pub fn get(&self, id: &str) -> Pyo3Secret {
        Pyo3Secret {
            secret: self.secrets.get(id),
        }
    }

    /// List all secrets in the swarm.
    ///
    /// Returns:
    ///     list[dict]: List of secret information dictionaries
    ///
    /// Raises:
    ///     SystemError: If the operation fails (e.g., swarm not initialized)
    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __secrets_list(&self.secrets, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Create a new secret in the swarm.
    ///
    /// Args:
    ///     name: Name of the secret
    ///     data: Secret data (will be base64 encoded automatically)
    ///     labels: Labels as dict (e.g., {"app": "web"})
    ///
    /// Returns:
    ///     Secret: Created secret instance
    ///
    /// Raises:
    ///     SystemError: If the secret cannot be created
    #[pyo3(signature = (name, data, labels=None))]
    pub fn create(
        &self,
        name: &str,
        data: &str,
        labels: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Pyo3Secret> {
        let mut opts = SecretCreateOpts::new(name, data);

        if let Some(labels_dict) = labels {
            let labels_map: HashMap<String, String> = labels_dict.extract().unwrap();
            for (k, v) in labels_map {
                opts = opts.add_label(k, v);
            }
        }

        let rv = __secrets_create(&self.secrets, &opts);

        match rv {
            Ok(secret) => Ok(Pyo3Secret { secret }),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __secrets_list(
    secrets: &Secrets,
    opts: &SecretListOpts,
) -> Result<Vec<docker_api::models::Secret>, docker_api::Error> {
    secrets.list(opts).await
}

#[tokio::main]
async fn __secrets_create(
    secrets: &Secrets,
    opts: &SecretCreateOpts,
) -> Result<Secret, docker_api::Error> {
    secrets.create(opts).await
}

#[pymethods]
impl Pyo3Secret {
    #[new]
    pub fn new(docker: Pyo3Docker, id: &str) -> Self {
        Pyo3Secret {
            secret: Secret::new(docker.0, id),
        }
    }

    /// Get the secret ID.
    ///
    /// Returns:
    ///     str: Secret ID
    pub fn id(&self) -> String {
        self.secret.name().to_string()
    }

    /// Inspect the secret to get detailed information.
    ///
    /// Note: The secret data itself is not returned for security reasons.
    ///
    /// Returns:
    ///     dict: Secret metadata including ID, version, created/updated times, and labels
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __secret_inspect(&self.secret);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Delete the secret from the swarm.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the secret cannot be deleted
    pub fn delete(&self) -> PyResult<()> {
        let rv = __secret_delete(&self.secret);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __secret_inspect(
    secret: &Secret,
) -> Result<docker_api::models::Secret, docker_api::Error> {
    secret.inspect().await
}

#[tokio::main]
async fn __secret_delete(secret: &Secret) -> Result<(), docker_api::Error> {
    secret.delete().await
}
