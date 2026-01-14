use chrono::{DateTime, Utc};
use docker_api::conn::TtyChunk;
use docker_api::models::{
    ContainerChanges200Response, ContainerInspect200Response, ContainerPrune200Response,
    ContainerSummary, ContainerTop200Response, ContainerWaitResponse,
};
use docker_api::opts::{
    ContainerCommitOpts, ContainerCreateOpts, ContainerListOpts, ContainerPruneOpts,
    ContainerRestartOpts, ContainerStopOpts, ExecCreateOpts, ExecStartOpts, LogsOpts, PublishPort,
};
use docker_api::{Container, Containers, Exec};
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyDelta, PyDict, PyList};
use pythonize::pythonize;
use std::{collections::HashMap, fs::File, io::Read};
use tar::Archive;

use crate::Pyo3Docker;

#[pymodule]
pub fn container(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Containers>()?;
    m.add_class::<Pyo3Container>()?;
    Ok(())
}

/// Interface for managing Docker containers collection.
#[derive(Debug)]
#[pyclass(name = "Containers")]
pub struct Pyo3Containers {
    containers: Containers,
    docker: docker_api::Docker,
}

/// Represents an individual Docker container.
#[derive(Debug)]
#[pyclass(name = "Container")]
pub struct Pyo3Container {
    container: Container,
    docker: docker_api::Docker,
}

