use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use pyo3::exceptions::{PyValueError, PyIOError, PyNotImplementedError};
use std::collections::HashMap;
use std::path::Path;
use crate::{Pyo3Docker, get_runtime};
use crate::error::DockerPyo3Error;
use super::service_simple::{Service as InternalService, BuildConfig};

/// Runtime state for tracking deployed stack resources
#[derive(Debug, Clone)]
pub struct StackState {
    /// Map of service name to container IDs
    pub containers: HashMap<String, Vec<String>>,
    /// Map of network name to network ID
    pub networks: HashMap<String, String>,
    /// Map of volume name to volume name (for named volumes)
    pub volumes: HashMap<String, String>,
    /// Overall stack status
    pub status: StackStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StackStatus {
    NotDeployed,
    Deploying,
    Running,
    PartiallyRunning,
    Stopped,
    Failed,
}

impl Default for StackState {
    fn default() -> Self {
        Self {
            containers: HashMap::new(),
            networks: HashMap::new(),
            volumes: HashMap::new(),
            status: StackStatus::NotDeployed,
        }
    }
}

/// Enhanced Stack implementation with full deployment capabilities
impl super::Pyo3Stack {
    /// Deploy the entire stack
    pub fn up(&mut self) -> PyResult<()> {
        self.state.status = StackStatus::Deploying;
        
        let runtime = get_runtime();
        runtime.block_on(async {
            // 1. Create networks first
            self.create_networks().await?;
            
            // 2. Create volumes
            self.create_volumes().await?;
            
            // 3. Deploy services in dependency order
            self.deploy_services().await?;
            
            Ok::<(), PyErr>(())
        })?;
        
        self.state.status = StackStatus::Running;
        Ok(())
    }
    
    /// Stop and remove the entire stack
    pub fn down(&mut self) -> PyResult<()> {
        let runtime = get_runtime();
        runtime.block_on(async {
            // Remove services in reverse order
            let service_names: Vec<String> = self.registered_services.keys().cloned().collect();
            for service_name in service_names.iter().rev() {
                self.remove_service_containers(service_name).await?;
            }
            
            // Remove networks and volumes
            self.remove_networks().await?;
            self.remove_volumes().await?;
            
            Ok::<(), PyErr>(())
        })?;
        
        self.state.status = StackStatus::NotDeployed;
        self.state = StackState::default();
        Ok(())
    }
    
    /// Scale a specific service
    pub fn scale(&mut self, service_name: String, replicas: u32) -> PyResult<()> {
        // Validate service exists
        if !self.registered_services.contains_key(&service_name) {
            return Err(PyValueError::new_err(
                format!("Service '{}' not found in stack", service_name)
            ));
        }
        
        // If stack is running, apply the scaling
        if self.state.status == StackStatus::Running || self.state.status == StackStatus::PartiallyRunning {
            let runtime = get_runtime();
            runtime.block_on(async {
                self.scale_service_containers(&service_name, replicas).await
            })?;
        }
        
        Ok(())
    }
    
    /// Restart a specific service
    pub fn restart_service(&mut self, service_name: String) -> PyResult<()> {
        if !self.registered_services.contains_key(&service_name) {
            return Err(PyValueError::new_err(
                format!("Service '{}' not found in stack", service_name)
            ));
        }
        
        let runtime = get_runtime();
        runtime.block_on(async {
            // Stop and remove existing containers
            self.remove_service_containers(&service_name).await?;
            
            // Redeploy the service
            self.deploy_single_service(&service_name).await
        })
    }
    
    /// Get logs from all services or specific services
    pub fn logs(&self, services: Option<Vec<String>>) -> PyResult<String> {
        let target_services = services.unwrap_or_else(|| self.registered_services.keys().cloned().collect());
        let mut all_logs = Vec::new();
        
        for service_name in target_services {
            if let Some(container_ids) = self.state.containers.get(&service_name) {
                for container_id in container_ids {
                    let container = self.docker.containers().get(container_id.clone())?;
                    if let Ok(logs) = container.logs(Some(true), Some(true), None, None, Some(true)) {
                        all_logs.push(format!("[{}] {}", service_name, logs));
                    }
                }
            }
        }
        
        Ok(all_logs.join("\n"))
    }
    
