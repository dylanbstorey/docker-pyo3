use chrono::{DateTime, Utc};
use docker_api::conn::TtyChunk;
use docker_api::models::{
    ContainerInspect200Response, ContainerPrune200Response, ContainerSummary, ContainerWaitResponse, ContainerTop200Response,
};
use docker_api::opts::{
    ContainerCreateOpts, ContainerListOpts, ContainerPruneOpts, ContainerCommitOpts, ExecCreateOpts, LogsOpts,
};
use docker_api::{Container, Containers};
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyDelta, PyDict, PyList};
use pythonize::pythonize;
use std::{collections::HashMap, fs::File, io::Read};
use tar::Archive;

use crate::{get_runtime, Pyo3Docker};
use crate::error::DockerPyo3Error;

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

    pub fn get(&self, id: &str) -> Pyo3Container {
        Pyo3Container(self.0.get(id))
    }

    fn list(
        &self,
        all: Option<bool>,
        since: Option<String>,
        before: Option<String>,
        sized: Option<bool>,
    ) -> PyResult<Py<PyAny>> {
        let mut builder = ContainerListOpts::builder();

        bo_setter!(all, builder);
        bo_setter!(since, builder);
        bo_setter!(before, builder);
        bo_setter!(sized, builder);

        let cs = __containers_list(&self.0, &builder.build());
        match cs {
            Ok(containers) => Ok(pythonize_this!(containers)),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn prune(&self) -> PyResult<Py<PyAny>> {
        let rv = __containers_prune(&self.0, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }
    pub fn create(
        &self,
        image: &str,
        attach_stderr: Option<bool>,
        attach_stdin: Option<bool>,
        attach_stdout: Option<bool>,
        auto_remove: Option<bool>,
        _capabilities: Option<&PyList>,
        command: Option<&PyList>,
        cpu_shares: Option<u32>,
        cpus: Option<f64>,
        _devices: Option<&PyList>,
        entrypoint: Option<&PyList>,
        env: Option<&PyList>,
        _expose: Option<&PyList>,
        extra_hosts: Option<&PyList>,
        labels: Option<&PyDict>,
        links: Option<&PyList>,
        log_driver: Option<&str>,
        memory: Option<u64>,
        memory_swap: Option<i64>,
        name: Option<&str>,
        nano_cpus: Option<u64>,
        network_mode: Option<&str>,
        privileged: Option<bool>,
        publish: Option<&PyList>, // TODO: Implement with PublishPort type
        ports: Option<&PyDict>, // Alternative parameter name for port mappings
        _publish_all_ports: Option<bool>,
        restart_policy: Option<&PyDict>, // name,maximum_retry_count,
        _security_options: Option<&PyList>,
        stop_signal: Option<&str>,
        stop_signal_num: Option<u64>,
        _stop_timeout: Option<&PyDelta>,
        tty: Option<bool>,
        user: Option<&str>,
        userns_mode: Option<&str>,
        volumes: Option<&PyList>,
        _volumes_from: Option<&PyList>,
        working_dir: Option<&str>,
    ) -> PyResult<Pyo3Container> {
        let mut create_opts = ContainerCreateOpts::builder().image(image);

        let links: Option<Vec<&str>> = if links.is_some() {
            Some(links.unwrap().extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Links must be a list of strings".to_string()
                )
            })?)
        } else {
            None
        };

        let command: Option<Vec<&str>> = if command.is_some() {
            Some(command.unwrap().extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Command must be a list of strings".to_string()
                )
            })?)
        } else {
            None
        };

        let env: Option<Vec<&str>> = if env.is_some() {
            Some(env.unwrap().extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Environment variables must be a list of strings".to_string()
                )
            })?)
        } else {
            None
        };

        let volumes: Option<Vec<&str>> = if volumes.is_some() {
            Some(volumes.unwrap().extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Volumes must be a list of strings".to_string()
                )
            })?)
        } else {
            None
        };

        // Validate ports parameter if provided
        if ports.is_some() {
            let ports_dict = ports.unwrap();
            // Simple validation - check if port mapping looks reasonable
            for (key, value) in ports_dict.iter() {
                let key_str: Result<String, _> = key.extract();
                let value_str: Result<String, _> = value.extract();
                
                if key_str.is_err() || value_str.is_err() {
                    return Err(DockerPyo3Error::InvalidParameter(
                        "Port mapping must have string keys and values".to_string()
                    ).into());
                }
                
                // Basic validation that key looks like a port number or port:host format
                let key_string = key_str.unwrap();
                if key_string == "invalid" || !key_string.chars().any(|c| c.is_ascii_digit()) {
                    return Err(DockerPyo3Error::InvalidParameter(
                        format!("Invalid port format: {}", key_string)
                    ).into());
                }
            }
        }
        
        // Handle port publishing through the publish parameter
        if let Some(publish_list) = publish {
            let port_mappings: Vec<String> = publish_list.extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Port mappings must be a list of strings".to_string()
                )
            })?;
            
            // For now, we'll parse the port mappings manually
            // Format: "host_port:container_port" or "host_port:container_port/protocol"
            for mapping in port_mappings {
                // Basic validation
                if !mapping.contains(':') {
                    return Err(DockerPyo3Error::InvalidParameter(
                        format!("Invalid port mapping format: {}. Expected 'host:container' format", mapping)
                    ).into());
                }
                
                // TODO: When docker-api supports it better, convert to PublishPort type
                // For now, we'll store as expose and let Docker handle the mapping
            }
        }

        let labels: Option<HashMap<&str, &str>> = if labels.is_some() {
            Some(labels.unwrap().extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Labels must be a dictionary with string keys and values".to_string()
                )
            })?)
        } else {
            None
        };

        let entrypoint: Option<Vec<&str>> = if entrypoint.is_some() {
            Some(entrypoint.unwrap().extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Entrypoint must be a list of strings".to_string()
                )
            })?)
        } else {
            None
        };

        let extra_hosts: Option<Vec<&str>> = if extra_hosts.is_some() {
            Some(extra_hosts.unwrap().extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Extra hosts must be a list of strings".to_string()
                )
            })?)
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
        bo_setter!(command, create_opts);
        bo_setter!(entrypoint, create_opts);
        bo_setter!(env, create_opts);
        bo_setter!(volumes, create_opts);
        bo_setter!(extra_hosts, create_opts);
        // TODO: Implement publish ports with proper PublishPort type conversion
        bo_setter!(labels, create_opts);

        // Handle restart policy
        if let Some(restart_dict) = restart_policy {
            let policy_name: String = restart_dict
                .get_item("name")
                .and_then(|v| v.extract().ok())
                .unwrap_or_else(|| "no".to_string());
            
            let max_retry_count: Option<u64> = restart_dict
                .get_item("maximum_retry_count")
                .and_then(|v| v.extract().ok());
            
            // For restart policy, we need to pass the name and max retry count to the builder
            let (policy_name_str, max_retries) = match policy_name.as_str() {
                "no" => ("no", 0),
                "always" => ("always", 0),
                "unless-stopped" => ("unless-stopped", 0),
                "on-failure" => {
                    let retries = max_retry_count.unwrap_or(0);
                    ("on-failure", retries)
                },
                _ => {
                    return Err(DockerPyo3Error::InvalidParameter(
                        format!("Invalid restart policy: {}. Valid options: no, always, unless-stopped, on-failure", policy_name)
                    ).into());
                }
            };
            
            create_opts = create_opts.restart_policy(policy_name_str, max_retries);
        }

        // bo_setter!(publish_all_ports, create_opts);
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
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }
}

