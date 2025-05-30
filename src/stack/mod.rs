// pub mod definition;
// pub mod stack;  
// pub mod service;

pub mod simple_test;
pub mod service_simple;

// Temporarily disable complex modules while fixing compilation
// pub use stack::Pyo3Stack;
// pub use definition::StackDefinition;
// pub use service::ServiceBuilder;

pub use service_simple::Service as InternalService;

// Python wrapper for Service
#[pyclass(name = "Service")]
#[derive(Debug, Clone)]
pub struct Service {
    internal: InternalService,
}

#[pymethods]
impl Service {
    #[new]
    pub fn new(name: String) -> Self {
        Self {
            internal: InternalService::new(name),
        }
    }

    #[getter]
    pub fn name(&self) -> String {
        self.internal.name().to_string()
    }

    /// Set the Docker image for this service
    pub fn image(&mut self, image: String) {
        self.internal = self.internal.clone().image(image);
    }

    /// Add port mappings
    pub fn ports(&mut self, ports: Vec<String>) {
        self.internal = self.internal.clone().ports(ports);
    }

    /// Add environment variable
    pub fn env(&mut self, key: String, value: String) {
        self.internal = self.internal.clone().env(key, value);
    }

    /// Add volume mount
    pub fn volume(&mut self, volume: String) {
        self.internal = self.internal.clone().volume(volume);
    }

    /// Set command
    pub fn command(&mut self, cmd: Vec<String>) {
        self.internal = self.internal.clone().command(cmd);
    }

    /// Set working directory
    pub fn working_dir(&mut self, dir: String) {
        self.internal = self.internal.clone().working_dir(dir);
    }

    /// Add network
    pub fn network(&mut self, network: String) {
        self.internal = self.internal.clone().network(network);
    }

    /// Add dependency
    pub fn depends_on_service(&mut self, service: String) {
        self.internal = self.internal.clone().depends_on_service(service);
    }

    /// Set restart policy
    pub fn restart_policy(&mut self, policy: String) {
        self.internal = self.internal.clone().restart_policy(policy);
    }

    /// Set hostname
    pub fn hostname(&mut self, hostname: String) {
        self.internal = self.internal.clone().hostname(hostname);
    }

    /// Add label
    pub fn label(&mut self, key: String, value: String) {
        self.internal = self.internal.clone().label(key, value);
    }

    /// Set replicas
    pub fn replicas(&mut self, count: u32) {
        self.internal = self.internal.clone().replicas(count);
    }

    /// Set memory limit
    pub fn memory(&mut self, limit: String) {
        self.internal = self.internal.clone().memory(limit);
    }
    
    // BUILD CONFIGURATION METHODS
    
    /// Set build context (alternative to image)
    pub fn build_context(&mut self, context: String) {
        self.internal = self.internal.clone().build_context(context);
    }
    
    /// Set build context with dockerfile
    pub fn build_with_dockerfile(&mut self, context: String, dockerfile: String) {
        self.internal = self.internal.clone().build_with_dockerfile(context, dockerfile);
    }
    
    /// Add build argument
    pub fn build_arg(&mut self, key: String, value: String) {
        self.internal = self.internal.clone().build_arg(key, value);
    }
    
    /// Set build target
    pub fn build_target(&mut self, target: String) {
        self.internal = self.internal.clone().build_target(target);
    }
    
    /// Add cache from image
    pub fn build_cache_from(&mut self, image: String) {
        self.internal = self.internal.clone().build_cache_from(image);
    }
    
    // RESOURCE MANAGEMENT METHODS
    
    /// Set memory reservation
    pub fn memory_reservation(&mut self, limit: String) {
        self.internal = self.internal.clone().memory_reservation(limit);
    }
    
    /// Set CPU limits
    pub fn cpus(&mut self, cpus: String) {
        self.internal = self.internal.clone().cpus(cpus);
    }
    
    /// Set CPU shares
    pub fn cpu_shares(&mut self, shares: u64) {
        self.internal = self.internal.clone().cpu_shares(shares);
    }
    
    /// Set CPU quota and period
    pub fn cpu_quota(&mut self, quota: u64, period: Option<u64>) {
        self.internal = self.internal.clone().cpu_quota(quota, period);
    }
    
    // ADVANCED PORT CONFIGURATION
    
