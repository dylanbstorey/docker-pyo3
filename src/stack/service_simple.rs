use std::collections::HashMap;

/// Build configuration for docker-compose build support
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub context: String,
    pub dockerfile: Option<String>,
    pub args: HashMap<String, String>,
    pub target: Option<String>,
    pub cache_from: Vec<String>,
    pub network: Option<String>,
    pub ssh: Option<String>,
}

impl BuildConfig {
    pub fn new<S: Into<String>>(context: S) -> Self {
        Self {
            context: context.into(),
            dockerfile: None,
            args: HashMap::new(),
            target: None,
            cache_from: Vec::new(),
            network: None,
            ssh: None,
        }
    }
}

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub memory: Option<String>,
    pub memory_reservation: Option<String>,
    pub cpus: Option<String>,
    pub cpu_shares: Option<u64>,
    pub cpu_quota: Option<u64>,
    pub cpu_period: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory: None,
            memory_reservation: None,
            cpus: None,
            cpu_shares: None,
            cpu_quota: None,
            cpu_period: None,
        }
    }
}

/// Port configuration with advanced options
#[derive(Debug, Clone)]
pub struct PortConfig {
    pub target: u16,
    pub published: Option<u16>,
    pub protocol: String, // tcp, udp
    pub mode: Option<String>, // host, ingress
}

impl PortConfig {
    pub fn new(target: u16) -> Self {
        Self {
            target,
            published: None,
            protocol: "tcp".to_string(),
            mode: None,
        }
    }
    
    pub fn simple_mapping(mapping: &str) -> Option<Self> {
        if let Some((published, target)) = mapping.split_once(':') {
            if let (Ok(pub_port), Ok(tgt_port)) = (published.parse::<u16>(), target.parse::<u16>()) {
                return Some(Self {
                    target: tgt_port,
                    published: Some(pub_port),
                    protocol: "tcp".to_string(),
                    mode: None,
                });
            }
        }
        None
    }
}

/// Volume configuration with advanced options
#[derive(Debug, Clone)]
pub struct VolumeConfig {
    pub source: String,
    pub target: String,
    pub volume_type: String, // bind, volume, tmpfs
    pub read_only: bool,
    pub bind_options: Option<HashMap<String, String>>,
}

impl VolumeConfig {
    pub fn simple_mapping(mapping: &str) -> Option<Self> {
        if let Some((source, target)) = mapping.split_once(':') {
            return Some(Self {
                source: source.to_string(),
                target: target.to_string(),
                volume_type: if source.starts_with('/') || source.starts_with('.') {
                    "bind".to_string()
                } else {
                    "volume".to_string()
                },
                read_only: false,
                bind_options: None,
            });
        }
        None
    }
}

/// Simplified independent Service class for composable service definitions
/// This avoids the complex docker-compose-types API issues for now
#[derive(Debug, Clone)]
pub struct Service {
    name: String,
    image: Option<String>,
    build: Option<BuildConfig>,
    ports: Vec<String>,
    advanced_ports: Vec<PortConfig>,
    environment: HashMap<String, String>,
    env_files: Vec<String>,
    volumes: Vec<String>,
    advanced_volumes: Vec<VolumeConfig>,
    command: Option<Vec<String>>,
    working_dir: Option<String>,
    networks: Vec<String>,
    depends_on: Vec<String>,
    restart_policy: Option<String>,
    hostname: Option<String>,
    labels: HashMap<String, String>,
    replicas: u32,
    resources: ResourceLimits,
    secrets: Vec<String>,
    healthcheck: Option<HashMap<String, String>>,
}

