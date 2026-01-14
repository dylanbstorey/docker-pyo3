//! Docker Compose file parsing and orchestration support.
//!
//! This module provides types and functions for parsing Docker Compose files
//! (docker-compose.yml / compose.yaml) and orchestrating multi-container deployments.

use docker_api::opts::{
    ContainerCreateOpts, ContainerListOpts, ContainerRestartOpts, ContainerStopOpts,
    ExecCreateOpts, ExecStartOpts, ImageBuildOpts, ImagePushOpts, LogsOpts, NetworkCreateOpts,
    PublishPort, PullOpts, RegistryAuth, VolumeCreateOpts,
};
use docker_api::{Containers, Images, Networks, Volumes};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pythonize::pythonize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::Pyo3Docker;

#[pymodule]
pub fn compose(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3ComposeFile>()?;
    m.add_class::<Pyo3ComposeProject>()?;
    m.add_function(wrap_pyfunction!(parse_compose_string, m)?)?;
    m.add_function(wrap_pyfunction!(parse_compose_file, m)?)?;
    Ok(())
}

/// Represents a parsed Docker Compose file.
///
/// Use `parse_compose_file()` or `parse_compose_string()` to create instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass(name = "ComposeFile")]
pub struct Pyo3ComposeFile {
    /// Compose file format version (e.g., "3.8", "3", "2.1")
    #[serde(default)]
    pub version: Option<String>,

    /// Service definitions
    #[serde(default)]
    pub services: HashMap<String, ComposeService>,

    /// Network definitions
    #[serde(default)]
    pub networks: HashMap<String, Option<ComposeNetwork>>,

    /// Volume definitions
    #[serde(default)]
    pub volumes: HashMap<String, Option<ComposeVolume>>,

    /// Config definitions (Swarm mode)
    #[serde(default)]
    pub configs: HashMap<String, Option<ComposeConfig>>,

    /// Secret definitions (Swarm mode)
    #[serde(default)]
    pub secrets: HashMap<String, Option<ComposeSecret>>,

    /// Top-level name for the project
    #[serde(default)]
    pub name: Option<String>,
}

/// Service definition in a Compose file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComposeService {
    /// Image to use for this service
    #[serde(default)]
    pub image: Option<String>,

    /// Build configuration
    #[serde(default)]
    pub build: Option<ComposeBuild>,

    /// Container name override
    #[serde(default)]
    pub container_name: Option<String>,

    /// Command to run
    #[serde(default)]
    pub command: Option<StringOrList>,

    /// Entrypoint override
    #[serde(default)]
    pub entrypoint: Option<StringOrList>,

    /// Environment variables
    #[serde(default)]
    pub environment: Option<EnvironmentVars>,

    /// Environment file(s)
    #[serde(default)]
    pub env_file: Option<StringOrList>,

    /// Port mappings
    #[serde(default)]
    pub ports: Option<Vec<PortMapping>>,

    /// Volume mounts
    #[serde(default)]
    #[serde(alias = "volume")]
    pub volumes: Option<Vec<VolumeMount>>,

    /// Network connections
    #[serde(default)]
    pub networks: Option<ServiceNetworks>,

    /// Service dependencies
    #[serde(default)]
    pub depends_on: Option<DependsOn>,

    /// Restart policy
    #[serde(default)]
    pub restart: Option<String>,

    /// Working directory
    #[serde(default)]
    pub working_dir: Option<String>,

    /// User to run as
    #[serde(default)]
    pub user: Option<String>,

    /// Labels
    #[serde(default)]
    pub labels: Option<Labels>,

    /// Extra hosts
    #[serde(default)]
    pub extra_hosts: Option<Vec<String>>,

    /// DNS servers
    #[serde(default)]
    pub dns: Option<StringOrList>,

    /// Hostname
    #[serde(default)]
    pub hostname: Option<String>,

    /// Domain name
    #[serde(default)]
    pub domainname: Option<String>,

    /// Privileged mode
    #[serde(default)]
    pub privileged: Option<bool>,

    /// Read-only root filesystem
    #[serde(default)]
    pub read_only: Option<bool>,

    /// Stdin open
    #[serde(default)]
    pub stdin_open: Option<bool>,

    /// TTY allocation
    #[serde(default)]
    pub tty: Option<bool>,

    /// Stop signal
    #[serde(default)]
    pub stop_signal: Option<String>,

    /// Stop grace period
    #[serde(default)]
    pub stop_grace_period: Option<String>,

    /// Health check configuration
    #[serde(default)]
    pub healthcheck: Option<HealthCheck>,

    /// Logging configuration
    #[serde(default)]
    pub logging: Option<LoggingConfig>,

    /// Deploy configuration (Swarm mode)
    #[serde(default)]
    pub deploy: Option<DeployConfig>,

    /// Secrets to expose
    #[serde(default)]
    pub secrets: Option<Vec<ServiceSecret>>,

    /// Configs to expose
    #[serde(default)]
    pub configs: Option<Vec<ServiceConfig>>,

    /// Capabilities to add
    #[serde(default)]
    pub cap_add: Option<Vec<String>>,

    /// Capabilities to drop
    #[serde(default)]
    pub cap_drop: Option<Vec<String>>,

    /// Devices to map
    #[serde(default)]
    pub devices: Option<Vec<String>>,

    /// Expose ports (not published)
    #[serde(default)]
    pub expose: Option<Vec<StringOrInt>>,

    /// Links (legacy)
    #[serde(default)]
    pub links: Option<Vec<String>>,

    /// Network mode
    #[serde(default)]
    pub network_mode: Option<String>,

    /// PID mode
    #[serde(default)]
    pub pid: Option<String>,

    /// IPC mode
    #[serde(default)]
    pub ipc: Option<String>,

    /// Security options
    #[serde(default)]
    pub security_opt: Option<Vec<String>>,

    /// Sysctls
    #[serde(default)]
    pub sysctls: Option<HashMap<String, StringOrInt>>,

    /// Ulimits
    #[serde(default)]
    pub ulimits: Option<HashMap<String, Ulimit>>,

    /// tmpfs mounts
    #[serde(default)]
    pub tmpfs: Option<StringOrList>,

    /// Init process
    #[serde(default)]
    pub init: Option<bool>,

    /// Profiles this service belongs to
    #[serde(default)]
    pub profiles: Option<Vec<String>>,

    /// Platform specification
    #[serde(default)]
    pub platform: Option<String>,

    /// Pull policy
    #[serde(default)]
    pub pull_policy: Option<String>,

    /// Scale (number of replicas)
    #[serde(default)]
    pub scale: Option<i32>,

    /// Memory limit
    #[serde(default)]
    pub mem_limit: Option<StringOrInt>,

    /// Memory reservation
    #[serde(default)]
    pub mem_reservation: Option<StringOrInt>,

    /// CPU count
    #[serde(default)]
    pub cpus: Option<f64>,

    /// Shared memory size
    #[serde(default)]
    pub shm_size: Option<StringOrInt>,
}

/// Build configuration for a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComposeBuild {
    /// Simple context path
    Simple(String),
    /// Full build configuration
    Full(BuildConfig),
}

/// Full build configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    /// Build context path
    #[serde(default)]
    pub context: Option<String>,

    /// Dockerfile path
    #[serde(default)]
    pub dockerfile: Option<String>,

    /// Build arguments
    #[serde(default)]
    pub args: Option<HashMap<String, Option<String>>>,

    /// Target stage
    #[serde(default)]
    pub target: Option<String>,

    /// Cache from images
    #[serde(default)]
    pub cache_from: Option<Vec<String>>,

    /// Extra hosts
    #[serde(default)]
    pub extra_hosts: Option<Vec<String>>,

    /// Network mode during build
    #[serde(default)]
    pub network: Option<String>,

    /// SSH authentication
    #[serde(default)]
    pub ssh: Option<Vec<String>>,

    /// Labels
    #[serde(default)]
    pub labels: Option<Labels>,

    /// Platform
    #[serde(default)]
    pub platform: Option<String>,
}

/// Network definition.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComposeNetwork {
    /// Network driver
    #[serde(default)]
    pub driver: Option<String>,

    /// Driver options
    #[serde(default)]
    pub driver_opts: Option<HashMap<String, String>>,

    /// External network reference
    #[serde(default)]
    pub external: Option<ExternalRef>,

    /// Enable IPv6
    #[serde(default)]
    pub enable_ipv6: Option<bool>,

    /// IPAM configuration
    #[serde(default)]
    pub ipam: Option<IpamConfig>,

    /// Internal network (no external access)
    #[serde(default)]
    pub internal: Option<bool>,

    /// Attachable
    #[serde(default)]
    pub attachable: Option<bool>,

    /// Labels
    #[serde(default)]
    pub labels: Option<Labels>,

    /// Network name
    #[serde(default)]
    pub name: Option<String>,
}

