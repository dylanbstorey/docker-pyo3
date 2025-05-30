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

/// Independent Service class for composable service definitions
/// This can be created standalone and then registered into a Stack
#[derive(Debug, Clone)]
pub struct Service {
    name: String,
    compose_service: ComposeService,
}

impl Service {
    /// Create a new independent service
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            compose_service: ComposeService::default(),
        }
    }

    /// Get the service name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the Docker image for this service
    pub fn image<S: Into<String>>(mut self, image: S) -> Self {
        self.compose_service.image = Some(image.into());
        self
    }

    /// Set the build context (alternative to image)
    pub fn build_context<S: Into<String>>(mut self, context: S) -> Self {
        self.compose_service.build = Some(docker_compose_types::BuildStep::Simple(context.into()));
        self
    }

    /// Add port mappings (e.g., ["80:80", "443:443"])
    pub fn ports(mut self, ports: Vec<String>) -> Self {
        self.compose_service.ports = Some(Ports::Short(ports));
        self
    }

    /// Add environment variables
    pub fn environment(mut self, env: HashMap<String, String>) -> Self {
        let env_map: IndexMap<String, Option<String>> = env
            .into_iter()
            .map(|(k, v)| (k, Some(v)))
            .collect();
        self.compose_service.environment = Some(Environment::KvPair(env_map));
        self
    }

    /// Add a single environment variable
    pub fn env<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        let env_map = match self.compose_service.environment {
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
        self.compose_service.environment = Some(Environment::KvPair(env_map));
        self
    }

    /// Add volume mounts (e.g., ["./data:/app/data", "logs:/var/log"])
    pub fn volumes(mut self, volumes: Vec<String>) -> Self {
        use docker_compose_types::{Volumes as VolumesType, VolumeMount};
        
        let volume_mounts: Vec<VolumeMount> = volumes
            .into_iter()
            .map(|v| VolumeMount::Simple(v))
            .collect();
        
        self.compose_service.volumes = Some(VolumesType::Simple(volume_mounts));
        self
    }

    /// Add a single volume mount
    pub fn volume<S: Into<String>>(mut self, volume: S) -> Self {
        use docker_compose_types::{Volumes as VolumesType, VolumeMount};
        
        let volume_mount = VolumeMount::Simple(volume.into());
        
        let volume_mounts = match self.compose_service.volumes {
            Some(VolumesType::Simple(mut mounts)) => {
                mounts.push(volume_mount);
                mounts
            }
            _ => vec![volume_mount],
        };
        
        self.compose_service.volumes = Some(VolumesType::Simple(volume_mounts));
        self
    }

    /// Set command to run in container
    pub fn command(mut self, cmd: Vec<String>) -> Self {
        self.compose_service.command = Some(Command::Simple(cmd));
        self
    }

    /// Set working directory
    pub fn working_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.compose_service.working_dir = Some(dir.into());
        self
    }

    /// Add network connections
    pub fn networks(mut self, networks: Vec<String>) -> Self {
        let network_map: IndexMap<String, Option<docker_compose_types::NetworkSettings>> = networks
            .into_iter()
            .map(|n| (n, None))
            .collect();
        self.compose_service.networks = Some(Networks::Advanced(network_map));
        self
    }

    /// Add single network
    pub fn network<S: Into<String>>(mut self, network: S) -> Self {
        let network_name = network.into();
        
        let network_map = match self.compose_service.networks {
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
        
        self.compose_service.networks = Some(Networks::Advanced(network_map));
        self
    }

    /// Set dependencies (services that must start before this one)
    pub fn depends_on(mut self, services: Vec<String>) -> Self {
        self.compose_service.depends_on = Some(DependsOnOptions::Simple(services));
        self
    }

    /// Add single dependency
    pub fn depends_on_service<S: Into<String>>(mut self, service: S) -> Self {
        let service_name = service.into();
        
        let deps = match self.compose_service.depends_on {
            Some(DependsOnOptions::Simple(mut list)) => {
                list.push(service_name);
                list
            }
            _ => vec![service_name],
        };
        
        self.compose_service.depends_on = Some(DependsOnOptions::Simple(deps));
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
        self.compose_service.restart = Some(restart_policy);
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
        
        self.compose_service.healthcheck = Some(healthcheck);
        self
    }

    /// Set container hostname
    pub fn hostname<S: Into<String>>(mut self, hostname: S) -> Self {
        self.compose_service.hostname = Some(hostname.into());
        self
    }

    /// Set container labels
    pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
        let label_map: IndexMap<String, String> = labels.into_iter().collect();
        self.compose_service.labels = Some(docker_compose_types::Labels::Map(label_map));
        self
    }

    /// Add single label
    pub fn label<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        let label_map = match &self.compose_service.labels {
            Some(docker_compose_types::Labels::Map(map)) => {
                let mut new_map = map.clone();
                new_map.insert(key.into(), value.into());
                new_map
            }
            _ => {
                let mut map = IndexMap::new();
                map.insert(key.into(), value.into());
                map
            }
        };
        self.compose_service.labels = Some(docker_compose_types::Labels::Map(label_map));
        self
    }

    /// Set number of replicas for scaling
    pub fn replicas(mut self, count: u32) -> Self {
        // Store replica count in labels for now (until we have better runtime config)
        self.label("docker-pyo3.replicas", count.to_string())
    }

    /// Set memory limit
    pub fn memory<S: Into<String>>(mut self, limit: S) -> Self {
        // Store memory limit in labels for now
        self.label("docker-pyo3.memory", limit.into())
    }

    /// Build the service definition into a docker-compose Service
    pub fn build(self) -> (String, ComposeService) {
        (self.name, self.compose_service)
    }

    /// Get a reference to the underlying compose service
    pub fn compose_service(&self) -> &ComposeService {
        &self.compose_service
    }

    /// Clone this service with a new name (useful for creating variants)
    pub fn clone_with_name<S: Into<String>>(&self, new_name: S) -> Self {
        Self {
            name: new_name.into(),
            compose_service: self.compose_service.clone(),
        }
    }
}

