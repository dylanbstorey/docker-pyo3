use std::collections::HashMap;

use crate::Pyo3Docker;
use docker_api::opts::{NodeListOpts, NodeUpdateOpts};
use docker_api::{Node, Nodes};
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::pythonize;

#[pymodule]
pub fn node(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Nodes>()?;
    m.add_class::<Pyo3Node>()?;
    Ok(())
}

/// Interface for managing Docker Swarm nodes collection.
///
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Nodes")]
pub struct Pyo3Nodes(pub Nodes);

/// Represents an individual Docker Swarm node.
///
/// Swarm mode must be enabled for these operations to work.
#[derive(Debug)]
#[pyclass(name = "Node")]
pub struct Pyo3Node(pub Node);

#[pymethods]
impl Pyo3Nodes {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Nodes(Nodes::new(docker.0))
    }

    /// Get a specific node by ID or name.
    ///
    /// Args:
    ///     id: Node ID or name
    ///
    /// Returns:
    ///     Node: Node instance
    pub fn get(&self, id: &str) -> Pyo3Node {
        Pyo3Node(self.0.get(id))
    }

    /// List all nodes in the swarm.
    ///
    /// Returns:
    ///     list[dict]: List of node information dictionaries
    ///
    /// Raises:
    ///     SystemError: If the operation fails (e.g., swarm not initialized)
    pub fn list(&self) -> PyResult<Py<PyAny>> {
        let rv = __nodes_list(&self.0, &Default::default());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __nodes_list(
    nodes: &Nodes,
    opts: &NodeListOpts,
) -> Result<Vec<docker_api::models::Node>, docker_api::Error> {
    nodes.list(opts).await
}

#[pymethods]
impl Pyo3Node {
    #[new]
    pub fn new(docker: Pyo3Docker, id: &str) -> Self {
        Pyo3Node(Node::new(docker.0, id))
    }

    /// Get the node ID.
    ///
    /// Returns:
    ///     str: Node ID
    pub fn id(&self) -> String {
        self.0.name().to_string()
    }

    /// Inspect the node to get detailed information.
    ///
    /// Returns:
    ///     dict: Detailed node information including status, description, spec, etc.
    ///
    /// Raises:
    ///     SystemError: If the operation fails
    pub fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __node_inspect(&self.0);

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Delete the node from the swarm.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the node cannot be deleted
    pub fn delete(&self) -> PyResult<()> {
        let rv = __node_delete(&self.0);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Force delete the node from the swarm.
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the node cannot be deleted
    pub fn force_delete(&self) -> PyResult<()> {
        let rv = __node_force_delete(&self.0);
        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Update the node configuration.
    ///
    /// Args:
    ///     version: Node version string (required, use inspect() to get current version)
    ///     name: Node name
    ///     role: Node role ("worker" or "manager")
    ///     availability: Node availability ("active", "pause", or "drain")
    ///     labels: Node labels as dict (e.g., {"env": "prod"})
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If the update fails
    #[pyo3(signature = (version, name=None, role=None, availability=None, labels=None))]
    pub fn update(
        &self,
        version: &str,
        name: Option<&str>,
        role: Option<&str>,
        availability: Option<&str>,
        labels: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        use docker_api::models::{NodeSpecAvailabilityInlineItem, NodeSpecRoleInlineItem};

        // Unfortunately the docker-api crate has a design issue where the setter methods
        // are on NodeUpdateOpts, not on NodeUpdateOptsBuilder. We need to manually construct
        // the opts by converting the builder struct fields.
        let rv = __node_update_with_params(
            &self.0,
            version,
            name,
            role.map(|r| match r.to_lowercase().as_str() {
                "worker" => Ok(NodeSpecRoleInlineItem::Worker),
                "manager" => Ok(NodeSpecRoleInlineItem::Manager),
                _ => Err(exceptions::PyValueError::new_err(format!(
                    "Invalid role: {}. Must be 'worker' or 'manager'",
                    r
                ))),
            })
            .transpose()?,
            availability
                .map(|a| match a.to_lowercase().as_str() {
                    "active" => Ok(NodeSpecAvailabilityInlineItem::Active),
                    "pause" => Ok(NodeSpecAvailabilityInlineItem::Pause),
                    "drain" => Ok(NodeSpecAvailabilityInlineItem::Drain),
                    _ => Err(exceptions::PyValueError::new_err(format!(
                        "Invalid availability: {}. Must be 'active', 'pause', or 'drain'",
                        a
                    ))),
                })
                .transpose()?,
            labels.map(|l| l.extract::<HashMap<String, String>>().unwrap()),
        );

        match rv {
            Ok(rv) => Ok(rv),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

#[tokio::main]
async fn __node_inspect(node: &Node) -> Result<docker_api::models::Node, docker_api::Error> {
    node.inspect().await
}

#[tokio::main]
async fn __node_delete(node: &Node) -> Result<(), docker_api::Error> {
    node.delete().await
}

#[tokio::main]
async fn __node_force_delete(node: &Node) -> Result<(), docker_api::Error> {
    node.force_delete().await
}

use docker_api::models::{NodeSpecAvailabilityInlineItem, NodeSpecRoleInlineItem};

#[tokio::main]
async fn __node_update_with_params(
    node: &Node,
    version: &str,
    name: Option<&str>,
    role: Option<NodeSpecRoleInlineItem>,
    availability: Option<NodeSpecAvailabilityInlineItem>,
    labels: Option<HashMap<String, String>>,
) -> Result<(), docker_api::Error> {
    // Build the opts by calling methods on NodeUpdateOpts (not the builder)
    // This is a workaround for the crate's design where builder methods are on the opts struct
    // We can work around this by using transmute since the builder and opts structs have
    // identical layout (both are wrappers around the same internal data)
    let empty_opts = NodeUpdateOpts::builder(version);

    // Use transmute to convert the builder to opts, then apply the setter methods
    unsafe {
        let opts: NodeUpdateOpts = std::mem::transmute(empty_opts);

        // Now apply the modifications
        let opts = if let Some(n) = name {
            opts.name(n)
        } else {
            opts
        };

        let opts = if let Some(r) = role {
            opts.role(r)
        } else {
            opts
        };

        let opts = if let Some(a) = availability {
            opts.availability(a)
        } else {
            opts
        };

        let opts = if let Some(l) = labels {
            opts.labels(l.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        } else {
            opts
        };

        node.update(&opts).await
    }
}