    /// Add advanced port configuration
    pub fn port_advanced(&mut self, target: u16, published: Option<u16>, protocol: Option<String>, mode: Option<String>) {
        self.internal = self.internal.clone().port_advanced(target, published, protocol, mode);
    }
    
    // ADVANCED VOLUME CONFIGURATION
    
    /// Add advanced volume configuration
    pub fn volume_advanced(&mut self, source: String, target: String, volume_type: Option<String>, read_only: bool) {
        self.internal = self.internal.clone().volume_advanced(source, target, volume_type, read_only);
    }
    
    // ENVIRONMENT FILES & SECRETS
    
    /// Add environment file
    pub fn env_file(&mut self, file: String) {
        self.internal = self.internal.clone().env_file(file);
    }
    
    /// Add secret
    pub fn secret(&mut self, secret: String) {
        self.internal = self.internal.clone().secret(secret);
    }
    
    /// Add health check
    pub fn healthcheck(&mut self, test: Vec<String>, interval: Option<String>, timeout: Option<String>, retries: Option<u32>, start_period: Option<String>) {
        self.internal = self.internal.clone().healthcheck(test, interval, timeout, retries, start_period);
    }

    /// Clone with new name
    pub fn clone_with_name(&self, new_name: String) -> Self {
        Self {
            internal: self.internal.clone_with_name(new_name),
        }
    }

    /// Create web service
    #[pyo3(name = "web_service")]
    #[staticmethod]
    pub fn web_service(name: String) -> Self {
        Self {
            internal: InternalService::web_service(name),
        }
    }

    /// Create database service
    #[pyo3(name = "database_service")]
    #[staticmethod]
    pub fn database_service(name: String) -> Self {
        Self {
            internal: InternalService::database_service(name),
        }
    }

    /// Create redis service
    #[pyo3(name = "redis_service")]
    #[staticmethod]
    pub fn redis_service(name: String) -> Self {
        Self {
            internal: InternalService::redis_service(name),
        }
    }
}

impl Service {
    pub fn internal(&self) -> &InternalService {
        &self.internal
    }
}

// Enhanced Stack class with service registration
use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::Pyo3Docker;
use std::collections::HashMap;

// mod stack_impl;  // Temporarily disabled due to compilation issues
// mod stack_simple;  // Moved implementations to pymethods block

#[derive(Debug, Clone, Default)]
pub struct StackState {
    pub containers: HashMap<String, Vec<String>>,
    pub networks: HashMap<String, String>,
    pub status: StackStatus,
}

#[derive(Debug, Clone)]
pub enum StackStatus {
    NotDeployed,
    Deploying,
    Running,
    PartiallyRunning,
    Stopped,
    Failed,
}

impl Default for StackStatus {
    fn default() -> Self {
        StackStatus::NotDeployed
    }
}

#[pyclass(name = "Stack")]
#[derive(Debug, Clone)]
pub struct Pyo3Stack {
    docker: Pyo3Docker,
    name: String,
    registered_services: HashMap<String, InternalService>,
    state: StackState,
}

#[pymethods]
impl Pyo3Stack {
    #[new]
    pub fn new(docker: Pyo3Docker, name: String) -> Self {
        Self { 
            docker, 
            name,
            registered_services: HashMap::new(),
            state: StackState::default(),
        }
    }