impl Service {
    /// Create a new independent service
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            image: None,
            build: None,
            ports: Vec::new(),
            advanced_ports: Vec::new(),
            environment: HashMap::new(),
            env_files: Vec::new(),
            volumes: Vec::new(),
            advanced_volumes: Vec::new(),
            command: None,
            working_dir: None,
            networks: Vec::new(),
            depends_on: Vec::new(),
            restart_policy: None,
            hostname: None,
            labels: HashMap::new(),
            replicas: 1,
            resources: ResourceLimits::default(),
            secrets: Vec::new(),
            healthcheck: None,
        }
    }

    /// Get the service name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the Docker image for this service
    pub fn image<S: Into<String>>(mut self, image: S) -> Self {
        self.image = Some(image.into());
        self.build = None; // Image and build are mutually exclusive
        self
    }

    /// Add port mappings (e.g., ["80:80", "443:443"])
    pub fn ports(mut self, ports: Vec<String>) -> Self {
        self.ports = ports;
        self
    }

    /// Add environment variable
    pub fn env<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn environment(mut self, env: HashMap<String, String>) -> Self {
        self.environment.extend(env);
        self
    }

    /// Add volume mount
    pub fn volume<S: Into<String>>(mut self, volume: S) -> Self {
        self.volumes.push(volume.into());
        self
    }

    /// Add multiple volume mounts
    pub fn volumes(mut self, volumes: Vec<String>) -> Self {
        self.volumes.extend(volumes);
        self
    }

    /// Set command to run in container
    pub fn command(mut self, cmd: Vec<String>) -> Self {
        self.command = Some(cmd);
        self
    }

    /// Set working directory
    pub fn working_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Add network
    pub fn network<S: Into<String>>(mut self, network: S) -> Self {
        self.networks.push(network.into());
        self
    }

    /// Add multiple networks
    pub fn networks(mut self, networks: Vec<String>) -> Self {
        self.networks.extend(networks);
        self
    }

    /// Add dependency
    pub fn depends_on_service<S: Into<String>>(mut self, service: S) -> Self {
        self.depends_on.push(service.into());
        self
    }

    /// Add multiple dependencies
    pub fn depends_on(mut self, services: Vec<String>) -> Self {
        self.depends_on.extend(services);
        self
    }

    /// Set restart policy
    pub fn restart_policy<S: Into<String>>(mut self, policy: S) -> Self {
        self.restart_policy = Some(policy.into());
        self
    }

    /// Set hostname
    pub fn hostname<S: Into<String>>(mut self, hostname: S) -> Self {
        self.hostname = Some(hostname.into());
        self
    }

    /// Add label
    pub fn label<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Add multiple labels
    pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels.extend(labels);
        self
    }

    /// Set number of replicas for scaling
    pub fn replicas(mut self, count: u32) -> Self {
        self.replicas = count;
        self
    }

    /// Set memory limit
    pub fn memory<S: Into<String>>(mut self, limit: S) -> Self {
        self.resources.memory = Some(limit.into());
        self
    }
    
    // BUILD CONFIGURATION
    
    /// Set build context (alternative to image)
    pub fn build_context<S: Into<String>>(mut self, context: S) -> Self {
        self.build = Some(BuildConfig::new(context));
        self.image = None; // Build and image are mutually exclusive
        self
    }
    
    /// Set build context with dockerfile
    pub fn build_with_dockerfile<C: Into<String>, D: Into<String>>(mut self, context: C, dockerfile: D) -> Self {
        let mut build_config = BuildConfig::new(context);
        build_config.dockerfile = Some(dockerfile.into());
        self.build = Some(build_config);
        self.image = None;
        self
    }
    
    /// Add build argument
    pub fn build_arg<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        if let Some(ref mut build) = self.build {
            build.args.insert(key.into(), value.into());
        } else {
            let mut build_config = BuildConfig::new(".");
            build_config.args.insert(key.into(), value.into());
            self.build = Some(build_config);
        }
        self
    }
    
    /// Set build target
    pub fn build_target<S: Into<String>>(mut self, target: S) -> Self {
        if let Some(ref mut build) = self.build {
            build.target = Some(target.into());
        } else {
            let mut build_config = BuildConfig::new(".");
            build_config.target = Some(target.into());
            self.build = Some(build_config);
        }
        self
    }
    
    /// Add cache from image
    pub fn build_cache_from<S: Into<String>>(mut self, image: S) -> Self {
        if let Some(ref mut build) = self.build {
            build.cache_from.push(image.into());
        } else {
            let mut build_config = BuildConfig::new(".");
            build_config.cache_from.push(image.into());
            self.build = Some(build_config);
        }
        self
    }
    
    // RESOURCE MANAGEMENT
    
    /// Set memory reservation
    pub fn memory_reservation<S: Into<String>>(mut self, limit: S) -> Self {
        self.resources.memory_reservation = Some(limit.into());
        self
    }
    
    /// Set CPU limits
    pub fn cpus<S: Into<String>>(mut self, cpus: S) -> Self {
        self.resources.cpus = Some(cpus.into());
        self
    }
    
    /// Set CPU shares
    pub fn cpu_shares(mut self, shares: u64) -> Self {
        self.resources.cpu_shares = Some(shares);
        self
    }
    
    /// Set CPU quota and period
    pub fn cpu_quota(mut self, quota: u64, period: Option<u64>) -> Self {
        self.resources.cpu_quota = Some(quota);
        if let Some(p) = period {
            self.resources.cpu_period = Some(p);
        }
        self
    }
    
    // ADVANCED PORT CONFIGURATION
    
    /// Add advanced port configuration
    pub fn port_advanced(mut self, target: u16, published: Option<u16>, protocol: Option<String>, mode: Option<String>) -> Self {
        let mut port_config = PortConfig::new(target);
        port_config.published = published;
        if let Some(proto) = protocol {
            port_config.protocol = proto;
        }
        port_config.mode = mode;
        self.advanced_ports.push(port_config);
        self
    }
    
    // ADVANCED VOLUME CONFIGURATION
    
    /// Add advanced volume configuration
    pub fn volume_advanced<S: Into<String>, T: Into<String>>(mut self, source: S, target: T, volume_type: Option<String>, read_only: bool) -> Self {
        let vol_config = VolumeConfig {
            source: source.into(),
            target: target.into(),
            volume_type: volume_type.unwrap_or_else(|| "bind".to_string()),
            read_only,
            bind_options: None,
        };
        self.advanced_volumes.push(vol_config);
        self
    }
    
    // ENVIRONMENT FILES & SECRETS
    
    /// Add environment file
    pub fn env_file<S: Into<String>>(mut self, file: S) -> Self {
        self.env_files.push(file.into());
        self
    }
    
    /// Add secret
    pub fn secret<S: Into<String>>(mut self, secret: S) -> Self {
        self.secrets.push(secret.into());
        self
    }
    
    /// Add health check
    pub fn healthcheck(mut self, test: Vec<String>, interval: Option<String>, timeout: Option<String>, retries: Option<u32>, start_period: Option<String>) -> Self {
        let mut hc = HashMap::new();
        hc.insert("test".to_string(), test.join(" "));
        if let Some(i) = interval {
            hc.insert("interval".to_string(), i);
        }
        if let Some(t) = timeout {
            hc.insert("timeout".to_string(), t);
        }
        if let Some(r) = retries {
            hc.insert("retries".to_string(), r.to_string());
        }
        if let Some(sp) = start_period {
            hc.insert("start_period".to_string(), sp);
        }
        self.healthcheck = Some(hc);
        self
    }

    /// Clone this service with a new name (useful for creating variants)
    pub fn clone_with_name<S: Into<String>>(&self, new_name: S) -> Self {
        let mut cloned = self.clone();
        cloned.name = new_name.into();
        cloned
    }
    
    /// Get the raw command as Vec<String> for proper Docker API usage
    pub fn get_raw_command(&self) -> Option<Vec<String>> {
        self.command.clone()
    }

    /// Get service configuration as a simple map for inspection
    pub fn to_config_map(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("name".to_string(), self.name.clone());
        
        if let Some(ref image) = self.image {
            config.insert("image".to_string(), image.clone());
        }
        
        if !self.ports.is_empty() {
            config.insert("ports".to_string(), self.ports.join(","));
        }
        
        if !self.environment.is_empty() {
            let env_str: Vec<String> = self.environment
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            config.insert("environment".to_string(), env_str.join(","));
        }
        
        if !self.volumes.is_empty() {
            config.insert("volumes".to_string(), self.volumes.join(","));
        }
        
        if let Some(ref cmd) = self.command {
            config.insert("command".to_string(), cmd.join(" "));
        }
        
        if let Some(ref workdir) = self.working_dir {
            config.insert("working_dir".to_string(), workdir.clone());
        }
        
        if !self.networks.is_empty() {
            config.insert("networks".to_string(), self.networks.join(","));
        }
        
        if !self.depends_on.is_empty() {
            config.insert("depends_on".to_string(), self.depends_on.join(","));
        }
        
        if let Some(ref restart) = self.restart_policy {
            config.insert("restart".to_string(), restart.clone());
        }
        
        if let Some(ref hostname) = self.hostname {
            config.insert("hostname".to_string(), hostname.clone());
        }
        
        if !self.labels.is_empty() {
            let labels_str: Vec<String> = self.labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            config.insert("labels".to_string(), labels_str.join(","));
        }
        
        config.insert("replicas".to_string(), self.replicas.to_string());
        
        // Resource limits
        if let Some(ref memory) = self.resources.memory {
            config.insert("memory".to_string(), memory.clone());
        }
        if let Some(ref memory_res) = self.resources.memory_reservation {
            config.insert("memory_reservation".to_string(), memory_res.clone());
        }
        if let Some(ref cpus) = self.resources.cpus {
            config.insert("cpus".to_string(), cpus.clone());
        }
        if let Some(cpu_shares) = self.resources.cpu_shares {
            config.insert("cpu_shares".to_string(), cpu_shares.to_string());
        }
        
        // Build configuration
        if let Some(ref build) = self.build {
            config.insert("build_context".to_string(), build.context.clone());
            if let Some(ref dockerfile) = build.dockerfile {
                config.insert("dockerfile".to_string(), dockerfile.clone());
            }
            if let Some(ref target) = build.target {
                config.insert("build_target".to_string(), target.clone());
            }
            if !build.args.is_empty() {
                let args_str: Vec<String> = build.args
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                config.insert("build_args".to_string(), args_str.join(","));
            }
        }
        
        // Environment files
        if !self.env_files.is_empty() {
            config.insert("env_files".to_string(), self.env_files.join(","));
        }
        
        // Secrets
        if !self.secrets.is_empty() {
            config.insert("secrets".to_string(), self.secrets.join(","));
        }
        
        // Advanced ports
        if !self.advanced_ports.is_empty() {
            let ports_str: Vec<String> = self.advanced_ports
                .iter()
                .map(|p| {
                    if let Some(published) = p.published {
                        format!("{}:{}:{}", published, p.target, p.protocol)
                    } else {
                        format!("{}:{}", p.target, p.protocol)
                    }
                })
                .collect();
            config.insert("advanced_ports".to_string(), ports_str.join(","));
        }
        
        // Advanced volumes
        if !self.advanced_volumes.is_empty() {
            let volumes_str: Vec<String> = self.advanced_volumes
                .iter()
                .map(|v| format!("{}:{}:{}:{}", v.source, v.target, v.volume_type, if v.read_only { "ro" } else { "rw" }))
                .collect();
            config.insert("advanced_volumes".to_string(), volumes_str.join(","));
        }
        
        // Health check
        if let Some(ref hc) = self.healthcheck {
            if let Some(test) = hc.get("test") {
                config.insert("healthcheck_test".to_string(), test.clone());
            }
        }
        
        config
    }
}

