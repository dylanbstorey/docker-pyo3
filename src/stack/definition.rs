use docker_compose_types::{Compose, Service as ComposeService, Services};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::DockerPyo3Error;

/// Runtime configuration that extends docker-compose functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Order in which services should be deployed (based on depends_on)
    pub deployment_order: Vec<String>,
    /// Global health check timeout in seconds
    pub health_check_timeout: u64,
    /// Scaling policies for services
    pub scale_policies: HashMap<String, ScalePolicy>,
    /// Custom network mappings
    pub network_mappings: HashMap<String, String>,
    /// Volume mappings
    pub volume_mappings: HashMap<String, String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            deployment_order: Vec::new(),
            health_check_timeout: 30,
            scale_policies: HashMap::new(),
            network_mappings: HashMap::new(),
            volume_mappings: HashMap::new(),
        }
    }
}

/// Scale policy for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalePolicy {
    pub replicas: u32,
    pub strategy: ScaleStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleStrategy {
    RollingUpdate,
    Recreate,
}

/// Enhanced stack definition that wraps docker-compose-types::Compose
#[derive(Debug, Clone)]
pub struct StackDefinition {
    pub name: String,
    pub compose: Compose,
    pub runtime_config: RuntimeConfig,
}

impl StackDefinition {
    /// Create a new stack definition with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            compose: Compose::default(),
            runtime_config: RuntimeConfig::default(),
        }
    }

    /// Create from a docker-compose-types::Compose object
    pub fn from_compose(name: String, compose: Compose) -> Result<Self, DockerPyo3Error> {
        let mut stack_def = Self {
            name,
            compose,
            runtime_config: RuntimeConfig::default(),
        };

        // Analyze dependencies and build deployment order
        stack_def.build_deployment_order()?;
        
        Ok(stack_def)
    }

    /// Convert back to docker-compose-types::Compose for export
    pub fn to_compose(&self) -> Compose {
        self.compose.clone()
    }

    /// Get all service names
    pub fn get_service_names(&self) -> Vec<String> {
        match &self.compose.services {
            Some(Services(services)) => services.keys().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Get a specific service definition
    pub fn get_service(&self, name: &str) -> Option<&ComposeService> {
        match &self.compose.services {
            Some(Services(services)) => services.get(name).and_then(|s| s.as_ref()),
            None => None,
        }
    }

    /// Add or update a service
    pub fn set_service(&mut self, name: String, service: ComposeService) {
        if let Some(Services(ref mut services)) = self.compose.services {
            services.insert(name, Some(service));
        } else {
            let mut services = indexmap::IndexMap::new();
            services.insert(name, Some(service));
            self.compose.services = Some(Services(services));
        }
    }

    /// Remove a service
    pub fn remove_service(&mut self, name: &str) -> Option<ComposeService> {
        match &mut self.compose.services {
            Some(Services(services)) => services.remove(name).flatten(),
            None => None,
        }
    }

    /// Build deployment order based on depends_on relationships
    pub fn build_deployment_order(&mut self) -> Result<(), DockerPyo3Error> {
        use std::collections::{HashMap, HashSet, VecDeque};

        let service_names = self.get_service_names();
        if service_names.is_empty() {
            return Ok(());
        }

        // Build dependency graph
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize all services
        for name in &service_names {
            dependencies.insert(name.clone(), Vec::new());
            dependents.insert(name.clone(), Vec::new());
        }

        // Extract depends_on relationships
        for name in &service_names {
            if let Some(service) = self.get_service(name) {
                if let Some(depends_on) = &service.depends_on {
                    match depends_on {
                        docker_compose_types::DependsOnOptions::Simple(deps) => {
                            for dep in deps {
                                if !service_names.contains(dep) {
                                    return Err(DockerPyo3Error::Configuration(
                                        format!("Service '{}' depends on '{}' which is not defined", name, dep)
                                    ));
                                }
                                dependencies.get_mut(name).unwrap().push(dep.clone());
                                dependents.get_mut(dep).unwrap().push(name.clone());
                            }
                        }
                        docker_compose_types::DependsOnOptions::Advanced(deps) => {
                            for (dep, _condition) in deps {
                                if !service_names.contains(dep) {
                                    return Err(DockerPyo3Error::Configuration(
                                        format!("Service '{}' depends on '{}' which is not defined", name, dep)
                                    ));
                                }
                                dependencies.get_mut(name).unwrap().push(dep.clone());
                                dependents.get_mut(dep).unwrap().push(name.clone());
                            }
                        }
                    }
                }
            }
        }

        // Topological sort using Kahn's algorithm
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for name in &service_names {
            in_degree.insert(name.clone(), dependencies[name].len());
        }

        let mut queue: VecDeque<String> = VecDeque::new();
        for (name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(name.clone());
            }
        }

        let mut result = Vec::new();
        while let Some(service) = queue.pop_front() {
            result.push(service.clone());
            
            for dependent in &dependents[&service] {
                let new_degree = in_degree[dependent] - 1;
                in_degree.insert(dependent.clone(), new_degree);
                if new_degree == 0 {
                    queue.push_back(dependent.clone());
                }
            }
        }

        // Check for circular dependencies
        if result.len() != service_names.len() {
            return Err(DockerPyo3Error::Configuration(
                "Circular dependency detected in services".to_string()
            ));
        }

        self.runtime_config.deployment_order = result;
        Ok(())
    }

    /// Get deployment order
    pub fn get_deployment_order(&self) -> &[String] {
        &self.runtime_config.deployment_order
    }

    /// Set scale policy for a service
    pub fn set_scale_policy(&mut self, service_name: String, policy: ScalePolicy) {
        self.runtime_config.scale_policies.insert(service_name, policy);
    }

    /// Get scale policy for a service
    pub fn get_scale_policy(&self, service_name: &str) -> Option<&ScalePolicy> {
        self.runtime_config.scale_policies.get(service_name)
    }
}

/// Parse a stack from YAML content
pub fn parse_stack_from_yaml(name: String, yaml_content: &str) -> Result<StackDefinition, DockerPyo3Error> {
    let compose: Compose = serde_yaml::from_str(yaml_content)
        .map_err(|e| DockerPyo3Error::Configuration(format!("Failed to parse YAML: {}", e)))?;
    
    StackDefinition::from_compose(name, compose)
}

/// Convert stack to YAML content
pub fn stack_to_yaml(stack: &StackDefinition) -> Result<String, DockerPyo3Error> {
    let compose = stack.to_compose();
    serde_yaml::to_string(&compose)
        .map_err(|e| DockerPyo3Error::Configuration(format!("Failed to serialize to YAML: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_definition_creation() {
        let stack = StackDefinition::new("test-stack".to_string());
        assert_eq!(stack.name, "test-stack");
        assert!(stack.get_service_names().is_empty());
    }

    #[test]
    fn test_deployment_order_simple() {
        let yaml = r#"
version: '3.8'
services:
  db:
    image: postgres:13
  web:
    image: nginx
    depends_on:
      - db
"#;
        
        let stack = parse_stack_from_yaml("test".to_string(), yaml).unwrap();
        let order = stack.get_deployment_order();
        
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], "db");
        assert_eq!(order[1], "web");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let yaml = r#"
version: '3.8'
services:
  service1:
    image: nginx
    depends_on:
      - service2
  service2:
    image: postgres
    depends_on:
      - service1
"#;
        
        let result = parse_stack_from_yaml("test".to_string(), yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular dependency"));
    }
}