/// Volume definition.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComposeVolume {
    /// Volume driver
    #[serde(default)]
    pub driver: Option<String>,

    /// Driver options
    #[serde(default)]
    pub driver_opts: Option<HashMap<String, String>>,

    /// External volume reference
    #[serde(default)]
    pub external: Option<ExternalRef>,

    /// Labels
    #[serde(default)]
    pub labels: Option<Labels>,

    /// Volume name
    #[serde(default)]
    pub name: Option<String>,
}

/// Config definition (Swarm mode).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComposeConfig {
    /// File path
    #[serde(default)]
    pub file: Option<String>,

    /// External config reference
    #[serde(default)]
    pub external: Option<ExternalRef>,

    /// Config name
    #[serde(default)]
    pub name: Option<String>,

    /// Template driver
    #[serde(default)]
    pub template_driver: Option<String>,
}

/// Secret definition (Swarm mode).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComposeSecret {
    /// File path
    #[serde(default)]
    pub file: Option<String>,

    /// Environment variable
    #[serde(default)]
    pub environment: Option<String>,

    /// External secret reference
    #[serde(default)]
    pub external: Option<ExternalRef>,

    /// Secret name
    #[serde(default)]
    pub name: Option<String>,

    /// Template driver
    #[serde(default)]
    pub template_driver: Option<String>,
}

/// External resource reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExternalRef {
    /// Simple boolean
    Bool(bool),
    /// Named external resource
    Named { name: Option<String> },
}

/// Service networks configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceNetworks {
    /// Simple list of network names
    List(Vec<String>),
    /// Map with network-specific options
    Map(HashMap<String, Option<ServiceNetworkConfig>>),
}

/// Service-specific network configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceNetworkConfig {
    /// Aliases for this service on the network
    #[serde(default)]
    pub aliases: Option<Vec<String>>,

    /// IPv4 address
    #[serde(default)]
    pub ipv4_address: Option<String>,

    /// IPv6 address
    #[serde(default)]
    pub ipv6_address: Option<String>,

    /// Priority
    #[serde(default)]
    pub priority: Option<i32>,
}

/// Depends on configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependsOn {
    /// Simple list of service names
    List(Vec<String>),
    /// Map with conditions
    Map(HashMap<String, DependsOnCondition>),
}

/// Depends on condition.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependsOnCondition {
    /// Condition to wait for
    #[serde(default)]
    pub condition: Option<String>,
}

/// Health check configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthCheck {
    /// Test command
    #[serde(default)]
    pub test: Option<StringOrList>,

    /// Interval between checks
    #[serde(default)]
    pub interval: Option<String>,

    /// Timeout for check
    #[serde(default)]
    pub timeout: Option<String>,

    /// Number of retries
    #[serde(default)]
    pub retries: Option<i32>,

    /// Start period
    #[serde(default)]
    pub start_period: Option<String>,

    /// Disable health check
    #[serde(default)]
    pub disable: Option<bool>,
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    /// Logging driver
    #[serde(default)]
    pub driver: Option<String>,

    /// Driver options
    #[serde(default)]
    pub options: Option<HashMap<String, String>>,
}

/// Deploy configuration (Swarm mode).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeployConfig {
    /// Deployment mode
    #[serde(default)]
    pub mode: Option<String>,

    /// Number of replicas
    #[serde(default)]
    pub replicas: Option<i32>,

    /// Endpoint mode
    #[serde(default)]
    pub endpoint_mode: Option<String>,

    /// Placement constraints
    #[serde(default)]
    pub placement: Option<PlacementConfig>,

    /// Resource limits and reservations
    #[serde(default)]
    pub resources: Option<ResourceConfig>,

    /// Restart policy
    #[serde(default)]
    pub restart_policy: Option<RestartPolicyConfig>,

    /// Rollback configuration
    #[serde(default)]
    pub rollback_config: Option<UpdateConfig>,

    /// Update configuration
    #[serde(default)]
    pub update_config: Option<UpdateConfig>,

    /// Labels
    #[serde(default)]
    pub labels: Option<Labels>,
}

/// Placement configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlacementConfig {
    /// Constraints
    #[serde(default)]
    pub constraints: Option<Vec<String>>,

    /// Preferences
    #[serde(default)]
    pub preferences: Option<Vec<HashMap<String, String>>>,

    /// Max replicas per node
    #[serde(default)]
    pub max_replicas_per_node: Option<i32>,
}

/// Resource configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceConfig {
    /// Resource limits
    #[serde(default)]
    pub limits: Option<ResourceSpec>,

    /// Resource reservations
    #[serde(default)]
    pub reservations: Option<ResourceSpec>,
}

/// Resource specification.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceSpec {
    /// CPU limit/reservation
    #[serde(default)]
    pub cpus: Option<String>,

    /// Memory limit/reservation
    #[serde(default)]
    pub memory: Option<String>,

    /// Devices
    #[serde(default)]
    pub devices: Option<Vec<DeviceSpec>>,
}

/// Device specification.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceSpec {
    /// Device capabilities
    #[serde(default)]
    pub capabilities: Option<Vec<String>>,

    /// Driver
    #[serde(default)]
    pub driver: Option<String>,

    /// Device count
    #[serde(default)]
    pub count: Option<StringOrInt>,

    /// Device IDs
    #[serde(default)]
    pub device_ids: Option<Vec<String>>,

    /// Options
    #[serde(default)]
    pub options: Option<HashMap<String, String>>,
}

/// Restart policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RestartPolicyConfig {
    /// Condition for restart
    #[serde(default)]
    pub condition: Option<String>,

    /// Delay between restarts
    #[serde(default)]
    pub delay: Option<String>,

    /// Max attempts
    #[serde(default)]
    pub max_attempts: Option<i32>,

    /// Window for restart decisions
    #[serde(default)]
    pub window: Option<String>,
}

/// Update/rollback configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateConfig {
    /// Parallelism
    #[serde(default)]
    pub parallelism: Option<i32>,

    /// Delay between updates
    #[serde(default)]
    pub delay: Option<String>,

    /// Failure action
    #[serde(default)]
    pub failure_action: Option<String>,

    /// Monitor duration
    #[serde(default)]
    pub monitor: Option<String>,

    /// Max failure ratio
    #[serde(default)]
    pub max_failure_ratio: Option<f64>,

    /// Order of operations
    #[serde(default)]
    pub order: Option<String>,
}

/// Service secret reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceSecret {
    /// Simple secret name
    Simple(String),
    /// Full secret configuration
    Full(ServiceSecretConfig),
}

/// Service secret configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceSecretConfig {
    /// Source secret name
    #[serde(default)]
    pub source: Option<String>,

    /// Target path in container
    #[serde(default)]
    pub target: Option<String>,

    /// UID
    #[serde(default)]
    pub uid: Option<String>,

    /// GID
    #[serde(default)]
    pub gid: Option<String>,

    /// File mode
    #[serde(default)]
    pub mode: Option<i32>,
}

/// Service config reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceConfig {
    /// Simple config name
    Simple(String),
    /// Full config configuration
    Full(ServiceConfigConfig),
}

/// Service config configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceConfigConfig {
    /// Source config name
    #[serde(default)]
    pub source: Option<String>,

    /// Target path in container
    #[serde(default)]
    pub target: Option<String>,

    /// UID
    #[serde(default)]
    pub uid: Option<String>,

    /// GID
    #[serde(default)]
    pub gid: Option<String>,

    /// File mode
    #[serde(default)]
    pub mode: Option<i32>,
}

/// IPAM configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpamConfig {
    /// IPAM driver
    #[serde(default)]
    pub driver: Option<String>,

    /// Subnet configurations
    #[serde(default)]
    pub config: Option<Vec<IpamPoolConfig>>,

    /// Driver options
    #[serde(default)]
    pub options: Option<HashMap<String, String>>,
}

/// IPAM pool configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpamPoolConfig {
    /// Subnet CIDR
    #[serde(default)]
    pub subnet: Option<String>,

    /// Gateway address
    #[serde(default)]
    pub gateway: Option<String>,

    /// IP range
    #[serde(default)]
    pub ip_range: Option<String>,

    /// Auxiliary addresses
    #[serde(default)]
    pub aux_addresses: Option<HashMap<String, String>>,
}

/// Ulimit configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ulimit {
    /// Simple integer value (both soft and hard)
    Simple(i64),
    /// Separate soft and hard values
    Full {
        soft: Option<i64>,
        hard: Option<i64>,
    },
}

/// String or list of strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrList {
    String(String),
    List(Vec<String>),
}

/// String or integer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrInt {
    String(String),
    Int(i64),
}

/// Environment variables - can be list or map.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EnvironmentVars {
    /// List format (KEY=value)
    List(Vec<String>),
    /// Map format
    Map(HashMap<String, Option<StringOrInt>>),
}

/// Labels - can be list or map.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Labels {
    /// List format
    List(Vec<String>),
    /// Map format
    Map(HashMap<String, String>),
}

/// Port mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PortMapping {
    /// Simple string format (e.g., "8080:80")
    Simple(String),
    /// Integer format
    Int(i64),
    /// Full port configuration
    Full(PortConfig),
}