    /// Get current stack status
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
                let service_status = self.get_service_status(service_name)?;
                services_dict.set_item(service_name, service_status)?;
            }
            status_dict.set_item("services", services_dict)?;
            
            // Resource counts
            status_dict.set_item("total_containers", self.state.containers.values().map(|v| v.len()).sum::<usize>())?;
            status_dict.set_item("networks", self.state.networks.len())?;
            status_dict.set_item("volumes", self.state.volumes.len())?;
            
            Ok(status_dict.into())
        })
    }
    
    /// Load stack from docker-compose.yml file
    pub fn impl_from_file(cls: &PyType, docker: Pyo3Docker, path: String) -> PyResult<Self> {
        let file_path = Path::new(&path);
        if !file_path.exists() {
            return Err(PyIOError::new_err(format!("File not found: {}", path)));
        }
        
        let yaml_content = std::fs::read_to_string(file_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to read file: {}", e)))?;
        
        // Extract stack name from filename
        let stack_name = file_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("stack")
            .to_string();
        
        Self::impl_from_yaml(cls, docker, stack_name, yaml_content)
    }
    
    /// Load stack from YAML content
    pub fn impl_from_yaml(_cls: &PyType, docker: Pyo3Docker, name: String, yaml_content: String) -> PyResult<Self> {
        use serde_yaml::Value;
        
        let yaml: Value = serde_yaml::from_str(&yaml_content)
            .map_err(|e| PyValueError::new_err(format!("Invalid YAML: {}", e)))?;
        
        let mut stack = Self::new(docker.clone(), name);
        
        // Parse services
        if let Some(services) = yaml.get("services").and_then(|s| s.as_mapping()) {
            for (service_name, service_config) in services {
                let name = service_name.as_str()
                    .ok_or_else(|| PyValueError::new_err("Service name must be a string"))?;
                
                let mut service = InternalService::new(name);
                
                // Parse service configuration
                if let Some(config) = service_config.as_mapping() {
                    // Image or build
                    if let Some(image) = config.get(&Value::String("image".to_string()))
                        .and_then(|v| v.as_str()) {
                        service = service.image(image);
                    } else if let Some(build) = config.get(&Value::String("build".to_string())) {
                        service = parse_build_config(service, build)?;
                    }
                    
                    // Ports
                    if let Some(ports) = config.get(&Value::String("ports".to_string()))
                        .and_then(|v| v.as_sequence()) {
                        let port_strings: Vec<String> = ports.iter()
                            .filter_map(|p| p.as_str().map(|s| s.to_string()))
                            .collect();
                        service = service.ports(port_strings);
                    }
                    
                    // Environment
                    if let Some(env) = config.get(&Value::String("environment".to_string())) {
                        service = parse_environment(service, env)?;
                    }
                    
                    // Volumes
                    if let Some(volumes) = config.get(&Value::String("volumes".to_string()))
                        .and_then(|v| v.as_sequence()) {
                        for volume in volumes {
                            if let Some(vol_str) = volume.as_str() {
                                service = service.volume(vol_str);
                            }
                        }
                    }
                    
                    // Networks
                    if let Some(networks) = config.get(&Value::String("networks".to_string())) {
                        service = parse_networks(service, networks)?;
                    }
                    
                    // Depends on
                    if let Some(depends_on) = config.get(&Value::String("depends_on".to_string()))
                        .and_then(|v| v.as_sequence()) {
                        for dep in depends_on {
                            if let Some(dep_str) = dep.as_str() {
                                service = service.depends_on_service(dep_str);
                            }
                        }
                    }
                    
                    // Additional configurations
                    if let Some(restart) = config.get(&Value::String("restart".to_string()))
                        .and_then(|v| v.as_str()) {
                        service = service.restart_policy(restart);
                    }
                    
                    if let Some(hostname) = config.get(&Value::String("hostname".to_string()))
                        .and_then(|v| v.as_str()) {
                        service = service.hostname(hostname);
                    }
                    
                    // Resource limits (deploy.resources in v3)
                    if let Some(deploy) = config.get(&Value::String("deploy".to_string()))
                        .and_then(|v| v.as_mapping()) {
                        if let Some(resources) = deploy.get(&Value::String("resources".to_string()))
                            .and_then(|v| v.as_mapping()) {
                            service = parse_resources(service, resources)?;
                        }
                    }
                }
                
                // Convert to Python Service and register
                let py_service = super::Service { internal: service };
                stack.register_service(py_service)?;
            }
        }
        
        Ok(stack)
    }
}

