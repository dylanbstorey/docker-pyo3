use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use std::collections::HashMap;
use std::path::Path;

use crate::{Pyo3Docker, get_runtime};
use crate::error::DockerPyo3Error;
use crate::container::Pyo3Containers;
use crate::network::Pyo3Networks;
use crate::volume::Pyo3Volumes;

use super::definition::{StackDefinition, parse_stack_from_yaml, stack_to_yaml, ScalePolicy, ScaleStrategy};
use super::service::ServiceBuilder;

use docker_compose_types::Service as ComposeService;

/// Runtime state for tracking deployed stack resources
#[derive(Debug, Clone)]
pub struct StackState {
    /// Map of service name to container IDs
    pub containers: HashMap<String, Vec<String>>,
    /// Map of network name to network ID
    pub networks: HashMap<String, String>,
    /// Map of volume name to volume ID  
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

/// Python-exposed Stack class for managing multi-container applications
#[pyclass(name = "Stack")]
#[derive(Debug, Clone)]
pub struct Pyo3Stack {
    docker: Pyo3Docker,
    definition: StackDefinition,
    state: StackState,
}

#[pymethods]
impl Pyo3Stack {
    /// Create a new stack with the given name
    #[new]
    pub fn new(docker: Pyo3Docker, name: String) -> Self {
        let definition = StackDefinition::new(name);
        Self {
            docker,
            definition,
            state: StackState::default(),
        }
    }

    /// Load stack from docker-compose.yml file
    #[classmethod]
    pub fn from_file(_cls: &PyType, docker: Pyo3Docker, path: String) -> PyResult<Self> {
        let file_path = Path::new(&path);
        if !file_path.exists() {
            return Err(DockerPyo3Error::Configuration(
                format!("File not found: {}", path)
            ).into());
        }

        let yaml_content = std::fs::read_to_string(file_path)
            .map_err(|e| DockerPyo3Error::Configuration(format!("Failed to read file {}: {}", path, e)))?;

        // Extract stack name from filename or use directory name
        let stack_name = file_path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("stack")
            .to_string();

        let definition = parse_stack_from_yaml(stack_name, &yaml_content)?;
        
        Ok(Self {
            docker,
            definition,
            state: StackState::default(),
        })
    }

    /// Export stack to docker-compose.yml file
    pub fn to_file(&self, path: String) -> PyResult<()> {
        let yaml_content = stack_to_yaml(&self.definition)?;
        std::fs::write(&path, yaml_content)
            .map_err(|e| DockerPyo3Error::Configuration(format!("Failed to write file {}: {}", path, e)))?;
        Ok(())
    }

    /// Get stack name
    #[getter]
    pub fn name(&self) -> String {
        self.definition.name.clone()
    }

    /// Add a service using the fluent ServiceBuilder API
    pub fn service(&mut self, name: String) -> ServiceBuilder {
        ServiceBuilder::new(name)
    }

    /// Add a completed service to the stack
    pub fn add_service(&mut self, name: String, service: ServiceBuilder) -> PyResult<()> {
        let compose_service = service.build();
        self.definition.set_service(name, compose_service);
        // Rebuild deployment order when services change
        self.definition.build_deployment_order()
            .map_err(|e| PyErr::from(e))?;
        Ok(())
    }

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
            // Remove services in reverse dependency order
            let service_order = self.definition.get_deployment_order();
            for service_name in service_order.iter().rev() {
                self.remove_service_containers(service_name).await?;
            }
            
            // Remove networks and volumes
            self.remove_networks().await?;
            self.remove_volumes().await?;
            