/// Full port configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortConfig {
    /// Target port (container)
    #[serde(default)]
    pub target: Option<i32>,

    /// Published port (host)
    #[serde(default)]
    pub published: Option<StringOrInt>,

    /// Protocol (tcp/udp)
    #[serde(default)]
    pub protocol: Option<String>,

    /// Mode (host/ingress)
    #[serde(default)]
    pub mode: Option<String>,

    /// Host IP
    #[serde(default)]
    pub host_ip: Option<String>,
}

/// Volume mount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VolumeMount {
    /// Simple string format (e.g., "./data:/app/data")
    Simple(String),
    /// Full volume configuration
    Full(VolumeMountConfig),
}

/// Full volume mount configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeMountConfig {
    /// Mount type (volume, bind, tmpfs, npipe)
    #[serde(default, rename = "type")]
    pub mount_type: Option<String>,

    /// Source path or volume name
    #[serde(default)]
    pub source: Option<String>,

    /// Target path in container
    #[serde(default)]
    pub target: Option<String>,

    /// Read-only
    #[serde(default)]
    pub read_only: Option<bool>,

    /// Bind mount options
    #[serde(default)]
    pub bind: Option<BindOptions>,

    /// Volume options
    #[serde(default)]
    pub volume: Option<VolumeOptions>,

    /// tmpfs options
    #[serde(default)]
    pub tmpfs: Option<TmpfsOptions>,

    /// Consistency mode
    #[serde(default)]
    pub consistency: Option<String>,
}

/// Bind mount options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BindOptions {
    /// Bind propagation
    #[serde(default)]
    pub propagation: Option<String>,

    /// Create host path if missing
    #[serde(default)]
    pub create_host_path: Option<bool>,

    /// SELinux label
    #[serde(default)]
    pub selinux: Option<String>,
}

/// Volume options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeOptions {
    /// Disable copy-up
    #[serde(default)]
    pub nocopy: Option<bool>,
}

/// tmpfs options.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TmpfsOptions {
    /// tmpfs size
    #[serde(default)]
    pub size: Option<StringOrInt>,

    /// File mode
    #[serde(default)]
    pub mode: Option<i32>,
}

// Python methods implementation

#[pymethods]
impl Pyo3ComposeFile {
    /// Get the compose file version.
    ///
    /// Returns:
    ///     str | None: Version string if specified
    #[getter]
    pub fn get_version(&self) -> Option<String> {
        self.version.clone()
    }

    /// Get the project name.
    ///
    /// Returns:
    ///     str | None: Project name if specified
    #[getter]
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Get list of service names.
    ///
    /// Returns:
    ///     list[str]: List of service names
    pub fn service_names(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }

    /// Get list of network names.
    ///
    /// Returns:
    ///     list[str]: List of network names
    pub fn network_names(&self) -> Vec<String> {
        self.networks.keys().cloned().collect()
    }

    /// Get list of volume names.
    ///
    /// Returns:
    ///     list[str]: List of volume names
    pub fn volume_names(&self) -> Vec<String> {
        self.volumes.keys().cloned().collect()
    }

    /// Get list of config names (Swarm mode).
    ///
    /// Returns:
    ///     list[str]: List of config names
    pub fn config_names(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }

    /// Get list of secret names (Swarm mode).
    ///
    /// Returns:
    ///     list[str]: List of secret names
    pub fn secret_names(&self) -> Vec<String> {
        self.secrets.keys().cloned().collect()
    }

