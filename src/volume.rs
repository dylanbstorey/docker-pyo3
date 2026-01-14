use std::collections::HashMap;

use docker_api::{
    models::VolumeListResponse,
    models::VolumePrune200Response,
    opts::{VolumeCreateOpts, VolumeListOpts, VolumePruneOpts},
    Volume, Volumes,
};
use pyo3::prelude::*;

use crate::Pyo3Docker;
use pyo3::exceptions;
use pyo3::types::PyDict;
use pythonize::pythonize;

#[pymodule]
pub fn volume(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Volumes>()?;
    m.add_class::<Pyo3Volume>()?;
    Ok(())
}

/// Interface for managing Docker volumes collection.
#[derive(Debug)]
#[pyclass(name = "Volumes")]
pub struct Pyo3Volumes(pub Volumes);

/// Represents an individual Docker volume.
#[derive(Debug)]
#[pyclass(name = "Volume")]
pub struct Pyo3Volume(pub Volume);

#[pymethods]
impl Pyo3Volumes {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Volumes(Volumes::new(docker.0))
    }

    /// Get a specific volume by name.
    ///
    /// Args:
    ///     name: Volume name
    ///
    /// Returns:
    ///     Volume: Volume instance
    pub fn get(&self, name: &str) -> Pyo3Volume {
        Pyo3Volume(self.0.get(name))
    }

    /// Remove unused volumes.
    ///
    /// Returns:
    ///     dict: Prune results including volumes deleted and space reclaimed
    pub fn prune(&self) -> PyResult<Py<PyAny>> {
        let rv = __volumes_prune(&self.0, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// List all volumes.
    ///
    /// Returns:
    ///     dict: Volume list information
    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __volumes_list(&self.0, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Create a new volume.
    ///
    /// Args:
    ///     name: Volume name
    ///     driver: Volume driver (e.g., "local")
    ///     driver_opts: Driver-specific options as dict
    ///     labels: Labels as dict (e.g., {"env": "prod"})
    ///
    /// Returns:
    ///     dict: Created volume information
    #[pyo3(signature = (name=None, driver=None, driver_opts=None, labels=None))]
    pub fn create(
        &self,
        name: Option<&str>,
        driver: Option<&str>,
        driver_opts: Option<&Bound<'_, PyDict>>,
        labels: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let mut opts = VolumeCreateOpts::builder();

        let driver_opts_map: Option<HashMap<String, String>> = if driver_opts.is_some() {
            Some(driver_opts.unwrap().extract().unwrap())
        } else {
            None
        };
        let driver_opts: Option<HashMap<&str, &str>> = driver_opts_map
            .as_ref()
            .map(|m| m.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect());

        let labels_map: Option<HashMap<String, String>> = if labels.is_some() {
            Some(labels.unwrap().extract().unwrap())
        } else {
            None
        };
        let labels: Option<HashMap<&str, &str>> = labels_map
            .as_ref()
            .map(|m| m.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect());

        bo_setter!(name, opts);
        bo_setter!(driver, opts);
        bo_setter!(driver_opts, opts);
        bo_setter!(labels, opts);

        let rv = __volumes_create(&self.0, &opts.build());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __volumes_prune(
    volumes: &Volumes,
    opts: &VolumePruneOpts,
) -> Result<VolumePrune200Response, docker_api::Error> {
    volumes.prune(opts).await
}

#[tokio::main]
async fn __volumes_list(
    volumes: &Volumes,
    opts: &VolumeListOpts,
) -> Result<VolumeListResponse, docker_api::Error> {
    volumes.list(opts).await
}

#[tokio::main]
async fn __volumes_create(
    volumes: &Volumes,
    opts: &VolumeCreateOpts,
) -> Result<docker_api::models::Volume, docker_api::Error> {
    volumes.create(opts).await
}

#[pymethods]
impl Pyo3Volume {
    #[new]
    pub fn new(docker: Pyo3Docker, name: &str) -> Self {
        Pyo3Volume(Volume::new(docker.0, name))
    }

    /// Get the volume name.
    ///
    /// Returns:
    ///     str: Volume name
    pub fn name(&self) -> String {
        self.0.name().to_string()
    }

    /// Inspect the volume to get detailed information.
    ///
    /// Returns:
    ///     dict: Detailed volume information including driver, mountpoint, etc.
    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __volume_inspect(&self.0);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Delete the volume.
    ///
    /// Returns:
    ///     None
    pub fn delete(&self) -> PyResult<()> {
        let rv = __volume_delete(&self.0);

        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __volume_inspect(
    volume: &Volume,
) -> Result<docker_api::models::Volume, docker_api::Error> {
    volume.inspect().await
}

#[tokio::main]
async fn __volume_delete(volume: &Volume) -> Result<(), docker_api::Error> {
    volume.delete().await
}
