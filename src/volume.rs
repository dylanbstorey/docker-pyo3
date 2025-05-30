use docker_api::{
    models::VolumeList200Response,
    models::VolumePrune200Response,
    opts::{VolumeCreateOpts, VolumeListOpts, VolumePruneOpts},
    Volume, Volumes,
};
use pyo3::prelude::*;

use crate::{get_runtime, Pyo3Docker};
use crate::error::DockerPyo3Error;
use pyo3::exceptions;
use pyo3::types::PyDict;
use pythonize::pythonize;
use std::collections::HashMap;

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
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __volumes_list(&self.0, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    pub fn create(
        &self,
        py: Python,
        name: Option<&str>,
        driver: Option<&str>,
        driver_opts: Option<&PyDict>,
        labels: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        let mut opts = VolumeCreateOpts::builder();
        
        let driver_opts: Option<HashMap<&str, &str>> = if driver_opts.is_some() {
            Some(driver_opts.unwrap().extract().unwrap())
        } else {
            None
        };

        let labels: Option<HashMap<&str, &str>> = if labels.is_some() {
            Some(labels.unwrap().extract().unwrap())
        } else {
            None
        };

        bo_setter!(name, opts);
        bo_setter!(driver, opts);
        bo_setter!(driver_opts, opts);
        bo_setter!(labels, opts);

        let rv = __volumes_create(&self.0, &opts.build());

        match rv {
            Ok(volume_response) => {
                // Extract the volume name from the response and return a Pyo3Volume object
                let name = &volume_response.name;
                let volume_obj = Pyo3Volume(self.0.get(name));
                let py_obj = Py::new(py, volume_obj)?;
                Ok(py_obj.into_py(py))
            },
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }
}

fn __volumes_prune(
    volumes: &Volumes,
    opts: &VolumePruneOpts,
) -> Result<VolumePrune200Response, docker_api::Error> {
    get_runtime().block_on(volumes.prune(opts))
}

fn __volumes_list(
    volumes: &Volumes,
    opts: &VolumeListOpts,
) -> Result<VolumeList200Response, docker_api::Error> {
    get_runtime().block_on(volumes.list(opts))
}

fn __volumes_create(
    volumes: &Volumes,
    opts: &VolumeCreateOpts,
) -> Result<docker_api::models::Volume, docker_api::Error> {
    get_runtime().block_on(volumes.create(opts))
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
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    pub fn delete(&self) -> PyResult<()> {
        let rv = __volume_delete(&self.0);

        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }
}

fn __volume_inspect(
    volume: &Volume,
) -> Result<docker_api::models::Volume, docker_api::Error> {
    get_runtime().block_on(volume.inspect())
}

fn __volume_delete(volume: &Volume) -> Result<(), docker_api::Error> {
    get_runtime().block_on(volume.delete())
}