    /// Get the full compose file as a dictionary.
    ///
    /// Returns:
    ///     dict: Complete compose file structure as nested dict
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        pythonize(py, self)
            .map(|bound| bound.unbind())
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// Get a specific service configuration.
    ///
    /// Args:
    ///     name: Service name
    ///
    /// Returns:
    ///     dict | None: Service configuration if found
    pub fn get_service(&self, py: Python<'_>, name: &str) -> PyResult<Option<Py<PyAny>>> {
        if let Some(service) = self.services.get(name) {
            let result = pythonize(py, service)
                .map(|bound| bound.unbind())
                .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Convert back to YAML string.
    ///
    /// Returns:
    ///     str: YAML representation of the compose file
    ///
    /// Raises:
    ///     ValueError: If serialization fails
    pub fn to_yaml(&self) -> PyResult<String> {
        serde_yaml::to_string(self)
            .map_err(|e| PyValueError::new_err(format!("YAML serialization error: {}", e)))
    }
}

/// Parse a Docker Compose file from a YAML string.
///
/// Args:
///     content: YAML content as string
///
/// Returns:
///     ComposeFile: Parsed compose file object
///
/// Raises:
///     ValueError: If parsing fails
///
/// Example:
///     >>> content = '''
///     ... version: "3.8"
///     ... services:
///     ...   web:
///     ...     image: nginx
///     ... '''
///     >>> compose = parse_compose_string(content)
///     >>> compose.service_names()
///     ['web']
#[pyfunction]
pub fn parse_compose_string(content: &str) -> PyResult<Pyo3ComposeFile> {
    serde_yaml::from_str(content)
        .map_err(|e| PyValueError::new_err(format!("Failed to parse compose file: {}", e)))
}

/// Parse a Docker Compose file from a file path.
///
/// Args:
///     path: Path to the compose file (docker-compose.yml or compose.yaml)
///
/// Returns:
///     ComposeFile: Parsed compose file object
///
/// Raises:
///     ValueError: If the file cannot be read or parsing fails
///
/// Example:
///     >>> compose = parse_compose_file("docker-compose.yml")
///     >>> compose.service_names()
///     ['web', 'db']
#[pyfunction]
pub fn parse_compose_file(path: &str) -> PyResult<Pyo3ComposeFile> {
    let path = Path::new(path);
    let content = fs::read_to_string(path)
        .map_err(|e| PyValueError::new_err(format!("Failed to read file: {}", e)))?;

    parse_compose_string(&content)
}

/// Manages a Docker Compose project for orchestrating multi-container deployments.
///
/// A ComposeProject represents a running or potential deployment of a Compose file.
/// It provides methods to bring services up (create and start) or down (stop and remove).
///
/// Example:
///     >>> docker = Docker()
///     >>> compose = parse_compose_file("docker-compose.yml")
///     >>> project = ComposeProject(docker, compose, "myproject")
///     >>> project.up()  # Create networks, volumes, and start containers
///     >>> project.down()  # Stop and remove containers
#[derive(Debug)]
#[pyclass(name = "ComposeProject")]
pub struct Pyo3ComposeProject {
    docker: docker_api::Docker,
    compose: Pyo3ComposeFile,
    project_name: String,
}

/// Result of a compose up operation
#[derive(Debug, Clone, Serialize)]
pub struct ComposeUpResult {
    /// IDs of created networks
    pub networks: Vec<String>,
    /// Names of created volumes
    pub volumes: Vec<String>,
    /// IDs of created/started containers
    pub containers: Vec<String>,
}

/// Result of a compose down operation
#[derive(Debug, Clone, Serialize)]
pub struct ComposeDownResult {
    /// IDs of stopped containers
    pub stopped_containers: Vec<String>,
    /// IDs of removed containers
    pub removed_containers: Vec<String>,
    /// IDs of removed networks
    pub removed_networks: Vec<String>,
    /// Names of removed volumes
    pub removed_volumes: Vec<String>,
}

#[pymethods]
impl Pyo3ComposeProject {
    /// Create a new ComposeProject.
    ///
    /// Args:
    ///     docker: Docker client instance
    ///     compose: Parsed ComposeFile instance
    ///     project_name: Name for this project (used as prefix for resources)
    ///
    /// Returns:
    ///     ComposeProject: Project instance ready for up/down operations
    #[new]
    pub fn new(docker: Pyo3Docker, compose: Pyo3ComposeFile, project_name: &str) -> Self {
        Pyo3ComposeProject {
            docker: docker.0,
            compose,
            project_name: project_name.to_string(),
        }
    }

    /// Get the project name.
    ///
    /// Returns:
    ///     str: Project name
    #[getter]
    pub fn get_project_name(&self) -> String {
        self.project_name.clone()
    }

    /// Bring up the compose project.
    ///
    /// Creates networks, volumes, and containers defined in the compose file,
    /// then starts the containers.
    ///
    /// Args:
    ///     detach: Run containers in the background (default: True)
    ///
    /// Returns:
    ///     dict: Results including created network IDs, volume names, and container IDs
    ///
    /// Raises:
    ///     RuntimeError: If any operation fails
    #[pyo3(signature = (detach=None))]
    pub fn up(&self, py: Python<'_>, detach: Option<bool>) -> PyResult<Py<PyAny>> {
        let _detach = detach.unwrap_or(true);
        let result = __compose_up(&self.docker, &self.compose, &self.project_name)?;
        pythonize(py, &result)
            .map(|bound| bound.unbind())
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// Bring down the compose project.
    ///
    /// Stops and removes containers, and optionally removes networks and volumes.
    ///
    /// Args:
    ///     remove_volumes: Also remove named volumes (default: False)
    ///     remove_networks: Also remove networks (default: True)
    ///     timeout: Timeout in seconds for stopping containers (default: 10)
    ///
    /// Returns:
    ///     dict: Results including stopped/removed container IDs, network IDs, volume names
    ///
    /// Raises:
    ///     RuntimeError: If any operation fails
    #[pyo3(signature = (remove_volumes=None, remove_networks=None, timeout=None))]
    pub fn down(
        &self,
        py: Python<'_>,
        remove_volumes: Option<bool>,
        remove_networks: Option<bool>,
        timeout: Option<u64>,
    ) -> PyResult<Py<PyAny>> {
        let remove_volumes = remove_volumes.unwrap_or(false);
        let remove_networks = remove_networks.unwrap_or(true);
        let timeout = timeout.unwrap_or(10);

        let result = __compose_down(
            &self.docker,
            &self.compose,
            &self.project_name,
            remove_volumes,
            remove_networks,
            timeout,
        )?;
        pythonize(py, &result)
            .map(|bound| bound.unbind())
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// List containers for this project.
    ///
    /// Returns:
    ///     list[str]: List of container IDs belonging to this project
    pub fn ps(&self) -> PyResult<Vec<String>> {
        __compose_ps(&self.docker, &self.project_name)
    }

    /// Start all stopped containers in the project.
    ///
    /// Starts containers that were previously stopped without recreating them.
    ///
    /// Returns:
    ///     list[str]: List of container IDs that were started
    ///
    /// Raises:
    ///     RuntimeError: If any container fails to start
    pub fn start(&self) -> PyResult<Vec<String>> {
        __compose_start(&self.docker, &self.project_name)
    }

    /// Stop all running containers in the project.
    ///
    /// Stops containers without removing them.
    ///
    /// Args:
    ///     timeout: Timeout in seconds to wait for containers to stop (default: 10)
    ///
    /// Returns:
    ///     list[str]: List of container IDs that were stopped
    ///
    /// Raises:
    ///     RuntimeError: If any container fails to stop
    #[pyo3(signature = (timeout=None))]
    pub fn stop(&self, timeout: Option<u64>) -> PyResult<Vec<String>> {
        let timeout = timeout.unwrap_or(10);
        __compose_stop(&self.docker, &self.project_name, timeout)
    }

    /// Restart all containers in the project.
    ///
    /// Restarts all containers without recreating them.
    ///
    /// Args:
    ///     timeout: Timeout in seconds to wait for containers to stop before restart (default: 10)
    ///
    /// Returns:
    ///     list[str]: List of container IDs that were restarted
    ///
    /// Raises:
    ///     RuntimeError: If any container fails to restart
    #[pyo3(signature = (timeout=None))]
    pub fn restart(&self, timeout: Option<u64>) -> PyResult<Vec<String>> {
        let timeout = timeout.unwrap_or(10);
        __compose_restart(&self.docker, &self.project_name, timeout)
    }

    /// Pause all running containers in the project.
    ///
    /// Pauses all processes within the containers.
    ///
    /// Returns:
    ///     list[str]: List of container IDs that were paused
    ///
    /// Raises:
    ///     RuntimeError: If any container fails to pause
    pub fn pause(&self) -> PyResult<Vec<String>> {
        __compose_pause(&self.docker, &self.project_name)
    }

    /// Unpause all paused containers in the project.
    ///
    /// Resumes all processes within the containers.
    ///
    /// Returns:
    ///     list[str]: List of container IDs that were unpaused
    ///
    /// Raises:
    ///     RuntimeError: If any container fails to unpause
    pub fn unpause(&self) -> PyResult<Vec<String>> {
        __compose_unpause(&self.docker, &self.project_name)
    }

    /// Pull images for all services in the project.
    ///
    /// Pulls the images specified in the compose file for all services that
    /// have an `image` field defined.
    ///
    /// Returns:
    ///     list[str]: List of images that were pulled
    ///
    /// Raises:
    ///     RuntimeError: If any image fails to pull
    pub fn pull(&self) -> PyResult<Vec<String>> {
        __compose_pull(&self.docker, &self.compose)
    }

    /// Build images for all services in the project that have a build config.
    ///
    /// Builds images for services that have a `build` field defined in the compose file.
    /// Services with only an `image` field are skipped.
    ///
    /// Args:
    ///     no_cache: Do not use cache when building (default: False)
    ///     pull: Always pull newer versions of base images (default: False)
    ///
    /// Returns:
    ///     list[str]: List of services that were built
    ///
    /// Raises:
    ///     RuntimeError: If any build fails
    #[pyo3(signature = (no_cache=None, pull=None))]
    pub fn build(&self, no_cache: Option<bool>, pull: Option<bool>) -> PyResult<Vec<String>> {
        let no_cache = no_cache.unwrap_or(false);
        let pull = pull.unwrap_or(false);
        __compose_build(
            &self.docker,
            &self.compose,
            &self.project_name,
            no_cache,
            pull,
        )
    }

    /// Push images for all services in the project.
    ///
    /// Pushes images for services that have an `image` field defined to their registry.
    ///
    /// Returns:
    ///     list[str]: List of images that were pushed
    ///
    /// Raises:
    ///     RuntimeError: If any image fails to push
    pub fn push(&self) -> PyResult<Vec<String>> {
        __compose_push(&self.docker, &self.compose)
    }

    /// Get detailed information about containers in the project.
    ///
    /// Returns detailed information about each container including ID, name, state,
    /// service name, and image.
    ///
    /// Returns:
    ///     list[dict]: List of container info dicts with keys:
    ///         - id: Container ID
    ///         - name: Container name
    ///         - service: Service name from compose file
    ///         - state: Container state (running, stopped, etc.)
    ///         - status: Container status message
    ///         - image: Image used by the container
    ///
    /// Raises:
    ///     RuntimeError: If container information cannot be retrieved
    pub fn ps_detailed(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let result = __compose_ps_detailed(&self.docker, &self.project_name)?;
        pythonize(py, &result)
            .map(|bound| bound.unbind())
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// Get logs from all containers in the project.
    ///
    /// Collects logs from all containers belonging to this project.
    ///
    /// Args:
    ///     service: Only get logs from this service (optional)
    ///     tail: Number of lines to show from the end of logs (optional)
    ///     timestamps: Include timestamps in output (default: False)
    ///
    /// Returns:
    ///     dict[str, str]: Mapping of container ID to its logs
    ///
    /// Raises:
    ///     RuntimeError: If logs cannot be retrieved
    #[pyo3(signature = (service=None, tail=None, timestamps=None))]
    pub fn logs(
        &self,
        py: Python<'_>,
        service: Option<&str>,
        tail: Option<usize>,
        timestamps: Option<bool>,
    ) -> PyResult<Py<PyAny>> {
        let timestamps = timestamps.unwrap_or(false);
        let result = __compose_logs(&self.docker, &self.project_name, service, tail, timestamps)?;
        pythonize(py, &result)
            .map(|bound| bound.unbind())
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// Get the compose configuration as a dictionary.
    ///
    /// Returns the parsed compose file configuration, useful for inspecting
    /// the services, networks, and volumes defined in the project.
    ///
    /// Returns:
    ///     dict: The compose file configuration
    pub fn config(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        pythonize(py, &self.compose)
            .map(|bound| bound.unbind())
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// Get running processes from all containers in the project.
    ///
    /// Returns process information from all running containers in the project.
    ///
    /// Args:
    ///     ps_args: Arguments to pass to ps command (e.g., "aux")
    ///
    /// Returns:
    ///     dict[str, dict]: Mapping of container ID to its process info
    ///
    /// Raises:
    ///     RuntimeError: If process information cannot be retrieved
    #[pyo3(signature = (ps_args=None))]
    pub fn top(&self, py: Python<'_>, ps_args: Option<&str>) -> PyResult<Py<PyAny>> {
        let result = __compose_top(&self.docker, &self.project_name, ps_args)?;
        pythonize(py, &result)
            .map(|bound| bound.unbind())
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    /// Execute a command in a running service container.
    ///
    /// Runs a command in the first running container of the specified service,
    /// similar to `docker-compose exec`.
    ///
    /// Args:
    ///     service: Name of the service to execute the command in
    ///     command: Command to execute as a list (e.g., ["ls", "-la"])
    ///     user: User to run the command as (optional)
    ///     workdir: Working directory inside the container (optional)
    ///     env: Environment variables as a list (e.g., ["VAR=value"]) (optional)
    ///     privileged: Give extended privileges to the command (default: False)
    ///     tty: Allocate a pseudo-TTY (default: False)
    ///
    /// Returns:
    ///     str: Output from the executed command
    ///
    /// Raises:
    ///     RuntimeError: If no running container is found for the service
    ///     RuntimeError: If command execution fails
    #[pyo3(signature = (service, command, user=None, workdir=None, env=None, privileged=None, tty=None))]
    pub fn exec(
        &self,
        service: &str,
        command: Vec<String>,
        user: Option<&str>,
        workdir: Option<&str>,
        env: Option<Vec<String>>,
        privileged: Option<bool>,
        tty: Option<bool>,
    ) -> PyResult<String> {
        let privileged = privileged.unwrap_or(false);
        let tty = tty.unwrap_or(false);
        __compose_exec(
            &self.docker,
            &self.project_name,
            service,
            command,
            user,
            workdir,
            env,
            privileged,
            tty,
        )
    }

    /// Run a one-off command in a new container for a service.
    ///
    /// Creates a new container based on the service configuration, runs the
    /// specified command, and optionally removes the container afterward.
    /// Similar to `docker-compose run`.
    ///
    /// Args:
    ///     service: Name of the service to run
    ///     command: Command to execute as a list (e.g., ["python", "script.py"]).
    ///              If not provided, uses the service's default command.
    ///     user: User to run the command as (optional)
    ///     workdir: Working directory inside the container (optional)
    ///     env: Additional environment variables as a list (e.g., ["VAR=value"])
    ///     rm: Remove the container after exit (default: True)
    ///     detach: Run container in the background (default: False)
    ///
    /// Returns:
    ///     dict: Result containing container_id and output (if not detached)
    ///
    /// Raises:
    ///     RuntimeError: If the service is not found in the compose file
    ///     RuntimeError: If container creation or execution fails
    #[pyo3(signature = (service, command=None, user=None, workdir=None, env=None, rm=None, detach=None))]
    pub fn run(
        &self,
        py: Python<'_>,
        service: &str,
        command: Option<Vec<String>>,
        user: Option<&str>,
        workdir: Option<&str>,
        env: Option<Vec<String>>,
        rm: Option<bool>,
        detach: Option<bool>,
    ) -> PyResult<Py<PyAny>> {
        let rm = rm.unwrap_or(true);
        let detach = detach.unwrap_or(false);
        let result = __compose_run(
            &self.docker,
            &self.compose,
            &self.project_name,
            service,
            command,
            user,
            workdir,
            env,
            rm,
            detach,
        )?;
        pythonize(py, &result)
            .map(|bound| bound.unbind())
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }
}

// Helper function to generate resource names with project prefix
fn resource_name(project_name: &str, resource: &str) -> String {
    format!("{}_{}", project_name, resource)
}

// Helper to convert environment from compose format to Docker format
fn env_to_vec(env: &Option<EnvironmentVars>) -> Vec<String> {
    match env {
        Some(EnvironmentVars::List(list)) => list.clone(),
        Some(EnvironmentVars::Map(map)) => map
            .iter()
            .map(|(k, v)| match v {
                Some(StringOrInt::String(s)) => format!("{}={}", k, s),
                Some(StringOrInt::Int(i)) => format!("{}={}", k, i),
                None => k.clone(),
            })
            .collect(),
        None => vec![],
    }
}

// Helper to convert command from compose format
fn command_to_vec(cmd: &Option<StringOrList>) -> Option<Vec<String>> {
    match cmd {
        Some(StringOrList::String(s)) => {
            Some(vec!["/bin/sh".to_string(), "-c".to_string(), s.clone()])
        }
        Some(StringOrList::List(list)) => Some(list.clone()),
        None => None,
    }
}

// Helper to convert labels from compose format
fn labels_to_map(
    labels: &Option<Labels>,
    project_name: &str,
    service_name: &str,
) -> HashMap<String, String> {
    let mut result = HashMap::new();
    // Add compose project labels for tracking
    result.insert(
        "com.docker.compose.project".to_string(),
        project_name.to_string(),
    );
    result.insert(
        "com.docker.compose.service".to_string(),
        service_name.to_string(),
    );

    match labels {
        Some(Labels::List(list)) => {
            for item in list {
                if let Some((k, v)) = item.split_once('=') {
                    result.insert(k.to_string(), v.to_string());
                }
            }
        }
        Some(Labels::Map(map)) => {
            for (k, v) in map {
                result.insert(k.clone(), v.clone());
            }
        }
        None => {}
    }
    result
}

#[tokio::main]
async fn __compose_up(
    docker: &docker_api::Docker,
    compose: &Pyo3ComposeFile,
    project_name: &str,
) -> PyResult<ComposeUpResult> {
    let mut result = ComposeUpResult {
        networks: vec![],
        volumes: vec![],
        containers: vec![],
    };

    // 1. Create networks
    let networks = Networks::new(docker.clone());
    for (name, network_config) in &compose.networks {
        let network_name = resource_name(project_name, name);

        // Check if network already exists
        let existing: Vec<docker_api::models::Network> =
            networks
                .list(&Default::default())
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to list networks: {}", e)))?;

        if existing
            .iter()
            .any(|n| n.name.as_ref() == Some(&network_name))
        {
            result.networks.push(network_name);
            continue;
        }

        let mut opts = NetworkCreateOpts::builder(&network_name);

        if let Some(Some(config)) = network_config.as_ref().map(|c| Some(c)) {
            if let Some(driver) = &config.driver {
                opts = opts.driver(driver.as_str());
            }
            if let Some(internal) = config.internal {
                opts = opts.internal(internal);
            }
            if let Some(attachable) = config.attachable {
                opts = opts.attachable(attachable);
            }
        }

        // Add project label
        opts = opts.labels([("com.docker.compose.project", project_name)]);

        let network = networks.create(&opts.build()).await.map_err(|e| {
            PyRuntimeError::new_err(format!("Failed to create network {}: {}", network_name, e))
        })?;

        result.networks.push(network.id().to_string());
    }

    // If no networks defined, create default network
    if compose.networks.is_empty() {
        let default_network_name = format!("{}_default", project_name);
        let existing: Vec<docker_api::models::Network> =
            networks
                .list(&Default::default())
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to list networks: {}", e)))?;

        if !existing
            .iter()
            .any(|n| n.name.as_ref() == Some(&default_network_name))
        {
            let opts = NetworkCreateOpts::builder(&default_network_name)
                .labels([("com.docker.compose.project", project_name)])
                .build();

            let network = networks.create(&opts).await.map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to create default network: {}", e))
            })?;

            result.networks.push(network.id().to_string());
        }
    }

    // 2. Create volumes
    let volumes = Volumes::new(docker.clone());
    for (name, volume_config) in &compose.volumes {
        let volume_name = resource_name(project_name, name);

        // Check if volume already exists
        let existing = volumes
            .list(&Default::default())
            .await
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to list volumes: {}", e)))?;

        if let Some(vols) = existing.volumes {
            if vols.iter().any(|v| v.name == volume_name) {
                result.volumes.push(volume_name);
                continue;
            }
        }

        let mut opts = VolumeCreateOpts::builder();
        opts = opts.name(&volume_name);

        if let Some(Some(config)) = volume_config.as_ref().map(|c| Some(c)) {
            if let Some(driver) = &config.driver {
                opts = opts.driver(driver.as_str());
            }
        }

        // Add project label
        opts = opts.labels([("com.docker.compose.project", project_name)]);

        volumes.create(&opts.build()).await.map_err(|e| {
            PyRuntimeError::new_err(format!("Failed to create volume {}: {}", volume_name, e))
        })?;

        result.volumes.push(volume_name);
    }

    // 3. Create and start containers (in order, respecting depends_on)
    let containers = Containers::new(docker.clone());
    let service_order = get_service_order(compose);

    for service_name in &service_order {
        if let Some(service) = compose.services.get(service_name) {
            // Skip if no image specified and no build config
            let image = match &service.image {
                Some(img) => img.clone(),
                None => {
                    // If build is specified, we'd need to build - for now, skip
                    if service.build.is_some() {
                        continue; // Skip services that need building
                    }
                    continue;
                }
            };

            let container_name = service
                .container_name
                .clone()
                .unwrap_or_else(|| resource_name(project_name, service_name));

            // Check if container already exists
            let existing: Vec<docker_api::models::ContainerSummary> = containers
                .list(&ContainerListOpts::builder().all(true).build())
                .await
                .map_err(|e| {
                    PyRuntimeError::new_err(format!("Failed to list containers: {}", e))
                })?;

            let existing_container = existing.iter().find(|c| {
                c.names.as_ref().map_or(false, |names| {
                    names
                        .iter()
                        .any(|n| n.trim_start_matches('/') == container_name)
                })
            });

            if let Some(existing) = existing_container {
                // Container exists, just start it if not running
                if existing.state.as_ref() != Some(&"running".to_string()) {
                    let container = containers.get(existing.id.as_ref().unwrap());
                    container.start().await.map_err(|e| {
                        PyRuntimeError::new_err(format!(
                            "Failed to start container {}: {}",
                            container_name, e
                        ))
                    })?;
                }
                result
                    .containers
                    .push(existing.id.clone().unwrap_or_default());
                continue;
            }

            // Build container create options
            let mut opts = ContainerCreateOpts::builder()
                .image(&image)
                .name(&container_name);

            // Set labels for compose project tracking
            let labels = labels_to_map(&service.labels, project_name, service_name);
            let labels_ref: HashMap<&str, &str> = labels
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            opts = opts.labels(labels_ref);

            // Set environment
            let env = env_to_vec(&service.environment);
            if !env.is_empty() {
                let env_refs: Vec<&str> = env.iter().map(|s| s.as_str()).collect();
                opts = opts.env(env_refs);
            }

            // Set command
            if let Some(cmd) = command_to_vec(&service.command) {
                let cmd_refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
                opts = opts.command(cmd_refs);
            }

            // Set working directory
            if let Some(wd) = &service.working_dir {
                opts = opts.working_dir(wd.as_str());
            }

            // Set user
            if let Some(user) = &service.user {
                opts = opts.user(user.as_str());
            }

            // Set tty
            if let Some(tty) = service.tty {
                opts = opts.tty(tty);
            }

            // Set stdin_open
            if let Some(stdin) = service.stdin_open {
                opts = opts.attach_stdin(stdin);
            }

            // Set privileged
            if let Some(priv_mode) = service.privileged {
                opts = opts.privileged(priv_mode);
            }

            // Set hostname
            if let Some(hostname) = &service.hostname {
                // Note: hostname is not directly supported by ContainerCreateOpts in the same way
                // We'll skip this for now
                let _ = hostname;
            }

            // Set network mode - use project default network
            let default_network = format!("{}_default", project_name);
            opts = opts.network_mode(&default_network);

            // Handle port mappings
            if let Some(ports) = &service.ports {
                for port_mapping in ports {
                    match port_mapping {
                        PortMapping::Simple(s) => {
                            // Parse "host:container" or "container"
                            if let Some((host_port, container_port)) = s.split_once(':') {
                                let container_port =
                                    container_port.split('/').next().unwrap_or(container_port);
                                if let (Ok(hp), Ok(cp)) =
                                    (host_port.parse::<u32>(), container_port.parse::<u32>())
                                {
                                    opts = opts.expose(PublishPort::tcp(cp), hp);
                                }
                            }
                        }
                        PortMapping::Int(p) => {
                            opts = opts.expose(PublishPort::tcp(*p as u32), *p as u32);
                        }
                        PortMapping::Full(config) => {
                            if let (Some(target), Some(published)) =
                                (&config.target, &config.published)
                            {
                                let host_port = match published {
                                    StringOrInt::String(s) => {
                                        s.parse::<u32>().unwrap_or(*target as u32)
                                    }
                                    StringOrInt::Int(i) => *i as u32,
                                };
                                let protocol = config.protocol.as_deref().unwrap_or("tcp");
                                let publish_port = match protocol {
                                    "udp" => PublishPort::udp(*target as u32),
                                    _ => PublishPort::tcp(*target as u32),
                                };
                                opts = opts.expose(publish_port, host_port);
                            }
                        }
                    }
                }
            }

            // Handle volume mounts
            if let Some(vol_mounts) = &service.volumes {
                let mut volume_bindings: Vec<String> = vec![];
                for mount in vol_mounts {
                    match mount {
                        VolumeMount::Simple(s) => {
                            // Check if it's a named volume (starts with volume name from compose)
                            let parts: Vec<&str> = s.split(':').collect();
                            if parts.len() >= 2 {
                                let source = parts[0];
                                let target = parts[1];
                                let mode = parts.get(2).unwrap_or(&"rw");

                                // Check if source is a named volume
                                if compose.volumes.contains_key(source) {
                                    let vol_name = resource_name(project_name, source);
                                    volume_bindings
                                        .push(format!("{}:{}:{}", vol_name, target, mode));
                                } else {
                                    // It's a bind mount
                                    volume_bindings.push(s.clone());
                                }
                            }
                        }
                        VolumeMount::Full(config) => {
                            if let (Some(source), Some(target)) = (&config.source, &config.target) {
                                let mode = if config.read_only.unwrap_or(false) {
                                    "ro"
                                } else {
                                    "rw"
                                };

                                // Check if source is a named volume
                                if compose.volumes.contains_key(source) {
                                    let vol_name = resource_name(project_name, source);
                                    volume_bindings
                                        .push(format!("{}:{}:{}", vol_name, target, mode));
                                } else {
                                    volume_bindings.push(format!("{}:{}:{}", source, target, mode));
                                }
                            }
                        }
                    }
                }
                if !volume_bindings.is_empty() {
                    let vol_refs: Vec<&str> = volume_bindings.iter().map(|s| s.as_str()).collect();
                    opts = opts.volumes(vol_refs);
                }
            }

            // Set restart policy
            if let Some(restart) = &service.restart {
                let (policy, retries) = match restart.as_str() {
                    "always" => ("always", 0u64),
                    "unless-stopped" => ("unless-stopped", 0u64),
                    "on-failure" => ("on-failure", 3u64),
                    _ => ("no", 0u64),
                };
                opts = opts.restart_policy(policy, retries);
            }

            // Create the container
            let container = containers.create(&opts.build()).await.map_err(|e| {
                PyRuntimeError::new_err(format!(
                    "Failed to create container {}: {}",
                    container_name, e
                ))
            })?;

            // Start the container
            container.start().await.map_err(|e| {
                PyRuntimeError::new_err(format!(
                    "Failed to start container {}: {}",
                    container_name, e
                ))
            })?;

            result.containers.push(container.id().to_string());
        }
    }

    Ok(result)
}

#[tokio::main]
async fn __compose_down(
    docker: &docker_api::Docker,
    compose: &Pyo3ComposeFile,
    project_name: &str,
    remove_volumes: bool,
    remove_networks: bool,
    timeout: u64,
) -> PyResult<ComposeDownResult> {
    let mut result = ComposeDownResult {
        stopped_containers: vec![],
        removed_containers: vec![],
        removed_networks: vec![],
        removed_volumes: vec![],
    };

    // 1. Stop and remove containers
    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    // Find containers belonging to this project
    let project_containers: Vec<_> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                labels.get("com.docker.compose.project") == Some(&project_name.to_string())
            })
        })
        .collect();

    for container_info in project_containers {
        if let Some(id) = &container_info.id {
            let container = containers.get(id);

            // Stop if running
            if container_info.state.as_ref() == Some(&"running".to_string()) {
                let stop_opts = ContainerStopOpts::builder()
                    .wait(std::time::Duration::from_secs(timeout))
                    .build();

                if container.stop(&stop_opts).await.is_ok() {
                    result.stopped_containers.push(id.clone());
                }
            }

            // Remove container
            if container.delete().await.is_ok() {
                result.removed_containers.push(id.clone());
            }
        }
    }

    // 2. Remove networks (if requested)
    if remove_networks {
        let networks = Networks::new(docker.clone());
        let network_list: Vec<docker_api::models::Network> = networks
            .list(&Default::default())
            .await
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to list networks: {}", e)))?;

        // Find networks belonging to this project
        let project_networks: Vec<_> = network_list
            .iter()
            .filter(|n| {
                n.labels.as_ref().map_or(false, |labels| {
                    labels.get("com.docker.compose.project") == Some(&project_name.to_string())
                })
            })
            .collect();

        for network_info in project_networks {
            if let Some(id) = &network_info.id {
                let network = networks.get(id);
                if network.delete().await.is_ok() {
                    result.removed_networks.push(id.clone());
                }
            }
        }
    }

    // 3. Remove volumes (if requested)
    if remove_volumes {
        let volumes = Volumes::new(docker.clone());

        for name in compose.volumes.keys() {
            let volume_name = resource_name(project_name, name);
            let volume = volumes.get(&volume_name);
            if volume.delete().await.is_ok() {
                result.removed_volumes.push(volume_name);
            }
        }
    }

    Ok(result)
}

#[tokio::main]
async fn __compose_ps(docker: &docker_api::Docker, project_name: &str) -> PyResult<Vec<String>> {
    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    let project_containers: Vec<String> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                labels.get("com.docker.compose.project") == Some(&project_name.to_string())
            })
        })
        .filter_map(|c| c.id.clone())
        .collect();

    Ok(project_containers)
}