// Convenience constructors for common service types
impl Service {
    /// Create a web service with common defaults
    pub fn web_service<S: Into<String>>(name: S) -> Self {
        Self::new(name)
            .restart_policy("unless-stopped")
            .healthcheck(
                vec!["CMD".to_string(), "curl".to_string(), "-f".to_string(), "http://localhost/health".to_string()],
                Some(30),
                Some(10),
                Some(3)
            )
    }

    /// Create a database service with common defaults
    pub fn database_service<S: Into<String>>(name: S) -> Self {
        Self::new(name)
            .restart_policy("unless-stopped")
            .healthcheck(
                vec!["CMD-SHELL".to_string(), "pg_isready -U postgres".to_string()],
                Some(30),
                Some(5),
                Some(5)
            )
    }

    /// Create a redis service with common defaults
    pub fn redis_service<S: Into<String>>(name: S) -> Self {
        Self::new(name)
            .image("redis:7-alpine")
            .ports(vec!["6379:6379".to_string()])
            .restart_policy("unless-stopped")
            .healthcheck(
                vec!["CMD".to_string(), "redis-cli".to_string(), "ping".to_string()],
                Some(30),
                Some(3),
                Some(5)
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_independent_service_creation() {
        let service = Service::new("web")
            .image("nginx:latest")
            .ports(vec!["80:80".to_string()])
            .env("ENV", "production");

        assert_eq!(service.name(), "web");
        assert_eq!(service.compose_service().image, Some("nginx:latest".to_string()));
    }

    #[test]
    fn test_service_cloning() {
        let base_service = Service::web_service("web")
            .image("nginx:latest")
            .ports(vec!["80:80".to_string()]);

        let dev_service = base_service.clone_with_name("web-dev")
            .env("ENV", "development")
            .ports(vec!["8080:80".to_string()]);

        assert_eq!(base_service.name(), "web");
        assert_eq!(dev_service.name(), "web-dev");
        
        // Both should have nginx image
        assert_eq!(base_service.compose_service().image, Some("nginx:latest".to_string()));
        assert_eq!(dev_service.compose_service().image, Some("nginx:latest".to_string()));
    }

    #[test]
    fn test_convenience_constructors() {
        let web = Service::web_service("api");
        let db = Service::database_service("postgres");
        let cache = Service::redis_service("cache");

        assert_eq!(web.name(), "api");
        assert_eq!(db.name(), "postgres");
        assert_eq!(cache.name(), "cache");

        // Redis should have default image and port
        assert_eq!(cache.compose_service().image, Some("redis:7-alpine".to_string()));
    }

    #[test]
    fn test_service_build() {
        let service = Service::new("test")
            .image("test:latest")
            .env("KEY", "value");

        let (name, compose_service) = service.build();
        assert_eq!(name, "test");
        assert_eq!(compose_service.image, Some("test:latest".to_string()));
    }
}