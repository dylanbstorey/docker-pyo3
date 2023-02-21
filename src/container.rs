use chrono::{DateTime, Utc};
use docker_api::conn::TtyChunk;
use docker_api::models::{
    ContainerInspect200Response, ContainerPrune200Response, ContainerSummary, ContainerWaitResponse,
};
use docker_api::opts::{
    ContainerCreateOpts, ContainerListOpts, ContainerPruneOpts, ExecCreateOpts, LogsOpts,
};
use docker_api::{Container, Containers};
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyDelta, PyDict, PyList};
use pythonize::pythonize;
use std::{fs::File, io::Read};
use tar::Archive;

use crate::Pyo3Docker;

#[pymodule]
pub fn container(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pyo3Containers>()?;
    m.add_class::<Pyo3Container>()?;
    Ok(())
}

#[derive(Debug)]
#[pyclass(name = "Containers")]
pub struct Pyo3Containers(pub Containers);

#[derive(Debug)]
#[pyclass(name = "Container")]
pub struct Pyo3Container(pub Container);

#[pymethods]
impl Pyo3Containers {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Containers(Containers::new(docker.0))
    }

    fn get(&self, id: &str) -> Pyo3Container {
        Pyo3Container(self.0.get(id))
    }

    fn list(
        &self,
        all: Option<bool>,
        since: Option<String>,
        before: Option<String>,
        sized: Option<bool>,
    ) -> Py<PyAny> {
        let mut builder = ContainerListOpts::builder();

        bo_setter!(all, builder);
        bo_setter!(since, builder);
        bo_setter!(before, builder);
        bo_setter!(sized, builder);

        let cs = __containers_list(&self.0, &builder.build());
        pythonize_this!(cs)
    }

    fn prune(&self) -> PyResult<Py<PyAny>> {
        let rv = __containers_prune(&self.0, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
    fn create(
        &self,
        image: &str,
        attach_stderr: Option<bool>,
        attach_stdin: Option<bool>,
        attach_stdout: Option<bool>,
        auto_remove: Option<bool>,
        _capabilities: Option<&PyList>,
        _command: Option<&PyList>,
        cpu_shares: Option<u32>,
        cpus: Option<f64>,
        _devices: Option<&PyList>,
        _entrypoint: Option<&PyList>,
        _env: Option<&PyList>,
        _expose: Option<&PyList>,
        _extra_hosts: Option<&PyList>,
        _labels: Option<&PyDict>,
        links: Option<&PyList>,
        log_driver: Option<&str>,
        memory: Option<u64>,
        memory_swap: Option<i64>,
        name: Option<&str>,
        nano_cpus: Option<u64>,
        network_mode: Option<&str>,
        privileged: Option<bool>,
        _publish: Option<&PyList>,
        _publish_all_ports: Option<bool>,
        _restart_policy: Option<&PyDict>, // name,maximum_retry_count,
        _security_options: Option<&PyList>,
        stop_signal: Option<&str>,
        stop_signal_num: Option<u64>,
        _stop_timeout: Option<&PyDelta>,
        tty: Option<bool>,
        user: Option<&str>,
        userns_mode: Option<&str>,
        _volumes: Option<&PyList>,
        _volumes_from: Option<&PyList>,
        working_dir: Option<&str>,
    ) -> PyResult<Pyo3Container> {
        let mut create_opts = ContainerCreateOpts::builder().image(image);

        let links: Option<Vec<&str>> = if links.is_some() {
            links.unwrap().extract().unwrap()
        } else {
            None
        };

        bo_setter!(attach_stderr, create_opts);
        bo_setter!(attach_stdin, create_opts);
        bo_setter!(attach_stdout, create_opts);
        bo_setter!(auto_remove, create_opts);
        bo_setter!(cpu_shares, create_opts);
        bo_setter!(cpus, create_opts);
        bo_setter!(log_driver, create_opts);
        bo_setter!(memory, create_opts);
        bo_setter!(memory_swap, create_opts);
        bo_setter!(name, create_opts);
        bo_setter!(nano_cpus, create_opts);
        bo_setter!(network_mode, create_opts);
        bo_setter!(privileged, create_opts);
        bo_setter!(stop_signal, create_opts);
        bo_setter!(stop_signal_num, create_opts);
        bo_setter!(tty, create_opts);
        bo_setter!(user, create_opts);
        bo_setter!(userns_mode, create_opts);
        bo_setter!(working_dir, create_opts);

        // this will suck

        // bo_setter!(devices, create_opts);

        bo_setter!(links, create_opts);

        // bo_setter!(publish_all_ports, create_opts);
        // bo_setter!(restart_policy, create_opts);
        // bo_setter!(security_options, create_opts);
        // bo_setter!(stop_timeout, create_opts);
        // bo_setter!(volumes, create_opts);
        // bo_setter!(volumes_from, create_opts);
        // bo_setter!(capabilities, create_opts);
        // bo_setter!(command, create_opts);
        // bo_setter!(entrypoint, create_opts);
        // bo_setter!(env, create_opts);
        // bo_setter!(expose, create_opts);
        // bo_setter!(extra_hosts, create_opts);
        // bo_setter!(labels, create_opts);

        let rv = __containers_create(&self.0, &create_opts.build());
        match rv {
            Ok(rv) => Ok(Pyo3Container(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __containers_list(
    containers: &Containers,
    opts: &ContainerListOpts,
) -> Vec<ContainerSummary> {
    let x = containers.list(opts).await;
    x.unwrap()
}

#[tokio::main]
async fn __containers_prune(
    containers: &Containers,
    opts: &ContainerPruneOpts,
) -> Result<ContainerPrune200Response, docker_api::Error> {
    containers.prune(opts).await
}

#[tokio::main]
async fn __containers_create(
    containers: &Containers,
    opts: &ContainerCreateOpts,
) -> Result<Container, docker_api::Error> {
    containers.create(opts).await
}

#[pymethods]
impl Pyo3Container {
    #[new]
    fn new(docker: Pyo3Docker, id: String) -> Self {
        Pyo3Container(Container::new(docker.0, id))
    }

    fn id(&self) -> String {
        self.0.id().to_string()
    }

    fn inspect(&self) -> Py<PyAny> {
        let ci = __container_inspect(&self.0);
        pythonize_this!(ci)
    }
    fn logs(
        &self,
        stdout: Option<bool>,
        stderr: Option<bool>,
        timestamps: Option<bool>,
        n_lines: Option<usize>,
        all: Option<bool>,
        since: Option<&PyDateTime>,
    ) -> String {
        let mut log_opts = LogsOpts::builder();

        bo_setter!(stdout, log_opts);
        bo_setter!(stderr, log_opts);
        bo_setter!(timestamps, log_opts);
        bo_setter!(n_lines, log_opts);

        if all.is_some() && all.unwrap() {
            // all needs to be called w/o a value
            log_opts = log_opts.all();
        }

        if since.is_some() {
            let rs_since: DateTime<Utc> = since.unwrap().extract().unwrap();
            log_opts = log_opts.since(&rs_since);
        }

        __container_logs(&self.0, &log_opts.build())
    }

    fn remove(&self) -> PyResult<()> {
        Err(exceptions::PyNotImplementedError::new_err(
            "This method is not available yet.",
        ))
    }

    fn delete(&self) -> PyResult<()> {
        let rv = __container_delete(&self.0);
        if rv.is_ok() {
            Ok(())
        } else {
            Err(exceptions::PySystemError::new_err(
                "Failed to delete container.",
            ))
        }
    }

    // fn top(&self) -> PyResult<()> {
    //     Err(exceptions::PyNotImplementedError::new_err(
    //         "This method is not available yet.",
    //     ))
    // }

    // fn export(&self, docker_path: &str, local_path: &str) -> PyResult<()> {
    //     let bytes = self.0.export();
    //     let mut archive = Archive::new(&bytes[..]);
    //     archive.unpack(local_path);

    //     Ok(())
    // }

    fn start(&self) -> PyResult<()> {
        let rv = __container_start(&self.0);

        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to start container",
            )),
        }
    }

    fn stop(&self, wait: Option<&PyDelta>) -> PyResult<()> {
        let wait: Option<std::time::Duration> = wait.map(|wait| {
            wait.extract::<chrono::Duration>()
                .unwrap()
                .to_std()
                .unwrap()
        });

        let rv = __container_stop(&self.0, wait);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to start container",
            )),
        }
    }

    fn restart(&self, wait: Option<&PyDelta>) -> PyResult<()> {
        let wait: Option<std::time::Duration> = wait.map(|wait| {
            wait.extract::<chrono::Duration>()
                .unwrap()
                .to_std()
                .unwrap()
        });

        let rv = __container_restart(&self.0, wait);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to stop container",
            )),
        }
    }

    fn kill(&self, signal: Option<&str>) -> PyResult<()> {
        let rv = __container_kill(&self.0, signal);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to kill container",
            )),
        }
    }

    fn rename(&self, name: &str) -> PyResult<()> {
        let rv = __container_rename(&self.0, name);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to rename container",
            )),
        }
    }

    fn pause(&self) -> PyResult<()> {
        let rv = __container_pause(&self.0);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to pause container",
            )),
        }
    }

    fn unpause(&self) -> PyResult<()> {
        let rv = __container_unpause(&self.0);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to unpause container",
            )),
        }
    }

    fn wait(&self) -> Py<PyAny> {
        let rv = __container_wait(&self.0).unwrap();
        pythonize_this!(rv)
    }

    fn exec(
        &self,
        command: &PyList,
        env: Option<&PyList>,
        attach_stdout: Option<bool>,
        attach_stderr: Option<bool>,
        // detach_keys: Option<&str>,
        // tty: Option<bool>,
        privileged: Option<bool>,
        user: Option<&str>,
        working_dir: Option<&str>,
    ) -> PyResult<()> {
        let command: Vec<&str> = command.extract().unwrap();
        let mut exec_opts = ExecCreateOpts::builder().command(command);

        if env.is_some() {
            let env: Vec<&str> = env.unwrap().extract().unwrap();
            exec_opts = exec_opts.env(env);
        }

        bo_setter!(attach_stdout, exec_opts);
        bo_setter!(attach_stderr, exec_opts);
        // bo_setter!(tty, exec_opts);
        // bo_setter!(detach_keys,exec_opts);
        bo_setter!(privileged, exec_opts);
        bo_setter!(user, exec_opts);
        bo_setter!(working_dir, exec_opts);

        let rv = __container_exec(&self.0, exec_opts.build());
        let rv = rv.unwrap();
        match rv {
            Ok(_rv) => Ok(()),
            Err(rv) => Err(exceptions::PySystemError::new_err(format!(
                "Failed to exec container {rv}"
            ))),
        }
    }

    fn copy_from(&self, src: &str, dst: &str) -> PyResult<()> {
        let rv = __container_copy_from(&self.0, src);

        match rv {
            Ok(rv) => {
                let mut archive = Archive::new(&rv[..]);
                let r = archive.unpack(dst);
                match r {
                    Ok(_r) => Ok(()),
                    Err(r) => Err(exceptions::PySystemError::new_err(format!("{r}"))),
                }
            }
            Err(rv) => Err(exceptions::PySystemError::new_err(format!("{rv}"))),
        }
    }

    fn copy_file_into(&self, src: &str, dst: &str) -> PyResult<()> {
        let mut file = File::open(src).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .expect("Cannot read file on the localhost.");

        let rv = __container_copy_file_into(&self.0, dst, &bytes);

        match rv {
            Ok(_rv) => Ok(()),
            Err(rv) => Err(exceptions::PySystemError::new_err(format!("{rv}"))),
        }
    }

    fn stat_file(&self, path: &str) -> Py<PyAny> {
        let rv = __container_stat_file(&self.0, path).unwrap();
        pythonize_this!(rv)
    }

    fn commit(&self) -> PyResult<()> {
        Err(exceptions::PyNotImplementedError::new_err(
            "This method is not available yet.",
        ))
    }

    fn __repr__(&self) -> String {
        let inspect = __container_inspect(&self.0);
        format!(
            "Container(id: {}, name: {}, status: {})",
            inspect.id.unwrap(),
            inspect.name.unwrap(),
            inspect.state.unwrap().status.unwrap()
        )
    }

    fn __string__(&self) -> String {
        self.__repr__()
    }
}