#[tokio::main]
async fn __compose_start(docker: &docker_api::Docker, project_name: &str) -> PyResult<Vec<String>> {
    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    // Find containers belonging to this project that are stopped
    let project_containers: Vec<_> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                labels.get("com.docker.compose.project") == Some(&project_name.to_string())
            })
        })
        .filter(|c| c.state.as_ref() != Some(&"running".to_string()))
        .collect();

    let mut started = Vec::new();

    for container_info in project_containers {
        if let Some(id) = &container_info.id {
            let container = containers.get(id);
            if container.start().await.is_ok() {
                started.push(id.clone());
            }
        }
    }

    Ok(started)
}

#[tokio::main]
async fn __compose_stop(
    docker: &docker_api::Docker,
    project_name: &str,
    timeout: u64,
) -> PyResult<Vec<String>> {
    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    // Find running containers belonging to this project
    let project_containers: Vec<_> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                labels.get("com.docker.compose.project") == Some(&project_name.to_string())
            })
        })
        .filter(|c| c.state.as_ref() == Some(&"running".to_string()))
        .collect();

    let mut stopped = Vec::new();

    for container_info in project_containers {
        if let Some(id) = &container_info.id {
            let container = containers.get(id);
            let stop_opts = ContainerStopOpts::builder()
                .wait(std::time::Duration::from_secs(timeout))
                .build();

            if container.stop(&stop_opts).await.is_ok() {
                stopped.push(id.clone());
            }
        }
    }

    Ok(stopped)
}

