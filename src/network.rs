use std::collections::HashMap;

use crate::Pyo3Docker;
use docker_api::opts::{ContainerConnectionOpts, NetworkPruneOpts};
use docker_api::opts::{ContainerDisconnectionOpts, NetworkCreateOpts};
use docker_api::{models::NetworkPrune200Response, Network, Networks};
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pythonize::pythonize;

#[pymodule]
pub fn network(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Networks>()?;
    m.add_class::<Pyo3Network>()?;
    Ok(())
}

#[derive(Debug)]
#[pyclass(name = "Networks")]
pub struct Pyo3Networks(pub Networks);

#[derive(Debug)]
#[pyclass(name = "Network")]
pub struct Pyo3Network(pub Network);

#[pymethods]
impl Pyo3Networks {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Networks(Networks::new(docker.0))
    }

    pub fn get(&self, id: &str) -> Pyo3Network {
        Pyo3Network(self.0.get(id))
    }

    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __networks_list(&self.0);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    pub fn prune(&self) -> PyResult<Py<PyAny>> {
        let rv = __networks_prune(&self.0, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    #[pyo3(signature = (name, *, check_duplicate=None, driver=None, internal=None, attachable=None, ingress=None, enable_ipv6=None, options=None, labels=None))]
    pub fn create(
        &self,
        name: &str,
        check_duplicate: Option<bool>,
        driver: Option<&str>,
        internal: Option<bool>,
        attachable: Option<bool>,
        ingress: Option<bool>,
        enable_ipv6: Option<bool>,
        options: Option<&Bound<'_, PyDict>>,
        labels: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Pyo3Network> {
        let mut network_opts = NetworkCreateOpts::builder(name);

        let options_map: Option<HashMap<String, String>> = if options.is_some() {
            Some(options.unwrap().extract().unwrap())
        } else {
            None
        };
        let options: Option<HashMap<&str, &str>> = options_map
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

        bo_setter!(check_duplicate, network_opts);
        bo_setter!(driver, network_opts);
        bo_setter!(internal, network_opts);
        bo_setter!(attachable, network_opts);
        bo_setter!(ingress, network_opts);
        bo_setter!(enable_ipv6, network_opts);
        bo_setter!(options, network_opts);
        bo_setter!(labels, network_opts);

        let rv = __networks_create(&self.0, &network_opts.build());
        match rv {
            Ok(rv) => Ok(Pyo3Network(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __networks_list(
    networks: &Networks,
) -> Result<Vec<docker_api::models::Network>, docker_api::Error> {
    networks.list(&Default::default()).await
}

#[tokio::main]
async fn __networks_prune(
    networks: &Networks,
    opts: &NetworkPruneOpts,
) -> Result<NetworkPrune200Response, docker_api::Error> {
    networks.prune(opts).await
}

#[tokio::main]
async fn __networks_create(
    networks: &Networks,
    opts: &NetworkCreateOpts,
) -> Result<Network, docker_api::Error> {
    networks.create(opts).await
}

#[pymethods]
impl Pyo3Network {
    #[new]
    pub fn new(docker: Pyo3Docker, id: &str) -> Self {
        Pyo3Network(Network::new(docker.0, id))
    }

    pub fn id(&self) -> String {
        self.0.id().to_string()
    }

    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __network_inspect(&self.0);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    pub fn delete(&self) -> PyResult<()> {
        let rv = __network_delete(&self.0);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    #[pyo3(signature = (container_id, ipam_config=None, aliases=None, links=None, network_id=None, endpoint_id=None, gateway=None, ipv4=None, prefix_len=None, ipv6_gateway=None, ipv6=None, ipv6_prefix_len=None, mac=None, driver_opts=None))]
    pub fn connect(
        &self,
        container_id: &str,
        ipam_config: Option<&str>,
        aliases: Option<&Bound<'_, PyList>>,
        links: Option<&Bound<'_, PyList>>,
        network_id: Option<&str>,
        endpoint_id: Option<&str>,
        gateway: Option<&str>,
        ipv4: Option<&str>,
        prefix_len: Option<isize>,
        ipv6_gateway: Option<&str>,
        ipv6: Option<&str>,
        ipv6_prefix_len: Option<i64>,
        mac: Option<&str>,
        driver_opts: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let mut connect_opts = ContainerConnectionOpts::builder(container_id);

        let aliases_strings: Option<Vec<String>> = if aliases.is_some() {
            aliases.unwrap().extract().unwrap()
        } else {
            None
        };
        let aliases: Option<Vec<&str>> = aliases_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let links_strings: Option<Vec<String>> = if links.is_some() {
            links.unwrap().extract().unwrap()
        } else {
            None
        };
        let links: Option<Vec<&str>> = links_strings
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());

        let driver_opts_map: Option<HashMap<String, String>> = if driver_opts.is_some() {
            driver_opts.unwrap().extract().unwrap()
        } else {
            None
        };
        let driver_opts: Option<HashMap<&str, &str>> = driver_opts_map
            .as_ref()
            .map(|m| m.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect());

        bo_setter!(network_id, connect_opts);
        bo_setter!(endpoint_id, connect_opts);
        bo_setter!(gateway, connect_opts);
        bo_setter!(ipv4, connect_opts);
        bo_setter!(prefix_len, connect_opts);
        bo_setter!(ipv6_gateway, connect_opts);
        bo_setter!(ipv6, connect_opts);
        bo_setter!(ipv6_prefix_len, connect_opts);
        bo_setter!(mac, connect_opts);

        bo_setter!(aliases, connect_opts);
        bo_setter!(links, connect_opts);
        bo_setter!(driver_opts, connect_opts);

        // bo_setter!(ipam_config, connect_opts);

        let rv = __network_connect(&self.0, &connect_opts.build());

        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    #[pyo3(signature = (container_id, force=None))]
    pub fn disconnect(&self, container_id: &str, force: Option<bool>) -> PyResult<()> {
        let mut disconnect_opts = ContainerDisconnectionOpts::builder(container_id);
        bo_setter!(force, disconnect_opts);

        let rv = __network_disconnect(&self.0, &disconnect_opts.build());

        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __network_inspect(
    network: &Network,
) -> Result<docker_api::models::Network, docker_api::Error> {
    network.inspect().await
}

#[tokio::main]
async fn __network_delete(network: &Network) -> Result<(), docker_api::Error> {
    network.delete().await
}

#[tokio::main]
async fn __network_connect(
    network: &Network,
    opts: &ContainerConnectionOpts,
) -> Result<(), docker_api::Error> {
    network.connect(opts).await
}

#[tokio::main]
async fn __network_disconnect(
    network: &Network,
    opts: &ContainerDisconnectionOpts,
) -> Result<(), docker_api::Error> {
    network.disconnect(opts).await
}
