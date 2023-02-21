use docker_api::{
    models::VolumeList200Response,
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
pub fn volume(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pyo3Volumes>()?;
    m.add_class::<Pyo3Volume>()?;
    Ok(())
}

#[derive(Debug)]
#[pyclass(name = "Volumes")]
pub struct Pyo3Volumes(pub Volumes);

#[derive(Debug)]
#[pyclass(name = "Volume")]
pub struct Pyo3Volume(pub Volume);

#[pymethods]
impl Pyo3Volumes {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Volumes(Volumes::new(docker.0))
    }

    pub fn get(&self, name: &str) -> Pyo3Volume {
        Pyo3Volume(self.0.get(name))
    }

    pub fn prune(&self) -> PyResult<Py<PyAny>> {
        let rv = __volumes_prune(&self.0, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __volumes_list(&self.0, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    pub fn create(
        &self,
        name: Option<&str>,
        driver: Option<&str>,
        _driver_opts: Option<&PyDict>,
        _labels: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        let mut opts = VolumeCreateOpts::builder();
        bo_setter!(name, opts);
        bo_setter!(driver, opts);
        // bo_setter!(driver_opts, opts);
        // bo_setter!(labels, opts);

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
) -> Result<VolumeList200Response, docker_api::Error> {
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

    pub fn name(&self) -> String {
        self.0.name().to_string()
    }

    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __volume_inspect(&self.0);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

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