// Convenience constructors for common service types
impl Service {
    /// Create a web service with common defaults
    pub fn web_service<S: Into<String>>(name: S) -> Self {
        Self::new(name)
            .restart_policy("unless-stopped")
            .label("service.type", "web")
    }

    /// Create a database service with common defaults
    pub fn database_service<S: Into<String>>(name: S) -> Self {
        Self::new(name)
            .restart_policy("unless-stopped")
            .label("service.type", "database")
    }

    /// Create a redis service with common defaults
    pub fn redis_service<S: Into<String>>(name: S) -> Self {
        Self::new(name)
            .image("redis:7-alpine")
            .ports(vec!["6379:6379".to_string()])
            .restart_policy("unless-stopped")
            .label("service.type", "cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_service_creation() {
        let service = Service::new("web")
            .image("nginx:latest")
            .ports(vec!["80:80".to_string()])
            .env("ENV", "production");

        assert_eq!(service.name(), "web");
        assert_eq!(service.image, Some("nginx:latest".to_string()));
        assert_eq!(service.ports, vec!["80:80"]);
        assert_eq!(service.environment.get("ENV"), Some(&"production".to_string()));
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
        assert_eq!(base_service.image, Some("nginx:latest".to_string()));
        assert_eq!(dev_service.image, Some("nginx:latest".to_string()));
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
        assert_eq!(cache.image, Some("redis:7-alpine".to_string()));
        assert_eq!(cache.ports, vec!["6379:6379"]);
    }

    #[test]
    fn test_config_map() {
        let service = Service::new("test")
            .image("test:latest")
            .env("KEY", "value")
            .ports(vec!["8080:80".to_string()]);

        let config = service.to_config_map();
        assert_eq!(config.get("name"), Some(&"test".to_string()));
        assert_eq!(config.get("image"), Some(&"test:latest".to_string()));
        assert_eq!(config.get("ports"), Some(&"8080:80".to_string()));
        assert!(config.get("environment").unwrap().contains("KEY=value"));
    }

    #[test]
    fn test_build_configuration() {
        let service = Service::new("app")
            .build_context(".")
            .build_arg("NODE_ENV", "production")
            .build_target("production")
            .build_cache_from("node:18-alpine");

        let config = service.to_config_map();
        assert_eq!(config.get("build_context"), Some(&".".to_string()));
        assert_eq!(config.get("build_target"), Some(&"production".to_string()));
        assert!(config.get("build_args").unwrap().contains("NODE_ENV=production"));
        
        // Should not have image when build is set
        assert!(service.image.is_none());
    }

    #[test]
    fn test_resource_limits() {
        let service = Service::new("app")
            .memory("512m")
            .memory_reservation("256m")
            .cpus("0.5")
            .cpu_shares(512);

        let config = service.to_config_map();
        assert_eq!(config.get("memory"), Some(&"512m".to_string()));
        assert_eq!(config.get("memory_reservation"), Some(&"256m".to_string()));
        assert_eq!(config.get("cpus"), Some(&"0.5".to_string()));
        assert_eq!(config.get("cpu_shares"), Some(&"512".to_string()));
    }

    #[test]
    fn test_advanced_ports() {
        let service = Service::new("app")
            .port_advanced(8080, Some(80), Some("tcp".to_string()), Some("ingress".to_string()));

        let config = service.to_config_map();
        assert!(config.get("advanced_ports").unwrap().contains("80:8080:tcp"));
    }

    #[test]
    fn test_environment_files_and_secrets() {
        let service = Service::new("app")
            .env_file(".env")
            .secret("db_password");

        let config = service.to_config_map();
        assert_eq!(config.get("env_files"), Some(&".env".to_string()));
        assert_eq!(config.get("secrets"), Some(&"db_password".to_string()));
    }

    #[test]
    fn test_healthcheck() {
        let service = Service::new("app")
            .healthcheck(
                vec!["CMD".to_string(), "curl".to_string(), "-f".to_string(), "http://localhost:8080/health".to_string()],
                Some("30s".to_string()),
                Some("10s".to_string()),
                Some(3),
                Some("40s".to_string())
            );

        let config = service.to_config_map();
        assert!(config.get("healthcheck_test").unwrap().contains("curl -f http://localhost:8080/health"));
    }
}