            Ok::<(), PyErr>(())
        })?;
        
        self.state = StackState::default();
        Ok(())
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
            for service_name in self.definition.get_service_names() {
                let service_status = self.get_service_status(&service_name)?;
                services_dict.set_item(&service_name, service_status)?;
            }
            status_dict.set_item("services", services_dict)?;
            
            // Resource counts
            status_dict.set_item("containers", self.state.containers.len())?;
            status_dict.set_item("networks", self.state.networks.len())?;
            status_dict.set_item("volumes", self.state.volumes.len())?;
            
            Ok(status_dict.into())
        })
    }

    /// Scale a specific service
    pub fn scale(&mut self, service_name: String, replicas: u32) -> PyResult<()> {
        // Validate service exists
        if self.definition.get_service(&service_name).is_none() {
            return Err(DockerPyo3Error::Configuration(
                format!("Service '{}' not found in stack", service_name)
            ).into());
        }

        // Update scale policy
        let scale_policy = ScalePolicy {
            replicas,
            strategy: ScaleStrategy::RollingUpdate,
        };
        self.definition.set_scale_policy(service_name.clone(), scale_policy);

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
        let runtime = get_runtime();
        runtime.block_on(async {
            // Stop existing containers
            if let Some(container_ids) = self.state.containers.get(&service_name) {
                for container_id in container_ids {
                    if let Ok(container) = self.docker.containers().get(container_id.clone()) {
                        let _ = container.stop();
                        let _ = container.remove();
                    }
                }
            }

            // Restart the service
            self.deploy_single_service(&service_name).await
        })
    }

    /// Get logs from all services or specific services
    pub fn logs(&self, services: Option<Vec<String>>) -> PyResult<String> {
        let target_services = services.unwrap_or_else(|| self.definition.get_service_names());
        let mut all_logs = Vec::new();

        for service_name in target_services {
            if let Some(container_ids) = self.state.containers.get(&service_name) {
                for container_id in container_ids {
                    if let Ok(container) = self.docker.containers().get(container_id.clone()) {
                        if let Ok(logs) = container.logs(Some(true), Some(true), None, None, Some(true)) {
                            all_logs.push(format!("[{}] {}", service_name, logs));
                        }
                    }
                }
            }
        }

        Ok(all_logs.join("\n"))
    }

    /// List all services in the stack
    pub fn get_services(&self) -> Vec<String> {
        self.definition.get_service_names()
    }

    /// Get the deployment order
    pub fn get_deployment_order(&self) -> Vec<String> {
        self.definition.get_deployment_order().to_vec()
    }
}

impl Pyo3Stack {
    /// Create networks defined in the stack
    async fn create_networks(&mut self) -> PyResult<()> {
        use docker_compose_types::ComposeNetwork;

        if let Some(networks) = &self.definition.compose.networks {
            for (network_name, network_config) in networks {
                if let Some(config) = network_config {
                    // Create the network
                    let full_name = format!("{}_{}", self.definition.name, network_name);
                    
                    let network = self.docker.networks().create(full_name.clone())?;
                    let network_id = network.id()?;
                    
                    self.state.networks.insert(network_name.clone(), network_id);
                }
            }
        }
        Ok(())
    }

    /// Create volumes defined in the stack
    async fn create_volumes(&mut self) -> PyResult<()> {
        if let Some(volumes) = &self.definition.compose.volumes {
            for (volume_name, volume_config) in volumes {
                if let Some(_config) = volume_config {
                    // Create the volume
                    let full_name = format!("{}_{}", self.definition.name, volume_name);
                    
                    let volume = self.docker.volumes().create(full_name.clone(), None)?;
                    let volume_name_result = volume.name()?;
                    
                    self.state.volumes.insert(volume_name.clone(), volume_name_result);
                }
            }
        }
        Ok(())
    }

    /// Deploy all services in dependency order
    async fn deploy_services(&mut self) -> PyResult<()> {
        let deployment_order = self.definition.get_deployment_order().to_vec();
        
        for service_name in deployment_order {
            self.deploy_single_service(&service_name).await?;
        }
        
        Ok(())
    }