    #[getter]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    
    /// Register a pre-built service into this stack
    pub fn register_service(&mut self, service: Service) -> PyResult<()> {
        let service_name = service.internal().name().to_string();
        
        // Check for duplicate service names
        if self.registered_services.contains_key(&service_name) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                format!("Service '{}' already registered in stack '{}'", service_name, self.name)
            ));
        }
        
        self.registered_services.insert(service_name, service.internal().clone());
        Ok(())
    }
    
    /// Unregister a service from this stack
    pub fn unregister_service(&mut self, service_name: String) -> PyResult<bool> {
        Ok(self.registered_services.remove(&service_name).is_some())
    }
    
    /// Get list of registered service names
    pub fn get_registered_services(&self) -> Vec<String> {
        self.registered_services.keys().cloned().collect()
    }
    
    /// Get count of registered services
    pub fn service_count(&self) -> usize {
        self.registered_services.len()
    }
    
    /// Check if a service is registered
    pub fn has_service(&self, service_name: String) -> bool {
        self.registered_services.contains_key(&service_name)
    }
    
    /// Export all registered services to simplified YAML
    pub fn to_yaml(&self) -> PyResult<String> {
        use std::collections::HashMap;
        
        let mut output = String::new();
        output.push_str("version: '3.8'\n");
        output.push_str("services:\n");
        
        for (name, service) in &self.registered_services {
            output.push_str(&format!("  {}:\n", name));
            
            let config = service.to_config_map();
            
            if let Some(image) = config.get("image") {
                output.push_str(&format!("    image: {}\n", image));
            }
            
            if let Some(ports) = config.get("ports") {
                if !ports.is_empty() {
                    output.push_str("    ports:\n");
                    for port in ports.split(',') {
                        output.push_str(&format!("      - \"{}\"\n", port));
                    }
                }
            }
            
            if let Some(env) = config.get("environment") {
                if !env.is_empty() {
                    output.push_str("    environment:\n");
                    for env_var in env.split(',') {
                        if let Some((key, value)) = env_var.split_once('=') {
                            output.push_str(&format!("      {}: {}\n", key, value));
                        }
                    }
                }
            }
            
            if let Some(volumes) = config.get("volumes") {
                if !volumes.is_empty() {
                    output.push_str("    volumes:\n");
                    for volume in volumes.split(',') {
                        output.push_str(&format!("      - {}\n", volume));
                    }
                }
            }
            
            if let Some(restart) = config.get("restart") {
                output.push_str(&format!("    restart: {}\n", restart));
            }
            
            if let Some(depends_on) = config.get("depends_on") {
                if !depends_on.is_empty() {
                    output.push_str("    depends_on:\n");
                    for dep in depends_on.split(',') {
                        output.push_str(&format!("      - {}\n", dep));
                    }
                }
            }
        }
        
        Ok(output)
    }
    
    pub fn test_docker_compose_integration(&self) -> PyResult<bool> {
        // Simple test to verify docker-compose-types is working
        let yaml = r#"
version: '3.8'
services:
  web:
    image: nginx:latest
"#;
        
        let compose: docker_compose_types::Compose = serde_yaml::from_str(yaml)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("YAML parse error: {}", e)))?;
        
        Ok(compose.version.is_some())
    }
    
    // Phase 2.0 Stack Deployment Methods
    
    /// Deploy the entire stack (Phase 2.0)
    pub fn up(&mut self) -> PyResult<()> {
        // Create default network
        let network_name = format!("{}_default", self.name);
        let network = self.docker.networks().create(
            &network_name,
            None, None, None, None, None, None, None, None
        )?;
        
        // Store network ID
        self.state.networks.insert("default".to_string(), network.id());
        
        // Deploy services
        for (service_name, service) in &self.registered_services {
            let config = service.to_config_map();
            
            // Get image or skip if build-only
            let image = match config.get("image") {
                Some(img) => img.clone(),
                None => {
                    eprintln!("Service {} has no image (build not implemented), skipping", service_name);
                    continue;
                }
            };
            
            // Create container with minimal configuration
            let container_name = format!("{}_{}_1", self.name, service_name);
            
            // Use a simple container creation approach
            let container = Python::with_gil(|py| -> PyResult<_> {
                // Create minimal command list if present
                let cmd_list = if let Some(cmd_str) = config.get("command") {
                    let cmd_vec: Vec<&str> = cmd_str.split_whitespace().collect();
                    let list = pyo3::types::PyList::new(py, &cmd_vec);
                    Some(list)
                } else {
                    None
                };
                
                // Create minimal environment list
                let env_list = if let Some(env_str) = config.get("environment") {
                    let env_pairs: Vec<&str> = env_str.split(',').collect();
                    let list = pyo3::types::PyList::new(py, &env_pairs);
                    Some(list)
                } else {
                    None
                };
                
                // Call the create method with proper arguments
                self.docker.containers().create(
                    &image,          // image
                    None,            // attach_stderr
                    None,            // attach_stdin
                    None,            // attach_stdout
                    None,            // auto_remove
                    None,            // capabilities
                    cmd_list,        // command
                    None,            // cpu_shares
                    None,            // cpus
                    None,            // devices
                    None,            // entrypoint
                    env_list,        // env
                    None,            // expose
                    None,            // extra_hosts
                    None,            // labels
                    None,            // links
                    None,            // log_driver
                    None,            // memory
                    None,            // memory_swap
                    Some(&container_name), // name
                    None,            // nano_cpus
                    None,            // network_mode
                    None,            // privileged
                    None,            // publish
                    None,            // ports
                    None,            // publish_all_ports
                    None,            // restart_policy
                    None,            // security_options
                    None,            // stop_signal
                    None,            // stop_signal_num
                    None,            // stop_timeout
                    None,            // tty
                    None,            // user
                    None,            // userns_mode
                    None,            // volumes
                    None,            // volumes_from
                    config.get("working_dir").map(|s| s.as_str()) // working_dir
                )
            })?;
            
            // Start the container
            container.start()?;
            
            // Track container by getting its ID
            let container_id = container.id()?;
            self.state.containers.entry(service_name.clone())
                .or_insert_with(Vec::new)
                .push(container_id);
        }
        
        self.state.status = StackStatus::Running;
        Ok(())
    }
    
    /// Stop and remove the entire stack (Phase 2.0)
    pub fn down(&mut self) -> PyResult<()> {
        // Remove containers
        for (_, container_ids) in self.state.containers.clone() {
            for container_id in container_ids {
                let container = self.docker.containers().get(&container_id);
                // Try to stop and remove (ignore errors for cleanup)
                let _ = container.stop(None);
                let _ = container.remove(Some(true), None);
            }
        }
        self.state.containers.clear();
        
        // Remove networks
        for (_, network_id) in self.state.networks.clone() {
            let network = self.docker.networks().get(&network_id);
            let _ = network.delete();
        }
        self.state.networks.clear();
        
        self.state.status = StackStatus::NotDeployed;
        Ok(())
    }
    
    /// Get stack status (Phase 2.0)
    pub fn status(&self) -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| {
            let status_dict = PyDict::new(py);
            
            // Overall status
            let status_str = match self.state.status {
                StackStatus::NotDeployed => "not_deployed",
                StackStatus::Deploying => "deploying", 
                StackStatus::Running => "running",
                StackStatus::PartiallyRunning => "partially_running",
                StackStatus::Stopped => "stopped",
                StackStatus::Failed => "failed",
            };
            status_dict.set_item("status", status_str)?;
            
            // Service statuses
            let services_dict = PyDict::new(py);
            for service_name in self.registered_services.keys() {
                let service_dict = PyDict::new(py);
                
                if let Some(container_ids) = self.state.containers.get(service_name) {
                    service_dict.set_item("replicas", container_ids.len())?;
                    service_dict.set_item("running", container_ids.len())?; // Simplified
                } else {
                    service_dict.set_item("replicas", 0)?;
                    service_dict.set_item("running", 0)?;
                }
                
                services_dict.set_item(service_name, service_dict)?;
            }
            status_dict.set_item("services", services_dict)?;
            
            // Resource counts
            let total_containers: usize = self.state.containers.values()
                .map(|v| v.len()).sum();
            status_dict.set_item("total_containers", total_containers)?;
            status_dict.set_item("networks", self.state.networks.len())?;
            
            Ok(status_dict.into())
        })
    }
    
    /// Get logs from services (Phase 2.0) 
    pub fn logs(&self, services: Option<Vec<String>>) -> PyResult<String> {
        let target_services = services.unwrap_or_else(|| 
            self.registered_services.keys().cloned().collect()
        );
        
        let mut all_logs = Vec::new();
        
        for service_name in target_services {
            if let Some(container_ids) = self.state.containers.get(&service_name) {
                for container_id in container_ids {
                    let container = self.docker.containers().get(container_id);
                    let logs = container.logs(
                        Some(true),  // stdout
                        Some(true),  // stderr
                        Some(true),  // timestamps
                        None,        // n_lines
                        None,        // all
                        None         // since
                    );
                    all_logs.push(format!("[{}] {}", service_name, logs));
                }
            }
        }
        
        Ok(all_logs.join("\n"))
    }
    
    /// Scale a service (Phase 2.0)
    pub fn scale(&mut self, service_name: String, replicas: u32) -> PyResult<()> {
        if !self.registered_services.contains_key(&service_name) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                format!("Service '{}' not found in stack", service_name)
            ));
        }
        
        // For now, just return Ok - full implementation would scale containers
        Ok(())
    }
    
    /// Restart a service (Phase 2.0)
    pub fn restart_service(&mut self, service_name: String) -> PyResult<()> {
        if !self.registered_services.contains_key(&service_name) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                format!("Service '{}' not found in stack", service_name)
            ));
        }
        
        // For now, just return Ok - full implementation would restart containers
        Ok(())
    }
}