use docker_compose_types::{
    Service as ComposeService, 
    Ports, 
    Environment,
    Volumes,
    Networks,
    Command,
    Healthcheck,
    RestartPolicy,
    DependsOnOptions,
};
use std::collections::HashMap;
use indexmap::IndexMap;

/// Builder for creating docker-compose services with a fluent API
#[derive(Debug, Clone)]
pub struct ServiceBuilder {
    service: ComposeService,
    service_name: String,
}

impl ServiceBuilder {
    /// Create a new service builder
    pub fn new(name: String) -> Self {
        Self {
            service: ComposeService::default(),
            service_name: name,
        }
    }

    /// Set the Docker image for this service
    pub fn image<S: Into<String>>(mut self, image: S) -> Self {
        self.service.image = Some(image.into());
        self.service.build = None; // Image and build are mutually exclusive
        self
    }

    /// Set the build context (alternative to image)
    pub fn build_context<S: Into<String>>(mut self, context: S) -> Self {
        self.service.build = Some(docker_compose_types::BuildStep::Simple(context.into()));
        self.service.image = None; // Build and image are mutually exclusive
        self
    }
    
    /// Set build context with dockerfile
    pub fn build_with_dockerfile<C: Into<String>, D: Into<String>>(mut self, context: C, dockerfile: D) -> Self {
        use docker_compose_types::{BuildStep, Build};
        let build = Build {
            context: Some(context.into()),
            dockerfile: Some(dockerfile.into()),
            args: None,
            target: None,
            cache_from: None,
            ..Default::default()
        };
        self.service.build = Some(BuildStep::Advanced(build));
        self.service.image = None;
        self
    }
    
    /// Add build argument
    pub fn build_arg<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        use docker_compose_types::{BuildStep, Build};
        use indexmap::IndexMap;
        
        let mut args = IndexMap::new();
        args.insert(key.into(), Some(value.into()));
        