// Helper functions for YAML parsing

fn parse_build_config(mut service: InternalService, build: &serde_yaml::Value) -> PyResult<InternalService> {
    use serde_yaml::Value;
    
    if let Some(context) = build.as_str() {
        // Simple string context
        Ok(service.build_context(context))
    } else if let Some(build_map) = build.as_mapping() {
        // Complex build configuration
        if let Some(context) = build_map.get(&Value::String("context".to_string()))
            .and_then(|v| v.as_str()) {
            service = service.build_context(context);
        }
        
        if let Some(dockerfile) = build_map.get(&Value::String("dockerfile".to_string()))
            .and_then(|v| v.as_str()) {
            if let Some(context) = build_map.get(&Value::String("context".to_string()))
                .and_then(|v| v.as_str()) {
                service = service.build_with_dockerfile(context, dockerfile);
            }
        }
        
        if let Some(args) = build_map.get(&Value::String("args".to_string())) {
            if let Some(args_map) = args.as_mapping() {
                for (key, value) in args_map {
                    if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
                        service = service.build_arg(k, v);
                    }
                }
            }
        }
        
        if let Some(target) = build_map.get(&Value::String("target".to_string()))
            .and_then(|v| v.as_str()) {
            service = service.build_target(target);
        }
        
        Ok(service)
    } else {
        Err(PyValueError::new_err("Invalid build configuration"))
    }
}

fn parse_environment(mut service: InternalService, env: &serde_yaml::Value) -> PyResult<InternalService> {
    use serde_yaml::Value;
    
    if let Some(env_list) = env.as_sequence() {
        // List format: ["KEY=value"]
        for item in env_list {
            if let Some(env_str) = item.as_str() {
                if let Some((key, value)) = env_str.split_once('=') {
                    service = service.env(key, value);
                }
            }
        }
    } else if let Some(env_map) = env.as_mapping() {
        // Map format: {KEY: value}
        for (key, value) in env_map {
            if let (Some(k), Some(v)) = (key.as_str(), value.as_str()) {
                service = service.env(k, v);
            }
        }
    }
    
    Ok(service)
}

fn parse_networks(mut service: InternalService, networks: &serde_yaml::Value) -> PyResult<InternalService> {
    if let Some(net_list) = networks.as_sequence() {
        for network in net_list {
            if let Some(net_str) = network.as_str() {
                service = service.network(net_str);
            }
        }
    }
    Ok(service)
}

fn parse_resources(mut service: InternalService, resources: &serde_yaml::mapping::Mapping) -> PyResult<InternalService> {
    use serde_yaml::Value;
    
    // Parse limits
    if let Some(limits) = resources.get(&Value::String("limits".to_string()))
        .and_then(|v| v.as_mapping()) {
        if let Some(memory) = limits.get(&Value::String("memory".to_string()))
            .and_then(|v| v.as_str()) {
            service = service.memory(memory);
        }
        if let Some(cpus) = limits.get(&Value::String("cpus".to_string()))
            .and_then(|v| v.as_str()) {
            service = service.cpus(cpus);
        }
    }
    
    // Parse reservations
    if let Some(reservations) = resources.get(&Value::String("reservations".to_string()))
        .and_then(|v| v.as_mapping()) {
        if let Some(memory) = reservations.get(&Value::String("memory".to_string()))
            .and_then(|v| v.as_str()) {
            service = service.memory_reservation(memory);
        }
    }
    
    Ok(service)
}

// Implementation helpers
impl super::Pyo3Stack {
    async fn create_networks(&mut self) -> PyResult<()> {
        // Create a default network for the stack
        let network_name = format!("{}_default", self.name);
        let network = self.docker.networks().create(network_name.clone())?;
        let network_id = network.id()?;
        self.state.networks.insert("default".to_string(), network_id);
        
        // TODO: Create custom networks defined in services
        
        Ok(())
    }
    