#[pymethods]
impl Pyo3Containers {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Containers {
            containers: Containers::new(docker.0.clone()),
            docker: docker.0,
        }
    }

    /// Get a specific container by ID or name.
    ///
    /// Args:
    ///     id: Container ID or name
    ///
    /// Returns:
    ///     Container: Container instance
    fn get(&self, id: &str) -> Pyo3Container {
        Pyo3Container {
            container: self.containers.get(id),
            docker: self.docker.clone(),
        }
    }

    /// List containers.
    ///
    /// Args:
    ///     all: Show all containers (default shows only running)
    ///     since: Show containers created since this container ID
    ///     before: Show containers created before this container ID
    ///     sized: Include size information
    ///
    /// Returns:
    ///     list[dict]: List of container information dictionaries
    #[pyo3(signature = (all=None, since=None, before=None, sized=None))]
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

        let cs = __containers_list(&self.containers, &builder.build());
        pythonize_this!(cs)
    }

    /// Remove stopped containers.
    ///
    /// Returns:
    ///     dict: Prune results including containers deleted and space reclaimed
    fn prune(&self) -> PyResult<Py<PyAny>> {
        let rv = __containers_prune(&self.containers, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Create a new container.
    ///
    /// Args:
    ///     image: Image name to use for the container
    ///     attach_stderr: Attach to stderr
    ///     attach_stdin: Attach to stdin
    ///     attach_stdout: Attach to stdout
    ///     auto_remove: Automatically remove the container when it exits
    ///     capabilities: List of Linux capabilities to add (e.g., ["NET_ADMIN", "SYS_TIME"])
    ///     command: Command to run as list (e.g., ["/bin/sh", "-c", "echo hello"])
    ///     cpu_shares: CPU shares (relative weight)
    ///     cpus: Number of CPUs
    ///     devices: List of device mappings, each a dict with PathOnHost, PathInContainer, CgroupPermissions
    ///     entrypoint: Entrypoint as list (e.g., ["/bin/sh"])
    ///     env: Environment variables as list (e.g., ["VAR=value"])
    ///     expose: List of port mappings to expose as dicts with srcport, hostport, protocol
    ///     extra_hosts: Extra host-to-IP mappings as list (e.g., ["hostname:192.168.1.1"])
    ///     labels: Labels as dict (e.g., {"app": "myapp", "env": "prod"})
    ///     links: Links to other containers as list
    ///     log_driver: Logging driver (e.g., "json-file", "syslog")
    ///     memory: Memory limit in bytes
    ///     memory_swap: Total memory limit (memory + swap)
    ///     name: Container name
    ///     nano_cpus: CPU quota in units of 10^-9 CPUs
    ///     network_mode: Network mode (e.g., "bridge", "host", "none")
    ///     privileged: Give extended privileges to this container
    ///     publish: List of ports to publish as dicts with port, protocol
    ///     publish_all_ports: Publish all exposed ports to random ports
    ///     restart_policy: Restart policy as dict with name and maximum_retry_count
    ///     security_options: Security options as list (e.g., ["label=user:USER"])
    ///     stop_signal: Signal to stop the container
    ///     stop_signal_num: Signal number to stop the container
    ///     stop_timeout: Timeout for stopping the container (timedelta)
    ///     tty: Allocate a pseudo-TTY
    ///     user: Username or UID
    ///     userns_mode: User namespace mode
    ///     volumes: Volume bindings as list (e.g., ["/host:/container:rw"])
    ///     volumes_from: Mount volumes from other containers as list
    ///     working_dir: Working directory inside the container
    ///
    /// Returns:
    ///     Container: Created container instance
    #[pyo3(signature = (image, *, attach_stderr=None, attach_stdin=None, attach_stdout=None, auto_remove=None, capabilities=None, command=None, cpu_shares=None, cpus=None, devices=None, entrypoint=None, env=None, expose=None, extra_hosts=None, labels=None, links=None, log_driver=None, memory=None, memory_swap=None, name=None, nano_cpus=None, network_mode=None, privileged=None, publish=None, publish_all_ports=None, restart_policy=None, security_options=None, stop_signal=None, stop_signal_num=None, stop_timeout=None, tty=None, user=None, userns_mode=None, volumes=None, volumes_from=None, working_dir=None))]
    fn create(
        &self,
        image: &str,
        attach_stderr: Option<bool>,
        attach_stdin: Option<bool>,
        attach_stdout: Option<bool>,
        auto_remove: Option<bool>,
        capabilities: Option<&Bound<'_, PyList>>,
        command: Option<&Bound<'_, PyList>>,
        cpu_shares: Option<u32>,
        cpus: Option<f64>,
        devices: Option<&Bound<'_, PyList>>,
        entrypoint: Option<&Bound<'_, PyList>>,
        env: Option<&Bound<'_, PyList>>,
        expose: Option<&Bound<'_, PyList>>,
        extra_hosts: Option<&Bound<'_, PyList>>,
        labels: Option<&Bound<'_, PyDict>>,
        links: Option<&Bound<'_, PyList>>,
        log_driver: Option<&str>,
        memory: Option<u64>,
        memory_swap: Option<i64>,
        name: Option<&str>,
        nano_cpus: Option<u64>,
        network_mode: Option<&str>,
        privileged: Option<bool>,
        publish: Option<&Bound<'_, PyList>>,
        publish_all_ports: Option<bool>,
        restart_policy: Option<&Bound<'_, PyDict>>, // name,maximum_retry_count,
        security_options: Option<&Bound<'_, PyList>>,
        stop_signal: Option<&str>,
        stop_signal_num: Option<u64>,
        stop_timeout: Option<&Bound<'_, PyDelta>>,
        tty: Option<bool>,
        user: Option<&str>,
        userns_mode: Option<&str>,
        volumes: Option<&Bound<'_, PyList>>,
        volumes_from: Option<&Bound<'_, PyList>>,
        working_dir: Option<&str>,
    ) -> PyResult<Pyo3Container> {
        let mut create_opts = ContainerCreateOpts::builder().image(image);

        let links: Option<Vec<String>> = if links.is_some() {
            links.unwrap().extract().unwrap()
        } else {
            None
        };
        let links: Option<Vec<&str>> = links
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let capabilities_strings: Option<Vec<String>> = if capabilities.is_some() {
            capabilities.unwrap().extract().unwrap()
        } else {
            None
        };
        let capabilities: Option<Vec<&str>> = capabilities_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let command_strings: Option<Vec<String>> = if command.is_some() {
            command.unwrap().extract().unwrap()
        } else {
            None
        };
        let command: Option<Vec<&str>> = command_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let entrypoint_strings: Option<Vec<String>> = if entrypoint.is_some() {
            entrypoint.unwrap().extract().unwrap()
        } else {
            None
        };
        let entrypoint: Option<Vec<&str>> = entrypoint_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let env_strings: Option<Vec<String>> = if env.is_some() {
            env.unwrap().extract().unwrap()
        } else {
            None
        };
        let env: Option<Vec<&str>> = env_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let extra_hosts_strings: Option<Vec<String>> = if extra_hosts.is_some() {
            extra_hosts.unwrap().extract().unwrap()
        } else {
            None
        };
        let extra_hosts: Option<Vec<&str>> = extra_hosts_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let security_options_strings: Option<Vec<String>> = if security_options.is_some() {
            security_options.unwrap().extract().unwrap()
        } else {
            None
        };
        let security_options: Option<Vec<&str>> = security_options_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let volumes_strings: Option<Vec<String>> = if volumes.is_some() {
            volumes.unwrap().extract().unwrap()
        } else {
            None
        };
        let volumes: Option<Vec<&str>> = volumes_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let volumes_from_strings: Option<Vec<String>> = if volumes_from.is_some() {
            volumes_from.unwrap().extract().unwrap()
        } else {
            None
        };
        let volumes_from: Option<Vec<&str>> = volumes_from_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let devices_vec: Option<Vec<HashMap<String, String>>> = if devices.is_some() {
            let list = devices.unwrap();
            let mut result = Vec::new();
            for item in list.iter() {
                let dict: HashMap<String, String> = item.extract().unwrap();
                result.push(dict);
            }
            Some(result)
        } else {
            None
        };
        let devices = devices_vec;

        let labels_map: Option<HashMap<String, String>> = if labels.is_some() {
            Some(labels.unwrap().extract().unwrap())
        } else {
            None
        };
        let labels: Option<HashMap<&str, &str>> = labels_map
            .as_ref()
            .map(|m| m.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect());

        let stop_timeout_duration: Option<std::time::Duration> =
            stop_timeout.map(|st| st.extract::<chrono::Duration>().unwrap().to_std().unwrap());
        let stop_timeout = stop_timeout_duration;

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

        bo_setter!(devices, create_opts);
        bo_setter!(links, create_opts);
        bo_setter!(capabilities, create_opts);
        bo_setter!(command, create_opts);
        bo_setter!(entrypoint, create_opts);
        bo_setter!(env, create_opts);
        bo_setter!(extra_hosts, create_opts);
        bo_setter!(security_options, create_opts);
        bo_setter!(volumes, create_opts);
        bo_setter!(volumes_from, create_opts);

        bo_setter!(labels, create_opts);
        bo_setter!(stop_timeout, create_opts);

        // Handle expose - expects list of dicts like [{"srcport": 8080, "protocol": "tcp", "hostport": 8000}]
        if let Some(expose_list) = expose {
            for item in expose_list.iter() {
                let port_dict: &Bound<'_, PyDict> = item.cast()?;
                let srcport: u32 = port_dict
                    .get_item("srcport")?
                    .expect("srcport required")
                    .extract()?;
                let hostport: u32 = port_dict
                    .get_item("hostport")?
                    .expect("hostport required")
                    .extract()?;
                let protocol: String = match port_dict.get_item("protocol")? {
                    Some(p) => p.extract()?,
                    None => "tcp".to_string(),
                };

                let publish_port = match protocol.as_str() {
                    "tcp" => PublishPort::tcp(srcport),
                    "udp" => PublishPort::udp(srcport),
                    "sctp" => PublishPort::sctp(srcport),
                    _ => {
                        return Err(exceptions::PyValueError::new_err(format!(
                            "unknown protocol: {}",
                            protocol
                        )))
                    }
                };

                create_opts = create_opts.expose(publish_port, hostport);
            }
        }

        // Handle publish - expects list of dicts like [{"port": 8080, "protocol": "tcp"}]
        if let Some(publish_list) = publish {
            for item in publish_list.iter() {
                let port_dict: &Bound<'_, PyDict> = item.cast()?;
                let port: u32 = port_dict
                    .get_item("port")?
                    .expect("port required")
                    .extract()?;
                let protocol: String = match port_dict.get_item("protocol")? {
                    Some(p) => p.extract()?,
                    None => "tcp".to_string(),
                };

                let publish_port = match protocol.as_str() {
                    "tcp" => PublishPort::tcp(port),
                    "udp" => PublishPort::udp(port),
                    "sctp" => PublishPort::sctp(port),
                    _ => {
                        return Err(exceptions::PyValueError::new_err(format!(
                            "unknown protocol: {}",
                            protocol
                        )))
                    }
                };

                create_opts = create_opts.publish(publish_port);
            }
        }

        if publish_all_ports.is_some() && publish_all_ports.unwrap() {
            create_opts = create_opts.publish_all_ports();
        }

        if restart_policy.is_some() {
            let policy_dict = restart_policy.unwrap();
            let name = policy_dict
                .get_item("name")
                .unwrap_or(None)
                .expect("restart_policy requires 'name' key")
                .extract::<String>()
                .unwrap();
            let max_retry = policy_dict
                .get_item("maximum_retry_count")
                .unwrap_or(None)
                .map(|v| v.extract::<u64>().unwrap())
                .unwrap_or(0);

            create_opts = create_opts.restart_policy(&name, max_retry);
        }

        // bo_setter!(expose, create_opts);
        // bo_setter!(publish, create_opts);

        let rv = __containers_create(&self.containers, &create_opts.build());
        match rv {
            Ok(container) => Ok(Pyo3Container {
                container,
                docker: self.docker.clone(),
            }),
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
        Pyo3Container {
            container: Container::new(docker.0.clone(), id),
            docker: docker.0,
        }
    }

    /// Get the container ID.
    ///
    /// Returns:
    ///     str: Container ID
    fn id(&self) -> String {
        self.container.id().to_string()
    }

    /// Inspect the container to get detailed information.
    ///
    /// Returns:
    ///     dict: Detailed container information including config, state, mounts, etc.
    fn inspect(&self) -> PyResult<Py<PyAny>> {
        let ci = __container_inspect(&self.container);
        Ok(pythonize_this!(ci))
    }

    /// Get container logs.
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
    ///     str: Container logs
    #[pyo3(signature = (stdout=None, stderr=None, timestamps=None, n_lines=None, all=None, since=None))]
    fn logs(
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
            // all needs to be called w/o a value
            log_opts = log_opts.all();
        }

        if since.is_some() {
            let rs_since: DateTime<Utc> = since.unwrap().extract().unwrap();
            log_opts = log_opts.since(&rs_since);
        }

        __container_logs(&self.container, &log_opts.build())
    }

    /// Remove the container (not implemented yet).
    fn remove(&self) -> PyResult<()> {
        Err(exceptions::PyNotImplementedError::new_err(
            "This method is not available yet.",
        ))
    }

    /// Delete the container.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the container cannot be deleted
    fn delete(&self) -> PyResult<()> {
        let rv = __container_delete(&self.container);
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

    /// Start the container.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the container cannot be started
    fn start(&self) -> PyResult<()> {
        let rv = __container_start(&self.container);

        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to start container",
            )),
        }
    }

    /// Stop the container.
    ///
    /// Args:
    ///     wait: Time to wait before killing the container (timedelta)
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the container cannot be stopped
    fn stop(&self, wait: Option<&Bound<'_, PyDelta>>) -> PyResult<()> {
        let wait: Option<std::time::Duration> = wait.map(|wait| {
            wait.extract::<chrono::Duration>()
                .unwrap()
                .to_std()
                .unwrap()
        });

        let rv = __container_stop(&self.container, wait);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to start container",
            )),
        }
    }

    /// Restart the container.
    ///
    /// Args:
    ///     wait: Time to wait before killing the container (timedelta)
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the container cannot be restarted
    fn restart(&self, wait: Option<&Bound<'_, PyDelta>>) -> PyResult<()> {
        let wait: Option<std::time::Duration> = wait.map(|wait| {
            wait.extract::<chrono::Duration>()
                .unwrap()
                .to_std()
                .unwrap()
        });

        let rv = __container_restart(&self.container, wait);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to stop container",
            )),
        }
    }

    /// Kill the container by sending a signal.
    ///
    /// Args:
    ///     signal: Signal to send (e.g., "SIGKILL", "SIGTERM")
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the container cannot be killed
    fn kill(&self, signal: Option<&str>) -> PyResult<()> {
        let rv = __container_kill(&self.container, signal);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to kill container",
            )),
        }
    }

    /// Rename the container.
    ///
    /// Args:
    ///     name: New name for the container
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the container cannot be renamed
    fn rename(&self, name: &str) -> PyResult<()> {
        let rv = __container_rename(&self.container, name);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to rename container",
            )),
        }
    }

    /// Pause the container.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the container cannot be paused
    fn pause(&self) -> PyResult<()> {
        let rv = __container_pause(&self.container);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to pause container",
            )),
        }
    }

    /// Unpause the container.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the container cannot be unpaused
    fn unpause(&self) -> PyResult<()> {
        let rv = __container_unpause(&self.container);
        match rv {
            Ok(_rv) => Ok(()),
            Err(_rv) => Err(exceptions::PySystemError::new_err(
                "Failed to unpause container",
            )),
        }
    }

    /// Wait for the container to stop.
    ///
    /// Returns:
    ///     dict: Wait response including status code
    fn wait(&self) -> Py<PyAny> {
        let rv = __container_wait(&self.container).unwrap();
        pythonize_this!(rv)
    }

    /// Get container resource usage statistics.
    ///
    /// Args:
    ///     stream: If True, continuously stream stats. If False (default), return single snapshot.
    ///
    /// Returns:
    ///     dict: Container statistics including CPU, memory, network, and I/O usage.
    ///           If stream=True, returns a list of stats snapshots.
    ///
    /// Raises:
    ///     SystemError: If stats cannot be retrieved
    #[pyo3(signature = (stream=None))]
    fn stats(&self, stream: Option<bool>) -> PyResult<Py<PyAny>> {
        let stream = stream.unwrap_or(false);
        let rv = __container_stats(&self.container, stream);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Attach to the container's standard streams.
    ///
    /// This method attaches to the container and collects output from stdout/stderr.
    /// It's designed for short-lived connections to collect output.
    ///
    /// Returns:
    ///     str: Combined stdout and stderr output from the container
    ///
    /// Raises:
    ///     SystemError: If attach fails
    fn attach(&self) -> PyResult<String> {
        let rv = __container_attach(&self.container);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Get filesystem changes made to the container.
    ///
    /// Returns a list of changes made to the container's filesystem.
    /// Each change includes the path and kind of change (Added, Modified, Deleted).
    ///
    /// Returns:
    ///     list[dict] | None: List of filesystem changes, or None if no changes
    ///
    /// Raises:
    ///     SystemError: If changes cannot be retrieved
    fn changes(&self) -> PyResult<Py<PyAny>> {
        let rv = __container_changes(&self.container);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Export the container as a tarball.
    ///
    /// Args:
    ///     path: Local path to save the exported tarball
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If export fails
    fn export(&self, path: &str) -> PyResult<()> {
        let rv = __container_export(&self.container);
        match rv {
            Ok(bytes) => {
                use std::io::Write;
                let mut file = File::create(path)
                    .map_err(|e| exceptions::PySystemError::new_err(format!("{e}")))?;
                file.write_all(&bytes)
                    .map_err(|e| exceptions::PySystemError::new_err(format!("{e}")))?;
                Ok(())
            }
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Get running processes in the container.
    ///
    /// Args:
    ///     ps_args: Arguments to pass to ps command (e.g., "aux", "-ef")
    ///
    /// Returns:
    ///     dict: Process information including titles and process list
    ///
    /// Raises:
    ///     SystemError: If top cannot be executed (Unix only)
    #[pyo3(signature = (ps_args=None))]
    fn top(&self, ps_args: Option<&str>) -> PyResult<Py<PyAny>> {
        let rv = __container_top(&self.container, ps_args);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Execute a command in the running container.
    ///
    /// Args:
    ///     command: Command to execute as list (e.g., ["/bin/sh", "-c", "ls"])
    ///     env: Environment variables as list (e.g., ["VAR=value"])
    ///     attach_stdout: Attach to stdout
    ///     attach_stderr: Attach to stderr
    ///     detach_keys: Override key sequence for detaching
    ///     tty: Allocate a pseudo-TTY
    ///     privileged: Run with extended privileges
    ///     user: Username or UID
    ///     working_dir: Working directory for the exec session
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the command cannot be executed
    fn exec(
        &self,
        command: &Bound<'_, PyList>,
        env: Option<&Bound<'_, PyList>>,
        attach_stdout: Option<bool>,
        attach_stderr: Option<bool>,
        detach_keys: Option<&str>,
        tty: Option<bool>,
        privileged: Option<bool>,
        user: Option<&str>,
        working_dir: Option<&str>,
    ) -> PyResult<()> {
        let command_strings: Vec<String> = command.extract().unwrap();
        let command: Vec<&str> = command_strings.iter().map(|s| s.as_str()).collect();
        let mut exec_opts = ExecCreateOpts::builder().command(command);

        if env.is_some() {
            let env_strings: Vec<String> = env.unwrap().extract().unwrap();
            let env: Vec<&str> = env_strings.iter().map(|s| s.as_str()).collect();
            exec_opts = exec_opts.env(env);
        }

        bo_setter!(attach_stdout, exec_opts);
        bo_setter!(attach_stderr, exec_opts);
        bo_setter!(tty, exec_opts);
        bo_setter!(detach_keys, exec_opts);
        bo_setter!(privileged, exec_opts);
        bo_setter!(user, exec_opts);
        bo_setter!(working_dir, exec_opts);

        let rv = __container_exec(&self.container, exec_opts.build());
        let rv = rv.unwrap();
        match rv {
            Ok(_rv) => Ok(()),
            Err(rv) => Err(exceptions::PySystemError::new_err(format!(
                "Failed to exec container {rv}"
            ))),
        }
    }

    /// Create an exec instance without starting it.
    ///
    /// Use this when you need to inspect or resize the exec session before or during execution.
    /// Returns the exec ID which can be used to create an Exec object.
    ///
    /// Args:
    ///     command: Command to execute as list (e.g., ["/bin/sh", "-c", "ls"])
    ///     env: Environment variables as list (e.g., ["VAR=value"])
    ///     attach_stdout: Attach to stdout
    ///     attach_stderr: Attach to stderr
    ///     detach_keys: Override key sequence for detaching
    ///     tty: Allocate a pseudo-TTY (required for resize)
    ///     privileged: Run with extended privileges
    ///     user: Username or UID
    ///     working_dir: Working directory for the exec session
    ///
    /// Returns:
    ///     str: Exec instance ID (use with Exec(docker, id) to get the exec object)
    ///
    /// Raises:
    ///     SystemError: If the exec cannot be created
    ///
    /// Example:
    ///     >>> exec_id = container.exec_create(["/bin/bash"], tty=True)
    ///     >>> exec_obj = Exec(docker, exec_id)
    ///     >>> exec_obj.resize(80, 24)
    ///     >>> exec_obj.inspect()
    #[pyo3(signature = (command, env=None, attach_stdout=None, attach_stderr=None, detach_keys=None, tty=None, privileged=None, user=None, working_dir=None))]
    fn exec_create(
        &self,
        command: &Bound<'_, PyList>,
        env: Option<&Bound<'_, PyList>>,
        attach_stdout: Option<bool>,
        attach_stderr: Option<bool>,
        detach_keys: Option<&str>,
        tty: Option<bool>,
        privileged: Option<bool>,
        user: Option<&str>,
        working_dir: Option<&str>,
    ) -> PyResult<String> {
        let command_strings: Vec<String> = command.extract().unwrap();
        let command: Vec<&str> = command_strings.iter().map(|s| s.as_str()).collect();
        let mut exec_opts = ExecCreateOpts::builder().command(command);

        if env.is_some() {
            let env_strings: Vec<String> = env.unwrap().extract().unwrap();
            let env: Vec<&str> = env_strings.iter().map(|s| s.as_str()).collect();
            exec_opts = exec_opts.env(env);
        }

        bo_setter!(attach_stdout, exec_opts);
        bo_setter!(attach_stderr, exec_opts);
        bo_setter!(tty, exec_opts);
        bo_setter!(detach_keys, exec_opts);
        bo_setter!(privileged, exec_opts);
        bo_setter!(user, exec_opts);
        bo_setter!(working_dir, exec_opts);

        let rv = __container_exec_create(
            self.docker.clone(),
            self.container.id().as_ref(),
            exec_opts.build(),
        );
        match rv {
            Ok(exec_id) => Ok(exec_id),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    fn copy_from(&self, src: &str, dst: &str) -> PyResult<()> {
        let rv = __container_copy_from(&self.container, src);

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

        let rv = __container_copy_file_into(&self.container, dst, &bytes);

        match rv {
            Ok(_rv) => Ok(()),
            Err(rv) => Err(exceptions::PySystemError::new_err(format!("{rv}"))),
        }
    }

    fn stat_file(&self, path: &str) -> Py<PyAny> {
        let rv = __container_stat_file(&self.container, path).unwrap();
        pythonize_this!(rv)
    }

    /// Create a new image from the container.
    ///
    /// Args:
    ///     repo: Repository name for the new image (e.g., "myimage")
    ///     tag: Tag for the new image (e.g., "v1.0")
    ///     comment: Commit message
    ///     author: Author of the commit (e.g., "John Doe <john@example.com>")
    ///     pause: Pause the container during commit
    ///     changes: Dockerfile instruction to apply (e.g., "CMD /bin/bash")
    ///
    /// Returns:
    ///     str: ID of the created image
    ///
    /// Raises:
    ///     SystemError: If commit fails
    #[pyo3(signature = (repo=None, tag=None, comment=None, author=None, pause=None, changes=None))]
    fn commit(
        &self,
        repo: Option<&str>,
        tag: Option<&str>,
        comment: Option<&str>,
        author: Option<&str>,
        pause: Option<bool>,
        changes: Option<&str>,
    ) -> PyResult<String> {
        let mut opts = ContainerCommitOpts::builder();

        bo_setter!(repo, opts);
        bo_setter!(tag, opts);
        bo_setter!(comment, opts);
        bo_setter!(author, opts);
        bo_setter!(pause, opts);
        bo_setter!(changes, opts);

        let rv = __container_commit(&self.container, &opts.build());
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    fn __repr__(&self) -> String {
        let inspect = __container_inspect(&self.container);
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
    let mut opts = ContainerStopOpts::builder();
    if let Some(w) = wait {
        opts = opts.wait(w);
    }
    container.stop(&opts.build()).await
}

#[tokio::main]
async fn __container_restart(
    container: &Container,
    wait: Option<std::time::Duration>,
) -> Result<(), docker_api::Error> {
    let mut opts = ContainerRestartOpts::builder();
    if let Some(w) = wait {
        opts = opts.wait(w);
    }
    container.restart(&opts.build()).await
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
async fn __container_stats(
    container: &Container,
    stream: bool,
) -> Result<serde_json::Value, docker_api::Error> {
    let mut stats_stream = container.stats();

    if stream {
        // Collect multiple stats snapshots (limit to avoid infinite stream)
        let mut stats_vec: Vec<serde_json::Value> = Vec::new();
        let mut count = 0;
        while let Some(stat_result) = stats_stream.next().await {
            match stat_result {
                Ok(stat) => {
                    stats_vec.push(stat);
                    count += 1;
                    if count >= 10 {
                        // Limit to 10 snapshots for streaming mode
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(serde_json::Value::Array(stats_vec))
    } else {
        // Return single snapshot
        match stats_stream.next().await {
            Some(Ok(stat)) => Ok(stat),
            Some(Err(e)) => Err(e),
            None => Ok(serde_json::Value::Null),
        }
    }
}

#[tokio::main]
async fn __container_attach(container: &Container) -> Result<String, docker_api::Error> {
    use futures_util::StreamExt;

    let mut multiplexer = container.attach().await?;
    let mut output = Vec::new();

    // Collect output chunks with a timeout/limit to avoid hanging
    let mut count = 0;
    while let Some(chunk_result) = multiplexer.next().await {
        match chunk_result {
            Ok(chunk) => {
                output.extend_from_slice(&chunk.to_vec());
                count += 1;
                if count >= 100 {
                    // Limit chunks to avoid infinite streaming
                    break;
                }
            }
            Err(_) => break,
        }
    }

    Ok(String::from_utf8_lossy(&output).to_string())
}

#[tokio::main]
async fn __container_changes(
    container: &Container,
) -> Result<Option<ContainerChanges200Response>, docker_api::Error> {
    container.changes().await
}

#[tokio::main]
async fn __container_export(container: &Container) -> Result<Vec<u8>, docker_api::Error> {
    container.export().try_concat().await
}

#[tokio::main]
async fn __container_top(
    container: &Container,
    ps_args: Option<&str>,
) -> Result<ContainerTop200Response, docker_api::Error> {
    container.top(ps_args).await
}

#[tokio::main]
async fn __container_commit(
    container: &Container,
    opts: &ContainerCommitOpts,
) -> Result<String, docker_api::Error> {
    container.commit(opts, None).await
}

#[tokio::main]
async fn __container_exec(
    container: &Container,
    exec_opts: ExecCreateOpts,
) -> Option<Result<TtyChunk, docker_api::conn::Error>> {
    let start_opts = ExecStartOpts::builder().build();
    match container.exec(&exec_opts, &start_opts).await {
        Ok(mut multiplexer) => multiplexer.next().await,
        Err(_) => None,
    }
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

#[tokio::main]
async fn __container_exec_create(
    docker: docker_api::Docker,
    container_id: &str,
    exec_opts: ExecCreateOpts,
) -> Result<String, docker_api::Error> {
    // Create the exec instance
    let exec = Exec::create(docker, container_id, &exec_opts).await?;
    // Inspect to get the ID (the only reliable way to get the ID from the Exec struct)
    let inspect = exec.inspect().await?;
    Ok(inspect.id.unwrap_or_default())
}