        match self.service.build {
            Some(BuildStep::Advanced(mut build)) => {
                if let Some(ref mut existing_args) = build.args {
                    existing_args.extend(args);
                } else {
                    build.args = Some(args);
                }
                self.service.build = Some(BuildStep::Advanced(build));
            }
            Some(BuildStep::Simple(context)) => {
                let build = Build {
                    context: Some(context),
                    args: Some(args),
                    ..Default::default()
                };
                self.service.build = Some(BuildStep::Advanced(build));
            }
            None => {
                let build = Build {
                    context: Some(".".to_string()),
                    args: Some(args),
                    ..Default::default()
                };
                self.service.build = Some(BuildStep::Advanced(build));
            }
        }
        self
    }
    
    /// Set build target
    pub fn build_target<S: Into<String>>(mut self, target: S) -> Self {
        use docker_compose_types::{BuildStep, Build};
        
        match self.service.build {
            Some(BuildStep::Advanced(mut build)) => {
                build.target = Some(target.into());
                self.service.build = Some(BuildStep::Advanced(build));
            }
            Some(BuildStep::Simple(context)) => {
                let build = Build {
                    context: Some(context),
                    target: Some(target.into()),
                    ..Default::default()
                };
                self.service.build = Some(BuildStep::Advanced(build));
            }
            None => {
                let build = Build {
                    context: Some(".".to_string()),
                    target: Some(target.into()),
                    ..Default::default()
                };
                self.service.build = Some(BuildStep::Advanced(build));
            }
        }
        self
    }
    
    /// Add cache from image
    pub fn build_cache_from<S: Into<String>>(mut self, image: S) -> Self {
        use docker_compose_types::{BuildStep, Build};
        
        match self.service.build {
            Some(BuildStep::Advanced(mut build)) => {
                if let Some(ref mut cache_from) = build.cache_from {
                    cache_from.push(image.into());
                } else {
                    build.cache_from = Some(vec![image.into()]);
                }
                self.service.build = Some(BuildStep::Advanced(build));
            }
            Some(BuildStep::Simple(context)) => {
                let build = Build {
                    context: Some(context),
                    cache_from: Some(vec![image.into()]),
                    ..Default::default()
                };
                self.service.build = Some(BuildStep::Advanced(build));
            }
            None => {
                let build = Build {
                    context: Some(".".to_string()),
                    cache_from: Some(vec![image.into()]),
                    ..Default::default()
                };
                self.service.build = Some(BuildStep::Advanced(build));
            }
        }
        self
    }
    
    /// Set resource limits (memory)
    pub fn memory_limit<S: Into<String>>(mut self, limit: S) -> Self {
        // Note: docker-compose-types doesn't have built-in resource limits
        // This would typically be handled via deploy.resources in docker-compose v3+
        // For now, we'll add it as a label for reference
        self.label("resource.memory", limit)
    }
    
    /// Set CPU limits
    pub fn cpu_limit<S: Into<String>>(mut self, cpus: S) -> Self {
        // Note: docker-compose-types doesn't have built-in resource limits
        // This would typically be handled via deploy.resources in docker-compose v3+
        // For now, we'll add it as a label for reference
        self.label("resource.cpus", cpus)
    }

    /// Add port mappings (e.g., ["80:80", "443:443"])
    pub fn ports(mut self, ports: Vec<String>) -> Self {
        self.service.ports = Some(Ports::Short(ports));
        self
    }

    /// Add environment variables
    pub fn environment(mut self, env: HashMap<String, String>) -> Self {
        let env_map: IndexMap<String, Option<String>> = env
            .into_iter()
            .map(|(k, v)| (k, Some(v)))
            .collect();
        self.service.environment = Some(Environment::KvPair(env_map));
        self
    }

    /// Add a single environment variable
    pub fn env<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        let env_map = match self.service.environment {
            Some(Environment::KvPair(mut map)) => {
                map.insert(key.into(), Some(value.into()));
                map
            }
            _ => {
                let mut map = IndexMap::new();
                map.insert(key.into(), Some(value.into()));
                map
            }
        };
        self.service.environment = Some(Environment::KvPair(env_map));
        self
    }

    /// Add volume mounts (e.g., ["./data:/app/data", "logs:/var/log"])
    pub fn volumes(mut self, volumes: Vec<String>) -> Self {
        use docker_compose_types::{Volumes as VolumesType, VolumeMount};
        
        let volume_mounts: Vec<VolumeMount> = volumes
            .into_iter()
            .map(|v| VolumeMount::Simple(v))
            .collect();
        
        self.service.volumes = Some(VolumesType::Simple(volume_mounts));
        self
    }

    /// Add a single volume mount
    pub fn volume<S: Into<String>>(mut self, volume: S) -> Self {
        use docker_compose_types::{Volumes as VolumesType, VolumeMount};
        
        let volume_mount = VolumeMount::Simple(volume.into());
        
        let volume_mounts = match self.service.volumes {
            Some(VolumesType::Simple(mut mounts)) => {
                mounts.push(volume_mount);
                mounts
            }
            _ => vec![volume_mount],
        };
        
        self.service.volumes = Some(VolumesType::Simple(volume_mounts));
        self
    }

    /// Set command to run in container
    pub fn command(mut self, cmd: Vec<String>) -> Self {
        self.service.command = Some(Command::Simple(cmd));
        self
    }

    /// Set working directory
    pub fn working_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.service.working_dir = Some(dir.into());
        self
    }

    /// Add network connections
    pub fn networks(mut self, networks: Vec<String>) -> Self {
        let network_map: IndexMap<String, Option<docker_compose_types::NetworkSettings>> = networks
            .into_iter()
            .map(|n| (n, None))
            .collect();
        self.service.networks = Some(Networks::Advanced(network_map));
        self
    }

    /// Add single network
    pub fn network<S: Into<String>>(mut self, network: S) -> Self {
        let network_name = network.into();
        
        let network_map = match self.service.networks {
            Some(Networks::Advanced(mut map)) => {
                map.insert(network_name, None);
                map
            }
            Some(Networks::Simple(mut list)) => {
                list.push(network_name.clone());
                return self.networks(list);
            }
            _ => {
                let mut map = IndexMap::new();
                map.insert(network_name, None);
                map
            }
        };
        
        self.service.networks = Some(Networks::Advanced(network_map));
        self
    }

    /// Set dependencies (services that must start before this one)
    pub fn depends_on(mut self, services: Vec<String>) -> Self {
        self.service.depends_on = Some(DependsOnOptions::Simple(services));
        self
    }

    /// Add single dependency
    pub fn depends_on_service<S: Into<String>>(mut self, service: S) -> Self {
        let service_name = service.into();
        
        let deps = match self.service.depends_on {
            Some(DependsOnOptions::Simple(mut list)) => {
                list.push(service_name);
                list
            }
            _ => vec![service_name],
        };
        
        self.service.depends_on = Some(DependsOnOptions::Simple(deps));
        self
    }

    /// Set restart policy
    pub fn restart_policy(mut self, policy: &str) -> Self {
        let restart_policy = match policy {
            "no" => RestartPolicy::No,
            "always" => RestartPolicy::Always,
            "on-failure" => RestartPolicy::OnFailure,
            "unless-stopped" => RestartPolicy::UnlessStopped,
            _ => RestartPolicy::No,
        };
        self.service.restart = Some(restart_policy);
        self
    }

    /// Add health check
    pub fn healthcheck(
        mut self, 
        test: Vec<String>, 
        interval_secs: Option<u64>,
        timeout_secs: Option<u64>,
        retries: Option<u64>
    ) -> Self {
        let mut healthcheck = Healthcheck::default();
        healthcheck.test = Some(docker_compose_types::HealthcheckTest::Multiple(test));
        
        if let Some(interval) = interval_secs {
            healthcheck.interval = Some(format!("{}s", interval));
        }
        if let Some(timeout) = timeout_secs {
            healthcheck.timeout = Some(format!("{}s", timeout));
        }
        if let Some(retries) = retries {
            healthcheck.retries = Some(retries);
        }
        
        self.service.healthcheck = Some(healthcheck);
        self
    }

    /// Set container hostname
    pub fn hostname<S: Into<String>>(mut self, hostname: S) -> Self {
        self.service.hostname = Some(hostname.into());
        self
    }

    /// Set container labels
    pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
        let label_map: IndexMap<String, String> = labels.into_iter().collect();
        self.service.labels = Some(docker_compose_types::Labels::Map(label_map));
        self
    }

    /// Add single label
    pub fn label<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        let label_map = match self.service.labels {
            Some(docker_compose_types::Labels::Map(mut map)) => {
                map.insert(key.into(), value.into());
                map
            }
            _ => {
                let mut map = IndexMap::new();
                map.insert(key.into(), value.into());
                map
            }
        };
        self.service.labels = Some(docker_compose_types::Labels::Map(label_map));
        self
    }

    /// Build the service definition
    pub fn build(self) -> ComposeService {
        self.service
    }

    /// Get the service name
    pub fn name(&self) -> &str {
        &self.service_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_builder_basic() {
        let service = ServiceBuilder::new("web".to_string())
            .image("nginx:latest")
            .ports(vec!["80:80".to_string()])
            .env("ENV", "production")
            .build();

        assert_eq!(service.image, Some("nginx:latest".to_string()));
        
        if let Some(Ports::Short(ports)) = service.ports {
            assert_eq!(ports, vec!["80:80"]);
        } else {
            panic!("Expected short port format");
        }

        if let Some(Environment::KvPair(env)) = service.environment {
            assert_eq!(env.get("ENV"), Some(&Some("production".to_string())));
        } else {
            panic!("Expected environment variables");
        }
    }

    #[test]
    fn test_service_builder_complex() {
        let service = ServiceBuilder::new("api".to_string())
            .image("myapp:latest")
            .ports(vec!["8080:8080".to_string()])
            .environment(HashMap::from([
                ("DATABASE_URL".to_string(), "postgres://...".to_string()),
                ("REDIS_URL".to_string(), "redis://...".to_string()),
            ]))
            .volumes(vec![
                "./logs:/app/logs".to_string(),
                "data_volume:/app/data".to_string(),
            ])
            .depends_on(vec!["db".to_string(), "redis".to_string()])
            .healthcheck(
                vec!["CMD".to_string(), "curl".to_string(), "-f".to_string(), "http://localhost:8080/health".to_string()],
                Some(30),
                Some(10),
                Some(3)
            )
            .restart_policy("unless-stopped")
            .build();

        assert_eq!(service.image, Some("myapp:latest".to_string()));
        
        // Check depends_on
        if let Some(DependsOnOptions::Simple(deps)) = service.depends_on {
            assert!(deps.contains(&"db".to_string()));
            assert!(deps.contains(&"redis".to_string()));
        } else {
            panic!("Expected simple depends_on");
        }

        // Check restart policy
        assert_eq!(service.restart, Some(RestartPolicy::UnlessStopped));

        // Check healthcheck
        assert!(service.healthcheck.is_some());
    }

    #[test]
    fn test_service_builder_build_configuration() {
        let service = ServiceBuilder::new("app".to_string())
            .build_context(".")
            .build_arg("NODE_ENV", "production")
            .build_target("production")
            .build_cache_from("node:18-alpine")
            .build();

        // Should not have image when build is set
        assert!(service.image.is_none());
        
        // Should have advanced build configuration
        if let Some(docker_compose_types::BuildStep::Advanced(build)) = service.build {
            assert_eq!(build.context, Some(".".to_string()));
            assert_eq!(build.target, Some("production".to_string()));
            assert!(build.args.is_some());
            assert!(build.cache_from.is_some());
        } else {
            panic!("Expected advanced build configuration");
        }
    }

    #[test]
    fn test_mutual_exclusivity_image_build() {
        // Test image -> build
        let service1 = ServiceBuilder::new("app".to_string())
            .image("nginx:latest")
            .build_context(".")
            .build();
        
        assert!(service1.image.is_none());
        assert!(service1.build.is_some());
        
        // Test build -> image
        let service2 = ServiceBuilder::new("app".to_string())
            .build_context(".")
            .image("nginx:latest")
            .build();
        
        assert!(service2.build.is_none());
        assert_eq!(service2.image, Some("nginx:latest".to_string()));
    }
}