    async fn create_volumes(&mut self) -> PyResult<()> {
        // Create named volumes used by services
        let mut volume_names = std::collections::HashSet::new();
        
        for service in self.registered_services.values() {
            let config = service.to_config_map();
            if let Some(volumes_str) = config.get("volumes") {
                for volume in volumes_str.split(',') {
                    if let Some(source) = volume.split(':').next() {
                        // Check if it's a named volume (not a path)
                        if !source.starts_with('/') && !source.starts_with('.') && !source.contains('/') {
                            volume_names.insert(source.to_string());
                        }
                    }
                }
            }
        }
        
        for volume_name in volume_names {
            let full_name = format!("{}_{}", self.name, volume_name);
            let volume = self.docker.volumes().create(full_name.clone(), None)?;
            self.state.volumes.insert(volume_name, full_name);
        }
        
        Ok(())
    }
    
    async fn deploy_services(&mut self) -> PyResult<()> {
        // Deploy services based on dependencies
        let mut deployed = std::collections::HashSet::new();
        let mut to_deploy: Vec<String> = self.registered_services.keys().cloned().collect();
        
        while !to_deploy.is_empty() {
            let mut made_progress = false;
            let mut next_round = Vec::new();
            
            for service_name in to_deploy {
                // Check if all dependencies are deployed
                let deps_satisfied = {
                    let service = &self.registered_services[&service_name];
                    let config = service.to_config_map();
                    if let Some(depends_on) = config.get("depends_on") {
                        depends_on.split(',').all(|dep| deployed.contains(dep))
                    } else {
                        true
                    }
                };
                
                if deps_satisfied {
                    self.deploy_single_service(&service_name).await?;
                    deployed.insert(service_name);
                    made_progress = true;
                } else {
                    next_round.push(service_name);
                }
            }
            
            if !made_progress && !next_round.is_empty() {
                return Err(PyValueError::new_err("Circular dependency detected in services"));
            }
            
            to_deploy = next_round;
        }
        
        Ok(())
    }
    
    async fn deploy_single_service(&mut self, service_name: &str) -> PyResult<()> {
        let service = self.registered_services.get(service_name)
            .ok_or_else(|| PyValueError::new_err(format!("Service {} not found", service_name)))?
            .clone();
        
        let config = service.to_config_map();
        
        // Determine image
        let image = if let Some(img) = config.get("image") {
            img.clone()
        } else if config.contains_key("build_context") {
            // For now, we'll skip build and use a placeholder
            return Err(PyNotImplementedError::new_err("Build support not yet implemented in deployment"));
        } else {
            return Err(PyValueError::new_err(format!("Service {} has no image", service_name)));
        };
        
        // Prepare container configuration
        let container_name = format!("{}_{}_1", self.name, service_name);
        
        // Environment variables
        let environment = if let Some(env_str) = config.get("environment") {
            let mut env_map = HashMap::new();
            for env_pair in env_str.split(',') {
                if let Some((k, v)) = env_pair.split_once('=') {
                    env_map.insert(k.to_string(), v.to_string());
                }
            }
            Some(env_map)
        } else {
            None
        };
        
        // Volumes
        let volumes = if let Some(vol_str) = config.get("volumes") {
            let mut vol_vec = Vec::new();
            for volume in vol_str.split(',') {
                // Replace named volumes with full names
                let parts: Vec<&str> = volume.split(':').collect();
                if parts.len() >= 2 {
                    let source = parts[0];
                    let target = parts[1];
                    let volume_spec = if !source.starts_with('/') && !source.starts_with('.') && !source.contains('/') {
                        // Named volume
                        if let Some(full_name) = self.state.volumes.get(source) {
                            format!("{}:{}", full_name, target)
                        } else {
                            volume.to_string()
                        }
                    } else {
                        volume.to_string()
                    };
                    vol_vec.push(volume_spec);
                }
            }
            Some(vol_vec)
        } else {
            None
        };
        
        // Get additional configuration
        let hostname = config.get("hostname").cloned();
        let working_dir = config.get("working_dir").cloned();
        let restart = config.get("restart").cloned();
        
        // Resource limits
        let memory = config.get("memory").cloned();
        let cpu_shares = config.get("cpu_shares").and_then(|s| s.parse::<i64>().ok());
        let cpu_quota = config.get("cpu_quota").and_then(|s| s.parse::<i64>().ok());
        let cpu_period = config.get("cpu_period").and_then(|s| s.parse::<i64>().ok());
        
        // Labels
        let mut labels = HashMap::new();
        labels.insert("com.docker.compose.project".to_string(), self.name.clone());
        labels.insert("com.docker.compose.service".to_string(), service_name.to_string());
        
        // Create container
        let container = self.docker.containers().create(
            image,
            Some(container_name.clone()),
            None, // command
            environment,
            volumes,
            working_dir,
            None, // user
            hostname,
            memory,
            cpu_shares,
            cpu_quota,
            cpu_period,
            Some(labels),
        )?;
        
        // Connect to network
        if let Some(network_id) = self.state.networks.get("default") {
            let network = self.docker.networks().get(network_id.clone())?;
            network.connect(container.id()?, None)?;
        }
        
        // Start container
        container.start()?;
        
        // Track container
        self.state.containers.entry(service_name.to_string())
            .or_insert_with(Vec::new)
            .push(container.id()?);
        
        Ok(())
    }
    