#[tokio::main]
async fn __container_inspect(container: &Container) -> ContainerInspect200Response {
    let c = container.inspect().await;
    c.unwrap()
}

#[tokio::main]
async fn __container_logs(container: &Container, log_opts: &LogsOpts) -> String {
    let log_stream = container.logs(log_opts);

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

#[tokio::main]
async fn __container_delete(container: &Container) -> Result<String, docker_api::Error> {
    container.delete().await
}

#[tokio::main]
async fn __container_start(container: &Container) -> Result<(), docker_api::Error> {
    container.start().await
}

#[tokio::main]
async fn __container_stop(
    container: &Container,
    wait: Option<std::time::Duration>,
) -> Result<(), docker_api::Error> {
    container.stop(wait).await
}

#[tokio::main]
async fn __container_restart(
    container: &Container,
    wait: Option<std::time::Duration>,
) -> Result<(), docker_api::Error> {
    container.restart(wait).await
}

#[tokio::main]
async fn __container_kill(
    container: &Container,
    signal: Option<&str>,
) -> Result<(), docker_api::Error> {
    container.kill(signal).await
}

#[tokio::main]
async fn __container_rename(container: &Container, name: &str) -> Result<(), docker_api::Error> {
    container.rename(name).await
}

#[tokio::main]
async fn __container_pause(container: &Container) -> Result<(), docker_api::Error> {
    container.pause().await
}

#[tokio::main]
async fn __container_unpause(container: &Container) -> Result<(), docker_api::Error> {
    container.unpause().await
}

#[tokio::main]
async fn __container_wait(
    container: &Container,
) -> Result<ContainerWaitResponse, docker_api::Error> {
    container.wait().await
}

#[tokio::main]
async fn __container_exec(
    container: &Container,
    exec_opts: ExecCreateOpts,
) -> Option<Result<TtyChunk, docker_api::conn::Error>> {
    container.exec(&exec_opts).next().await
}

#[tokio::main]
async fn __container_copy_from(
    container: &Container,
    path: &str,
) -> Result<Vec<u8>, docker_api::Error> {
    container.copy_from(path).try_concat().await
}

#[tokio::main]
async fn __container_copy_file_into(
    container: &Container,
    dst: &str,
    bytes: &Vec<u8>,
) -> Result<(), docker_api::Error> {
    container.copy_file_into(dst, bytes).await
}

#[tokio::main]
async fn __container_stat_file(
    container: &Container,
    src: &str,
) -> Result<String, docker_api::Error> {
    container.stat_file(src).await
}
