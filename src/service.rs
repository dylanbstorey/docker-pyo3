use crate::Pyo3Docker;
use chrono::{DateTime, Utc};
use docker_api::opts::{LogsOpts, ServiceListOpts};
use docker_api::{Service, Services};
use futures_util::stream::StreamExt;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDateTime;
use pythonize::pythonize;

#[pymodule]
pub fn service(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Services>()?;
    m.add_class::<Pyo3Service>()?;
    Ok(())
}

/// Interface for managing Docker Swarm services collection.
///
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Services")]
pub struct Pyo3Services {
    services: Services,
}

/// Represents an individual Docker Swarm service.
///
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Service")]
pub struct Pyo3Service {
    service: Service,
}

#[pymethods]
impl Pyo3Services {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Services {
            services: Services::new(docker.0),
        }
    }

    /// Get a specific service by ID or name.
    ///
    /// Args:
    ///     id: Service ID or name
    ///
    /// Returns:
    ///     Service: Service instance
    pub fn get(&self, id: &str) -> Pyo3Service {
        Pyo3Service {
            service: self.services.get(id),
        }
    }

    /// List all services in the swarm.
    ///
    /// Returns:
    ///     list[dict]: List of service information dictionaries
    ///
    /// Raises:
    ///     SystemError: If the operation fails (e.g., swarm not initialized)
    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __services_list(&self.services, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __services_list(
    services: &Services,
    opts: &ServiceListOpts,
) -> Result<Vec<docker_api::models::Service>, docker_api::Error> {
    services.list(opts).await
}

#[pymethods]
impl Pyo3Service {
    #[new]
    pub fn new(docker: Pyo3Docker, id: &str) -> Self {
        Pyo3Service {
            service: Service::new(docker.0, id),
        }
    }

    /// Get the service ID.
    ///
    /// Returns:
    ///     str: Service ID
    pub fn id(&self) -> String {
        self.service.name().to_string()
    }

    /// Inspect the service to get detailed information.
    ///
    /// Returns:
    ///     dict: Detailed service information including spec, endpoint, update status, etc.
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __service_inspect(&self.service);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Delete the service from the swarm.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the service cannot be deleted
    pub fn delete(&self) -> PyResult<()> {
        let rv = __service_delete(&self.service);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Get service logs.
    ///
    /// Args:
    ///     stdout: Include stdout
    ///     stderr: Include stderr
    ///     timestamps: Include timestamps
    ///     n_lines: Number of lines to return from the end of logs
    ///     all: Return all logs
    ///     since: Only return logs since this datetime
    ///
    /// Returns:
    ///     str: Service logs
    #[pyo3(signature = (stdout=None, stderr=None, timestamps=None, n_lines=None, all=None, since=None))]
    pub fn logs(
        &self,
        stdout: Option<bool>,
        stderr: Option<bool>,
        timestamps: Option<bool>,
        n_lines: Option<usize>,
        all: Option<bool>,
        since: Option<&Bound<'_, PyDateTime>>,
    ) -> String {
        let mut log_opts = LogsOpts::builder();

        bo_setter!(stdout, log_opts);
        bo_setter!(stderr, log_opts);
        bo_setter!(timestamps, log_opts);
        bo_setter!(n_lines, log_opts);

        if all.is_some() && all.unwrap() {
            log_opts = log_opts.all();
        }

        if since.is_some() {
            let rs_since: DateTime<Utc> = since.unwrap().extract().unwrap();
            log_opts = log_opts.since(&rs_since);
        }

        __service_logs(&self.service, &log_opts.build())
    }
}

#[tokio::main]
async fn __service_inspect(
    service: &Service,
) -> Result<docker_api::models::Service, docker_api::Error> {
    service.inspect().await
}

#[tokio::main]
async fn __service_delete(service: &Service) -> Result<(), docker_api::Error> {
    service.delete().await
}

#[tokio::main]
async fn __service_logs(service: &Service, log_opts: &LogsOpts) -> String {
    let log_stream = service.logs(log_opts);

    let log = log_stream
        .map(|chunk| match chunk {
            Ok(chunk) => chunk.to_vec(),
            Err(e) => {
                eprintln!("Error: {e}");
                vec![]
            }
        })
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    format!("{}", String::from_utf8_lossy(&log))
}