    async fn remove_service_containers(&mut self, service_name: &str) -> PyResult<()> {
        if let Some(container_ids) = self.state.containers.remove(service_name) {
            for container_id in container_ids {
                if let Ok(container) = self.docker.containers().get(container_id) {
                    let _ = container.stop();
                    let _ = container.remove(Some(true), None);
                }
            }
        }
        Ok(())
    }
    
    async fn scale_service_containers(&mut self, service_name: &str, target_replicas: u32) -> PyResult<()> {
        let current_containers = self.state.containers
            .get(service_name)
            .cloned()
            .unwrap_or_default();
        
        let current_count = current_containers.len() as u32;
        
        if target_replicas > current_count {
            // Scale up
            for i in current_count..target_replicas {
                let service = self.registered_services.get(service_name)
                    .ok_or_else(|| PyValueError::new_err(format!("Service {} not found", service_name)))?
                    .clone();
                
                let config = service.to_config_map();
                let image = config.get("image")
                    .ok_or_else(|| PyValueError::new_err(format!("Service {} has no image", service_name)))?;
                
                let container_name = format!("{}_{}_{}", self.name, service_name, i + 1);
                
                // Create container with same config as deploy_single_service
                let container = self.docker.containers().create(
                    image.clone(),
                    Some(container_name),
                    None, None, None, None, None, None, None, None, None, None, None,
                )?;
                
                container.start()?;
                
                self.state.containers.entry(service_name.to_string())
                    .or_insert_with(Vec::new)
                    .push(container.id()?);
            }
        } else if target_replicas < current_count {
            // Scale down
            let mut containers = current_containers;
            while containers.len() > target_replicas as usize {
                if let Some(container_id) = containers.pop() {
                    if let Ok(container) = self.docker.containers().get(container_id) {
                        let _ = container.stop();
                        let _ = container.remove(Some(true), None);
                    }
                }
            }
            self.state.containers.insert(service_name.to_string(), containers);
        }
        
        Ok(())
    }
    
    async fn remove_networks(&mut self) -> PyResult<()> {
        for (_, network_id) in self.state.networks.drain() {
            if let Ok(network) = self.docker.networks().get(network_id) {
                let _ = network.delete();
            }
        }
        Ok(())
    }
    
    async fn remove_volumes(&mut self) -> PyResult<()> {
        for (_, volume_name) in self.state.volumes.drain() {
            if let Ok(volume) = self.docker.volumes().get(volume_name) {
                let _ = volume.delete();
            }
        }
        Ok(())
    }
    
    fn get_service_status(&self, service_name: &str) -> PyResult<Py<PyAny>> {
        Python::with_gil(|py| {
            let status_dict = PyDict::new(py);
            
            if let Some(container_ids) = self.state.containers.get(service_name) {
                status_dict.set_item("replicas", container_ids.len())?;
                
                let mut running_count = 0;
                for container_id in container_ids {
                    if let Ok(container) = self.docker.containers().get(container_id.clone()) {
                        if let Ok(info) = container.inspect() {
                            if let Ok(state) = info.get_item("State") {
                                if let Ok(running) = state.get_item("Running") {
                                    if running.is_true()? {
                                        running_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                
                status_dict.set_item("running", running_count)?;
                status_dict.set_item("healthy", running_count == container_ids.len())?;
            } else {
                status_dict.set_item("replicas", 0)?;
                status_dict.set_item("running", 0)?;
                status_dict.set_item("healthy", false)?;
            }
            
            Ok(status_dict.into())
        })
    }
}