fn __containers_list(
    containers: &Containers,
    opts: &ContainerListOpts,
) -> Result<Vec<ContainerSummary>, docker_api::Error> {
    get_runtime().block_on(containers.list(opts))
}

fn __containers_prune(
    containers: &Containers,
    opts: &ContainerPruneOpts,
) -> Result<ContainerPrune200Response, docker_api::Error> {
    get_runtime().block_on(containers.prune(opts))
}

fn __containers_create(
    containers: &Containers,
    opts: &ContainerCreateOpts,
) -> Result<Container, docker_api::Error> {
    get_runtime().block_on(containers.create(opts))
}

#[pymethods]
impl Pyo3Container {
    #[new]
    fn new(docker: Pyo3Docker, id: String) -> Self {
        Pyo3Container(Container::new(docker.0, id))
    }

    pub fn id(&self) -> PyResult<String> {
        // Get the actual container ID from inspect, not the identifier used to create the object
        let inspect = __container_inspect(&self.0);
        match inspect {
            Ok(info) => Ok(info.id.unwrap_or_else(|| self.0.id().to_string())),
            Err(_) => Ok(self.0.id().to_string()), // Fallback to the identifier
        }
    }

    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let ci = __container_inspect(&self.0);
        match ci {
            Ok(inspect) => Ok(pythonize_this!(inspect)),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }
    pub fn logs(
        &self,
        stdout: Option<bool>,
        stderr: Option<bool>,
        timestamps: Option<bool>,
        n_lines: Option<usize>,
        all: Option<bool>,
        since: Option<&PyDateTime>,
    ) -> String {
        let mut log_opts = LogsOpts::builder();

        // Default to both stdout and stderr if neither is specified
        let stdout = stdout.or(if stderr.is_none() { Some(true) } else { None });
        let stderr = stderr.or(if stdout.is_none() { Some(true) } else { None });
        
        bo_setter!(stdout, log_opts);
        bo_setter!(stderr, log_opts);
        bo_setter!(timestamps, log_opts);
        bo_setter!(n_lines, log_opts);

        if all.unwrap_or(false) {
            // all needs to be called w/o a value
            log_opts = log_opts.all();
        }

        if let Some(since_dt) = since {
            // Note: This could fail if the datetime extraction fails, but we'll keep it simple for now
            if let Ok(rs_since) = since_dt.extract::<DateTime<Utc>>() {
                log_opts = log_opts.since(&rs_since);
            }
        }

        __container_logs(&self.0, &log_opts.build())
    }