    /// Deploy a single service
    async fn deploy_single_service(&mut self, service_name: &str) -> PyResult<()> {
        let service = self.definition.get_service(service_name)
            .ok_or_else(|| DockerPyo3Error::Configuration(
                format!("Service '{}' not found", service_name)
            ))?;

        let image_name = service.image.as_ref()
            .ok_or_else(|| DockerPyo3Error::Configuration(
                format!("Service '{}' has no image defined", service_name)
            ))?;

        // Determine number of replicas
        let replicas = self.definition
            .get_scale_policy(service_name)
            .map(|p| p.replicas)
            .unwrap_or(1);

        let mut container_ids = Vec::new();

        for replica in 0..replicas {
            let container_name = if replicas == 1 {
                format!("{}_{}", self.definition.name, service_name)
            } else {
                format!("{}_{}_{}", self.definition.name, service_name, replica + 1)
            };

            // Create container
            let container = self.docker.containers().create(
                image_name.clone(),
                Some(container_name),
                None, // command - could extract from service.command
                None, // environment - could extract from service.environment
                None, // volumes - could extract from service.volumes
                None, // working_dir - could extract from service.working_dir
                None, // user
                None, // hostname - could extract from service.hostname
                None, // memory
                None, // cpu_shares
                None, // cpu_quota
                None, // cpu_period
                None, // labels - could extract from service.labels
            )?;

            // Start the container
            container.start()?;
            let container_id = container.id()?;
            container_ids.push(container_id);
        }

        self.state.containers.insert(service_name.to_string(), container_ids);
        Ok(())
    }

    /// Scale service containers to target replica count
    async fn scale_service_containers(&mut self, service_name: &str, target_replicas: u32) -> PyResult<()> {
        let current_containers = self.state.containers
            .get(service_name)
            .cloned()
            .unwrap_or_default();

        let current_count = current_containers.len() as u32;

        if target_replicas > current_count {
            // Scale up: create additional containers
            for replica in current_count..target_replicas {
                // This is a simplified version - in reality we'd want to
                // properly handle service configuration like in deploy_single_service
                let service = self.definition.get_service(service_name).unwrap();
                let image_name = service.image.as_ref().unwrap();
                
                let container_name = format!("{}_{}_{}", self.definition.name, service_name, replica + 1);
                let container = self.docker.containers().create(
                    image_name.clone(),
                    Some(container_name),
                    None, None, None, None, None, None, None, None, None, None, None,
                )?;
                
                container.start()?;
                let container_id = container.id()?;
                
                let mut containers = self.state.containers.get(service_name).cloned().unwrap_or_default();
                containers.push(container_id);
                self.state.containers.insert(service_name.to_string(), containers);
            }
        } else if target_replicas < current_count {
            // Scale down: remove excess containers
            let containers_to_remove = current_count - target_replicas;
            let mut remaining_containers = current_containers;
            
            for _ in 0..containers_to_remove {
                if let Some(container_id) = remaining_containers.pop() {
                    if let Ok(container) = self.docker.containers().get(container_id) {
                        let _ = container.stop();
                        let _ = container.remove();
                    }
                }
            }
            
            self.state.containers.insert(service_name.to_string(), remaining_containers);
        }

        Ok(())
    }

    /// Remove all containers for a service
    async fn remove_service_containers(&mut self, service_name: &str) -> PyResult<()> {
        if let Some(container_ids) = self.state.containers.remove(service_name) {
            for container_id in container_ids {
                if let Ok(container) = self.docker.containers().get(container_id) {
                    let _ = container.stop();
                    let _ = container.remove();
                }
            }
        }
        Ok(())
    }

    /// Remove networks created by the stack
    async fn remove_networks(&mut self) -> PyResult<()> {
        for (_, network_id) in self.state.networks.drain() {
            if let Ok(network) = self.docker.networks().get(network_id) {
                let _ = network.delete();
            }
        }
        Ok(())
    }

    /// Remove volumes created by the stack
    async fn remove_volumes(&mut self) -> PyResult<()> {
        for (_, volume_name) in self.state.volumes.drain() {
            if let Ok(volume) = self.docker.volumes().get(volume_name) {
                let _ = volume.delete();
            }
        }
        Ok(())
    }

    /// Get status of a specific service
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