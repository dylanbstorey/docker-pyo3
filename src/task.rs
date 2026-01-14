use crate::Pyo3Docker;
use chrono::{DateTime, Utc};
use docker_api::opts::LogsOpts;
use docker_api::task::TaskListOpts;
use docker_api::{Task, Tasks};
use futures_util::stream::StreamExt;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDateTime;
use pythonize::pythonize;

#[pymodule]
pub fn task(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Tasks>()?;
    m.add_class::<Pyo3Task>()?;
    Ok(())
}

/// Interface for managing Docker Swarm tasks collection.
///
/// Tasks are individual units of work running on swarm nodes as part of a service.
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Tasks")]
pub struct Pyo3Tasks {
    tasks: Tasks,
}

/// Represents an individual Docker Swarm task.
///
/// A task is a container instance running as part of a swarm service.
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Task")]
pub struct Pyo3Task {
    task: Task,
}

#[pymethods]
impl Pyo3Tasks {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Tasks {
            tasks: Tasks::new(docker.0),
        }
    }

    /// Get a specific task by ID.
    ///
    /// Args:
    ///     id: Task ID
    ///
    /// Returns:
    ///     Task: Task instance
    pub fn get(&self, id: &str) -> Pyo3Task {
        Pyo3Task {
            task: self.tasks.get(id),
        }
    }

    /// List all tasks in the swarm.
    ///
    /// Returns:
    ///     list[dict]: List of task information dictionaries
    ///
    /// Raises:
    ///     SystemError: If the operation fails (e.g., swarm not initialized)
    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __tasks_list(&self.tasks, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __tasks_list(
    tasks: &Tasks,
    opts: &TaskListOpts,
) -> Result<Vec<docker_api::models::Task>, docker_api::Error> {
    tasks.list(opts).await
}

#[pymethods]
impl Pyo3Task {
    #[new]
    pub fn new(docker: Pyo3Docker, id: &str) -> Self {
        Pyo3Task {
            task: Task::new(docker.0, id),
        }
    }

    /// Get the task ID.
    ///
    /// Returns:
    ///     str: Task ID
    pub fn id(&self) -> String {
        self.task.id().to_string()
    }

    /// Inspect the task to get detailed information.
    ///
    /// Returns:
    ///     dict: Detailed task information including status, spec, assigned node, etc.
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __task_inspect(&self.task);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Get task logs.
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
    ///     str: Task logs
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

        __task_logs(&self.task, &log_opts.build())
    }
}

#[tokio::main]
async fn __task_inspect(task: &Task) -> Result<docker_api::models::Task, docker_api::Error> {
    task.inspect().await
}

#[tokio::main]
async fn __task_logs(task: &Task, log_opts: &LogsOpts) -> String {
    let log_stream = task.logs(log_opts);

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