#[tokio::main]
async fn __compose_restart(
    docker: &docker_api::Docker,
    project_name: &str,
    timeout: u64,
) -> PyResult<Vec<String>> {
    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    // Find containers belonging to this project
    let project_containers: Vec<_> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                labels.get("com.docker.compose.project") == Some(&project_name.to_string())
            })
        })
        .collect();

    let mut restarted = Vec::new();

    for container_info in project_containers {
        if let Some(id) = &container_info.id {
            let container = containers.get(id);
            let restart_opts = ContainerRestartOpts::builder()
                .wait(std::time::Duration::from_secs(timeout))
                .build();

            if container.restart(&restart_opts).await.is_ok() {
                restarted.push(id.clone());
            }
        }
    }

    Ok(restarted)
}

#[tokio::main]
async fn __compose_pause(docker: &docker_api::Docker, project_name: &str) -> PyResult<Vec<String>> {
    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    // Find running containers belonging to this project
    let project_containers: Vec<_> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                labels.get("com.docker.compose.project") == Some(&project_name.to_string())
            })
        })
        .filter(|c| c.state.as_ref() == Some(&"running".to_string()))
        .collect();

    let mut paused = Vec::new();

    for container_info in project_containers {
        if let Some(id) = &container_info.id {
            let container = containers.get(id);
            if container.pause().await.is_ok() {
                paused.push(id.clone());
            }
        }
    }

    Ok(paused)
}