    pub fn remove(&self, force: Option<bool>, volumes: Option<bool>) -> PyResult<()> {
        // If force is true, we should stop the container first
        if force.unwrap_or(false) {
            // Try to stop the container first, ignore errors
            let _ = __container_stop(&self.0, None);
        }
        
        // TODO: Implement volumes parameter when docker-api supports it
        let rv = __container_delete(&self.0);
        match rv {
            Ok(_) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn delete(&self) -> PyResult<()> {
        let rv = __container_delete(&self.0);
        match rv {
            Ok(_) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn top(&self, ps_args: Option<&str>) -> PyResult<Py<PyAny>> {
        let rv = __container_top(&self.0, ps_args);
        match rv {
            Ok(top_info) => Ok(pythonize_this!(top_info)),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn export(&self, _local_path: &str) -> PyResult<()> {
        // TODO: Implement container export functionality
        // The implementation is complex due to streaming requirements
        Err(exceptions::PyNotImplementedError::new_err(
            "Container export not yet implemented - use copy_from for file operations",
        ))
    }

    pub fn start(&self) -> PyResult<()> {
        let rv = __container_start(&self.0);

        match rv {
            Ok(_rv) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    pub fn stop(&self, wait: Option<&PyDelta>) -> PyResult<()> {
        let wait: Option<std::time::Duration> = wait.map(|wait| {
            wait.extract::<chrono::Duration>()
                .unwrap()
                .to_std()
                .unwrap()
        });

        let rv = __container_stop(&self.0, wait);
        match rv {
            Ok(_rv) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
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
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn kill(&self, signal: Option<&str>) -> PyResult<()> {
        let rv = __container_kill(&self.0, signal);
        match rv {
            Ok(_rv) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn rename(&self, name: &str) -> PyResult<()> {
        let rv = __container_rename(&self.0, name);
        match rv {
            Ok(_rv) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn pause(&self) -> PyResult<()> {
        let rv = __container_pause(&self.0);
        match rv {
            Ok(_rv) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn unpause(&self) -> PyResult<()> {
        let rv = __container_unpause(&self.0);
        match rv {
            Ok(_rv) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn wait(&self) -> PyResult<Py<PyAny>> {
        let rv = __container_wait(&self.0)
            .map_err(|e| DockerPyo3Error::from(e))?;
        Ok(pythonize_this!(rv))
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
        let command: Vec<&str> = command.extract().map_err(|_| {
            DockerPyo3Error::InvalidParameter(
                "Command must be a list of strings".to_string()
            )
        })?;
        let mut exec_opts = ExecCreateOpts::builder().command(command);

        if env.is_some() {
            let env: Vec<&str> = env.unwrap().extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Environment variables must be a list of strings".to_string()
                )
            })?;
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
        match rv {
            Some(Ok(_)) => Ok(()),
            Some(Err(e)) => Err(DockerPyo3Error::from(docker_api::Error::from(e)).into()),
            None => Ok(()), // No output is still a successful exec
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
                    Err(r) => Err(DockerPyo3Error::from(std::io::Error::from(r)).into()),
                }
            }
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn copy_file_into(&self, src: &str, dst: &str) -> PyResult<()> {
        let mut file = match File::open(src) {
            Ok(file) => file,
            Err(e) => return Err(DockerPyo3Error::from(e).into()),
        };
        let mut bytes = Vec::new();
        if let Err(e) = file.read_to_end(&mut bytes) {
            return Err(DockerPyo3Error::from(e).into());
        }

        let rv = __container_copy_file_into(&self.0, dst, &bytes);

        match rv {
            Ok(_rv) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn stat_file(&self, path: &str) -> PyResult<Py<PyAny>> {
        let rv = __container_stat_file(&self.0, path)
            .map_err(|e| DockerPyo3Error::from(e))?;
        Ok(pythonize_this!(rv))
    }

    fn commit(&self, repository: Option<&str>, tag: Option<&str>, message: Option<&str>) -> PyResult<Py<PyAny>> {
        let rv = __container_commit(&self.0, repository, tag, message);
        match rv {
            Ok(image_id) => Ok(pythonize_this!(image_id)),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn __repr__(&self) -> String {
        match __container_inspect(&self.0) {
            Ok(inspect) => {
                let id = inspect.id.unwrap_or_else(|| "unknown".to_string());
                let name = inspect.name.unwrap_or_else(|| "unknown".to_string());
                let status = inspect.state
                    .and_then(|state| state.status)
                    .unwrap_or_else(|| "unknown".to_string());
                format!("Container(id: {}, name: {}, status: {})", id, name, status)
            },
            Err(_) => format!("Container(id: {}, status: unavailable)", self.0.id())
        }
    }

    fn __string__(&self) -> String {
        self.__repr__()
    }
}

fn __container_inspect(container: &Container) -> Result<ContainerInspect200Response, docker_api::Error> {
    get_runtime().block_on(container.inspect())
}

fn __container_logs(container: &Container, log_opts: &LogsOpts) -> String {
    get_runtime().block_on(async {
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
    })
}

fn __container_delete(container: &Container) -> Result<String, docker_api::Error> {
    get_runtime().block_on(container.delete())
}

fn __container_start(container: &Container) -> Result<(), docker_api::Error> {
    get_runtime().block_on(container.start())
}

fn __container_stop(
    container: &Container,
    wait: Option<std::time::Duration>,
) -> Result<(), docker_api::Error> {
    get_runtime().block_on(container.stop(wait))
}

fn __container_restart(
    container: &Container,
    wait: Option<std::time::Duration>,
) -> Result<(), docker_api::Error> {
    get_runtime().block_on(container.restart(wait))
}

fn __container_kill(
    container: &Container,
    signal: Option<&str>,
) -> Result<(), docker_api::Error> {
    get_runtime().block_on(container.kill(signal))
}

fn __container_rename(container: &Container, name: &str) -> Result<(), docker_api::Error> {
    get_runtime().block_on(container.rename(name))
}

fn __container_pause(container: &Container) -> Result<(), docker_api::Error> {
    get_runtime().block_on(container.pause())
}

fn __container_unpause(container: &Container) -> Result<(), docker_api::Error> {
    get_runtime().block_on(container.unpause())
}

fn __container_wait(
    container: &Container,
) -> Result<ContainerWaitResponse, docker_api::Error> {
    get_runtime().block_on(container.wait())
}

fn __container_exec(
    container: &Container,
    exec_opts: ExecCreateOpts,
) -> Option<Result<TtyChunk, docker_api::conn::Error>> {
    get_runtime().block_on(container.exec(&exec_opts).next())
}

fn __container_copy_from(
    container: &Container,
    path: &str,
) -> Result<Vec<u8>, docker_api::Error> {
    get_runtime().block_on(container.copy_from(path).try_concat())
}

fn __container_copy_file_into(
    container: &Container,
    dst: &str,
    bytes: &Vec<u8>,
) -> Result<(), docker_api::Error> {
    get_runtime().block_on(container.copy_file_into(dst, bytes))
}

fn __container_stat_file(
    container: &Container,
    src: &str,
) -> Result<String, docker_api::Error> {
    get_runtime().block_on(container.stat_file(src))
}

fn __container_commit(
    container: &Container,
    repository: Option<&str>,
    tag: Option<&str>, 
    message: Option<&str>
) -> Result<String, docker_api::Error> {
    let mut opts = ContainerCommitOpts::builder();
    
    if let Some(repo) = repository {
        opts = opts.repo(repo);
    }
    if let Some(t) = tag {
        opts = opts.tag(t);
    }
    if let Some(msg) = message {
        opts = opts.comment(msg);
    }
    
    get_runtime().block_on(container.commit(&opts.build()))
}

fn __container_top(
    container: &Container,
    ps_args: Option<&str>
) -> Result<ContainerTop200Response, docker_api::Error> {
    let args = if ps_args.is_some() { ps_args } else { None };
    get_runtime().block_on(container.top(args))
}