#[tokio::main]
async fn __compose_unpause(
    docker: &docker_api::Docker,
    project_name: &str,
) -> PyResult<Vec<String>> {
    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    // Find paused containers belonging to this project
    let project_containers: Vec<_> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                labels.get("com.docker.compose.project") == Some(&project_name.to_string())
            })
        })
        .filter(|c| c.state.as_ref() == Some(&"paused".to_string()))
        .collect();

    let mut unpaused = Vec::new();

    for container_info in project_containers {
        if let Some(id) = &container_info.id {
            let container = containers.get(id);
            if container.unpause().await.is_ok() {
                unpaused.push(id.clone());
            }
        }
    }

    Ok(unpaused)
}

#[tokio::main]
async fn __compose_pull(
    docker: &docker_api::Docker,
    compose: &Pyo3ComposeFile,
) -> PyResult<Vec<String>> {
    use futures_util::StreamExt;

    let images = Images::new(docker.clone());
    let mut pulled = Vec::new();

    for (_service_name, service) in &compose.services {
        // Only pull if there's an image field (not just build)
        if let Some(image) = &service.image {
            let pull_opts = PullOpts::builder()
                .image(image.as_str())
                .auth(RegistryAuth::builder().build())
                .build();

            let mut stream = images.pull(&pull_opts);
            let mut success = true;

            while let Some(result) = stream.next().await {
                if result.is_err() {
                    success = false;
                    break;
                }
            }

            if success {
                pulled.push(image.clone());
            }
        }
    }

    Ok(pulled)
}

#[tokio::main]
async fn __compose_build(
    docker: &docker_api::Docker,
    compose: &Pyo3ComposeFile,
    project_name: &str,
    no_cache: bool,
    pull: bool,
) -> PyResult<Vec<String>> {
    use futures_util::StreamExt;

    let images = Images::new(docker.clone());
    let mut built = Vec::new();

    for (service_name, service) in &compose.services {
        // Only build if there's a build field
        if let Some(build_config) = &service.build {
            let build_context = match build_config {
                ComposeBuild::Simple(path) => path.clone(),
                ComposeBuild::Full(config) => {
                    config.context.clone().unwrap_or_else(|| ".".to_string())
                }
            };

            let dockerfile = match build_config {
                ComposeBuild::Simple(_) => None,
                ComposeBuild::Full(config) => config.dockerfile.clone(),
            };

            // Determine the tag for the built image
            let tag = service
                .image
                .clone()
                .unwrap_or_else(|| format!("{}_{}", project_name, service_name));

            let mut build_opts = ImageBuildOpts::builder(&build_context);
            build_opts = build_opts.tag(&tag);

            if let Some(df) = &dockerfile {
                build_opts = build_opts.dockerfile(df);
            }

            if no_cache {
                build_opts = build_opts.nocahe(true);
            }

            if pull {
                build_opts = build_opts.pull("true");
            }

            let mut stream = images.build(&build_opts.build());
            let mut success = true;

            while let Some(result) = stream.next().await {
                if result.is_err() {
                    success = false;
                    break;
                }
            }

            if success {
                built.push(service_name.clone());
            }
        }
    }

    Ok(built)
}

#[tokio::main]
async fn __compose_push(
    docker: &docker_api::Docker,
    compose: &Pyo3ComposeFile,
) -> PyResult<Vec<String>> {
    let images = Images::new(docker.clone());
    let mut pushed = Vec::new();

    for (_service_name, service) in &compose.services {
        // Only push if there's an image field
        if let Some(image_name) = &service.image {
            let image = images.get(image_name);
            let push_opts = ImagePushOpts::builder()
                .auth(RegistryAuth::builder().build())
                .build();

            if image.push(&push_opts).await.is_ok() {
                pushed.push(image_name.clone());
            }
        }
    }

    Ok(pushed)
}

/// Container info for ps_detailed
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContainerInfo {
    id: String,
    name: String,
    service: String,
    state: String,
    status: String,
    image: String,
}

#[tokio::main]
async fn __compose_ps_detailed(
    docker: &docker_api::Docker,
    project_name: &str,
) -> PyResult<Vec<ContainerInfo>> {
    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    let project_containers: Vec<ContainerInfo> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                labels.get("com.docker.compose.project") == Some(&project_name.to_string())
            })
        })
        .map(|c| {
            let service = c
                .labels
                .as_ref()
                .and_then(|l| l.get("com.docker.compose.service"))
                .cloned()
                .unwrap_or_default();

            let name = c
                .names
                .as_ref()
                .and_then(|n| n.first())
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default();

            ContainerInfo {
                id: c.id.clone().unwrap_or_default(),
                name,
                service,
                state: c.state.clone().unwrap_or_default(),
                status: c.status.clone().unwrap_or_default(),
                image: c.image.clone().unwrap_or_default(),
            }
        })
        .collect();

    Ok(project_containers)
}

#[tokio::main]
async fn __compose_logs(
    docker: &docker_api::Docker,
    project_name: &str,
    service_filter: Option<&str>,
    tail: Option<usize>,
    timestamps: bool,
) -> PyResult<HashMap<String, String>> {
    use futures_util::StreamExt;

    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    let project_containers: Vec<_> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                let matches_project =
                    labels.get("com.docker.compose.project") == Some(&project_name.to_string());
                let matches_service = service_filter.map_or(true, |svc| {
                    labels.get("com.docker.compose.service") == Some(&svc.to_string())
                });
                matches_project && matches_service
            })
        })
        .collect();

    let mut logs_map: HashMap<String, String> = HashMap::new();

    for container_info in project_containers {
        if let Some(id) = &container_info.id {
            let container = containers.get(id);

            let mut log_opts = LogsOpts::builder();
            log_opts = log_opts.stdout(true);
            log_opts = log_opts.stderr(true);

            if let Some(n) = tail {
                log_opts = log_opts.n_lines(n);
            }

            if timestamps {
                log_opts = log_opts.timestamps(true);
            }

            let log_stream = container.logs(&log_opts.build());
            let log_chunks: Vec<Vec<u8>> = log_stream
                .map(|chunk| match chunk {
                    Ok(chunk) => chunk.to_vec(),
                    Err(_) => vec![],
                })
                .collect()
                .await;

            let log_bytes: Vec<u8> = log_chunks.into_iter().flatten().collect();
            let log_str = String::from_utf8_lossy(&log_bytes).to_string();

            let name = container_info
                .names
                .as_ref()
                .and_then(|n| n.first())
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_else(|| id.clone());

            logs_map.insert(name, log_str);
        }
    }

    Ok(logs_map)
}

#[tokio::main]
async fn __compose_top(
    docker: &docker_api::Docker,
    project_name: &str,
    ps_args: Option<&str>,
) -> PyResult<HashMap<String, serde_json::Value>> {
    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    // Find running containers belonging to this project
    let project_containers: Vec<_> = container_list
        .iter()
        .filter(|c| {
            c.labels.as_ref().map_or(false, |labels| {
                labels.get("com.docker.compose.project") == Some(&project_name.to_string())
            })
        })
        .filter(|c| c.state.as_ref() == Some(&"running".to_string()))
        .collect();

    let mut top_map: HashMap<String, serde_json::Value> = HashMap::new();

    for container_info in project_containers {
        if let Some(id) = &container_info.id {
            let container = containers.get(id);

            if let Ok(top_result) = container.top(ps_args).await {
                let name = container_info
                    .names
                    .as_ref()
                    .and_then(|n| n.first())
                    .map(|n| n.trim_start_matches('/').to_string())
                    .unwrap_or_else(|| id.clone());

                // Convert ContainerTop200Response to serde_json::Value
                let value = serde_json::json!({
                    "titles": top_result.titles,
                    "processes": top_result.processes
                });

                top_map.insert(name, value);
            }
        }
    }

    Ok(top_map)
}

/// Result of a compose run operation
#[derive(Debug, Clone, Serialize)]
pub struct ComposeRunResult {
    /// ID of the created container
    pub container_id: String,
    /// Output from the command (if not detached)
    pub output: Option<String>,
    /// Exit code (if waited for)
    pub exit_code: Option<i64>,
}

#[tokio::main]
async fn __compose_exec(
    docker: &docker_api::Docker,
    project_name: &str,
    service: &str,
    command: Vec<String>,
    user: Option<&str>,
    workdir: Option<&str>,
    env: Option<Vec<String>>,
    privileged: bool,
    tty: bool,
) -> PyResult<String> {
    use futures_util::StreamExt;

    let containers = Containers::new(docker.clone());
    let container_list: Vec<docker_api::models::ContainerSummary> = containers
        .list(&ContainerListOpts::builder().all(true).build())
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to list containers: {}", e)))?;

    // Find a running container for the specified service
    let service_container = container_list.iter().find(|c| {
        c.labels.as_ref().map_or(false, |labels| {
            labels.get("com.docker.compose.project") == Some(&project_name.to_string())
                && labels.get("com.docker.compose.service") == Some(&service.to_string())
        }) && c.state.as_ref() == Some(&"running".to_string())
    });

    let container_info = service_container.ok_or_else(|| {
        PyRuntimeError::new_err(format!(
            "No running container found for service '{}' in project '{}'",
            service, project_name
        ))
    })?;

    let container_id = container_info
        .id
        .as_ref()
        .ok_or_else(|| PyRuntimeError::new_err("Container ID not found".to_string()))?;

    let container = containers.get(container_id);

    // Build exec options
    let cmd_refs: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
    let mut exec_opts = ExecCreateOpts::builder()
        .command(cmd_refs)
        .attach_stdout(true)
        .attach_stderr(true)
        .privileged(privileged)
        .tty(tty);

    if let Some(u) = user {
        exec_opts = exec_opts.user(u);
    }

    if let Some(wd) = workdir {
        exec_opts = exec_opts.working_dir(wd);
    }

    if let Some(env_vars) = &env {
        let env_refs: Vec<&str> = env_vars.iter().map(|s| s.as_str()).collect();
        exec_opts = exec_opts.env(env_refs);
    }

    // Create and start the exec instance
    let start_opts = ExecStartOpts::builder().build();
    let mut multiplexer = container
        .exec(&exec_opts.build(), &start_opts)
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to exec command: {}", e)))?;

    // Collect output
    let mut output = Vec::new();
    while let Some(chunk_result) = multiplexer.next().await {
        match chunk_result {
            Ok(chunk) => {
                output.extend_from_slice(&chunk.to_vec());
            }
            Err(_) => break,
        }
    }

    Ok(String::from_utf8_lossy(&output).to_string())
}

#[tokio::main]
async fn __compose_run(
    docker: &docker_api::Docker,
    compose: &Pyo3ComposeFile,
    project_name: &str,
    service: &str,
    command: Option<Vec<String>>,
    user: Option<&str>,
    workdir: Option<&str>,
    env: Option<Vec<String>>,
    rm: bool,
    detach: bool,
) -> PyResult<ComposeRunResult> {
    use futures_util::StreamExt;

    // Get the service configuration
    let service_config = compose.services.get(service).ok_or_else(|| {
        PyRuntimeError::new_err(format!("Service '{}' not found in compose file", service))
    })?;

    // Determine the image to use
    let image = service_config.image.as_ref().ok_or_else(|| {
        PyRuntimeError::new_err(format!(
            "Service '{}' does not have an image specified",
            service
        ))
    })?;

    // Generate a unique container name for the run
    let container_name = format!(
        "{}_{}_run_{}",
        project_name,
        service,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let containers = Containers::new(docker.clone());

    // Build container create options
    let mut opts = ContainerCreateOpts::builder()
        .image(image)
        .name(&container_name)
        .auto_remove(rm);

    // Set labels for compose project tracking
    let mut labels = HashMap::new();
    labels.insert(
        "com.docker.compose.project".to_string(),
        project_name.to_string(),
    );
    labels.insert(
        "com.docker.compose.service".to_string(),
        service.to_string(),
    );
    labels.insert("com.docker.compose.oneoff".to_string(), "True".to_string());
    let labels_ref: HashMap<&str, &str> = labels
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    opts = opts.labels(labels_ref);

    // Set command - use provided command or service default
    let cmd = command.or_else(|| command_to_vec(&service_config.command));
    if let Some(c) = &cmd {
        let cmd_refs: Vec<&str> = c.iter().map(|s| s.as_str()).collect();
        opts = opts.command(cmd_refs);
    }

    // Set environment - combine service env with provided env
    let mut all_env = env_to_vec(&service_config.environment);
    if let Some(additional_env) = &env {
        all_env.extend(additional_env.iter().cloned());
    }
    if !all_env.is_empty() {
        let env_refs: Vec<&str> = all_env.iter().map(|s| s.as_str()).collect();
        opts = opts.env(env_refs);
    }

    // Set user
    if let Some(u) = user {
        opts = opts.user(u);
    } else if let Some(u) = &service_config.user {
        opts = opts.user(u.as_str());
    }

    // Set working directory
    if let Some(wd) = workdir {
        opts = opts.working_dir(wd);
    } else if let Some(wd) = &service_config.working_dir {
        opts = opts.working_dir(wd.as_str());
    }

    // Set tty if configured
    if let Some(tty) = service_config.tty {
        opts = opts.tty(tty);
    }

    // Set stdin_open if configured
    if let Some(stdin) = service_config.stdin_open {
        opts = opts.attach_stdin(stdin);
    }

    // Set network mode - use project default network
    let default_network = format!("{}_default", project_name);
    opts = opts.network_mode(&default_network);

    // Create the container
    let container = containers.create(&opts.build()).await.map_err(|e| {
        PyRuntimeError::new_err(format!("Failed to create container for run: {}", e))
    })?;

    let container_id = container.id().to_string();

    // Start the container
    container
        .start()
        .await
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to start run container: {}", e)))?;

    if detach {
        // Return immediately for detached mode
        return Ok(ComposeRunResult {
            container_id,
            output: None,
            exit_code: None,
        });
    }

    // Wait for the container and collect output
    let log_opts = LogsOpts::builder()
        .stdout(true)
        .stderr(true)
        .follow(true)
        .build();

    let log_stream = container.logs(&log_opts);
    let log_chunks: Vec<Vec<u8>> = log_stream
        .map(|chunk| match chunk {
            Ok(chunk) => chunk.to_vec(),
            Err(_) => vec![],
        })
        .collect()
        .await;

    let log_bytes: Vec<u8> = log_chunks.into_iter().flatten().collect();
    let output = String::from_utf8_lossy(&log_bytes).to_string();

    // Wait for container to finish
    let wait_result = container.wait().await.ok();
    let exit_code = wait_result.map(|r| r.status_code);

    // Container is auto-removed if rm=true, otherwise leave it
    // auto_remove handles cleanup automatically

    Ok(ComposeRunResult {
        container_id,
        output: Some(output),
        exit_code,
    })
}

/// Get services in dependency order (topological sort)
fn get_service_order(compose: &Pyo3ComposeFile) -> Vec<String> {
    let mut result = Vec::new();
    let mut visited = std::collections::HashSet::new();

    fn visit(
        name: &str,
        compose: &Pyo3ComposeFile,
        visited: &mut std::collections::HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(name) {
            return;
        }
        visited.insert(name.to_string());

        // Visit dependencies first
        if let Some(service) = compose.services.get(name) {
            if let Some(depends) = &service.depends_on {
                match depends {
                    DependsOn::List(deps) => {
                        for dep in deps {
                            visit(dep, compose, visited, result);
                        }
                    }
                    DependsOn::Map(deps) => {
                        for dep in deps.keys() {
                            visit(dep, compose, visited, result);
                        }
                    }
                }
            }
        }

        result.push(name.to_string());
    }

    for name in compose.services.keys() {
        visit(name, compose, &mut visited, &mut result);
    }

    result
}
