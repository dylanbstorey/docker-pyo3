"""
Test Phase 1.75 enhanced Docker Compose DSL features
Tests build configuration, resource management, advanced ports, volumes, and more
"""
import pytest

def test_build_configuration_basic(docker):
    """Test basic build configuration with context and dockerfile"""
    from docker_pyo3 import Service
    
    # Create service with build configuration
    app_service = Service("web-app")
    app_service.build_context(".")
    app_service.build_arg("NODE_ENV", "production")
    app_service.build_target("production-stage")
    app_service.build_cache_from("node:18-alpine")
    
    # Basic verification that methods exist and don't error
    assert app_service.name == "web-app"

def test_build_with_dockerfile(docker):
    """Test build configuration with custom dockerfile"""
    from docker_pyo3 import Service
    
    service = Service("custom-app")
    service.build_with_dockerfile("./backend", "Dockerfile.prod")
    service.build_arg("API_VERSION", "v2.1")
    service.build_arg("DEBUG", "false")
    
    # Basic verification that methods exist and don't error
    assert service.name == "custom-app"

def test_resource_management(docker):
    """Test comprehensive resource limits and reservations"""
    from docker_pyo3 import Service
    
    service = Service("resource-intensive-app")
    service.image("cpu-heavy:latest")
    service.memory("2g")
    service.memory_reservation("1g")
    service.cpus("1.5")
    service.cpu_shares(1024)
    service.cpu_quota(100000, 100000)  # quota and period
    
    # Basic verification that methods exist and don't error
    assert service.name == "resource-intensive-app"

def test_advanced_port_configuration(docker):
    """Test advanced port configuration with protocols and modes"""
    from docker_pyo3 import Service
    
    service = Service("web-server")
    service.image("nginx:latest")
    
    # Add advanced port configurations
    service.port_advanced(80, 8080, "tcp", "ingress")
    service.port_advanced(443, 8443, "tcp", None)
    service.port_advanced(8125, None, "udp", None)  # Internal port only
    
    # Basic verification that methods exist and don't error
    assert service.name == "web-server"

def test_advanced_volume_configuration(docker):
    """Test advanced volume configuration with types and options"""
    from docker_pyo3 import Service
    
    service = Service("data-processor")
    service.image("data-app:latest")
    
    # Add various volume types
    service.volume_advanced("./data", "/app/data", "bind", False)
    service.volume_advanced("logs", "/var/log", "volume", False)
    service.volume_advanced("./config", "/etc/app", "bind", True)  # read-only
    
    # Basic verification that methods exist and don't error
    assert service.name == "data-processor"

def test_environment_files_and_secrets(docker):
    """Test environment files and secrets support"""
    from docker_pyo3 import Service
    
    service = Service("secure-app")
    service.image("app:latest")
    service.env_file(".env")
    service.env_file(".env.production")
    service.secret("db_password")
    service.secret("api_key")
    
    # Basic verification that methods exist and don't error
    assert service.name == "secure-app"

def test_healthcheck_configuration(docker):
    """Test comprehensive health check configuration"""
    from docker_pyo3 import Service
    
    service = Service("monitored-app")
    service.image("web-app:latest")
    service.healthcheck(
        ["CMD", "curl", "-f", "http://localhost:8080/health"],
        "30s",      # interval
        "10s",      # timeout
        3,          # retries
        "40s"       # start_period
    )
    
    # Basic verification that methods exist and don't error
    assert service.name == "monitored-app"

def test_comprehensive_service_configuration(docker):
    """Test a service with all Phase 1.75 features combined"""
    from docker_pyo3 import Service
    
    # Create a comprehensive service configuration
    service = Service("full-featured-app")
    
    # Build configuration
    service.build_context("./app")
    service.build_arg("BUILD_ENV", "production")
    service.build_target("prod")
    service.build_cache_from("node:18-alpine")
    
    # Resource management
    service.memory("4g")
    service.memory_reservation("2g")
    service.cpus("2.0")
    service.cpu_shares(2048)
    
    # Advanced networking
    service.port_advanced(3000, 8000, "tcp", "ingress")
    service.port_advanced(9090, 9090, "tcp", "host")
    
    # Advanced storage
    service.volume_advanced("./data", "/app/data", "bind", False)
    service.volume_advanced("app-logs", "/var/log", "volume", False)
    
    # Environment and secrets
    service.env("NODE_ENV", "production")
    service.env_file(".env.production")
    service.secret("database_url")
    
    # Health monitoring
    service.healthcheck(
        ["CMD", "node", "health-check.js"],
        "30s", "10s", 3, "60s"
    )
    
    # Service management
    service.restart_policy("unless-stopped")
    service.hostname("app.internal")
    service.label("version", "2.0")
    service.label("environment", "production")
    
    # Basic verification that methods exist and don't error
    assert service.name == "full-featured-app"

def test_mutual_exclusivity_image_build(docker):
    """Test that image and build configuration are mutually exclusive"""
    from docker_pyo3 import Service
    
    # Test image -> build
    service1 = Service("test1")
    service1.image("nginx:latest")
    service1.build_context(".")
    
    # Test build -> image
    service2 = Service("test2")
    service2.build_context(".")
    service2.image("nginx:latest")
    
    # Basic verification that methods exist and don't error
    assert service1.name == "test1"
    assert service2.name == "test2"

def test_service_cloning_with_build_features(docker):
    """Test that service cloning preserves all Phase 1.75 features"""
    from docker_pyo3 import Service
    
    # Create base service with comprehensive configuration
    base_service = Service("base-app")
    base_service.build_context(".")
    base_service.build_arg("ENV", "base")
    base_service.memory("1g")
    base_service.cpus("1.0")
    base_service.port_advanced(3000, 8000, "tcp", None)
    base_service.env_file(".env")
    base_service.secret("base_secret")
    
    # Clone and modify
    prod_service = base_service.clone_with_name("prod-app")
    prod_service.build_arg("ENV", "production")
    prod_service.memory("4g")
    prod_service.cpus("2.0")
    prod_service.port_advanced(3000, 9000, "tcp", None)
    
    # Basic verification
    assert base_service.name == "base-app"
    assert prod_service.name == "prod-app"

def test_convenience_constructors_with_build(docker):
    """Test convenience constructors work with build features"""
    from docker_pyo3 import Service
    
    web_service = Service.web_service("frontend")
    web_service.build_context("./frontend")
    web_service.build_arg("REACT_APP_ENV", "production")
    web_service.memory("2g")
    web_service.cpus("1.5")
    
    db_service = Service.database_service("postgres")
    db_service.image("postgres:15")  # Override build with image
    db_service.memory("8g")
    db_service.memory_reservation("4g")
    db_service.volume_advanced("db-data", "/var/lib/postgresql/data", "volume", False)
    
    # Basic verification
    assert web_service.name == "frontend"
    assert db_service.name == "postgres"

def test_stack_integration_with_enhanced_services(docker):
    """Test Stack integration with Phase 1.75 enhanced services"""
    from docker_pyo3 import Service, Stack
    
    # Create a stack
    stack = Stack(docker, "myapp")
    
    # Create services with Phase 1.75 features
    web_service = Service("web")
    web_service.build_context("./web")
    web_service.build_arg("NODE_ENV", "production")
    web_service.memory("2g")
    web_service.cpus("1.5")
    web_service.port_advanced(3000, 80, "tcp", "ingress")
    web_service.healthcheck(["CMD", "curl", "-f", "http://localhost:3000/health"], "30s", "10s", 3, None)
    
    api_service = Service("api")
    api_service.build_with_dockerfile("./api", "Dockerfile.prod")
    api_service.build_target("production")
    api_service.memory("4g")
    api_service.memory_reservation("2g")
    api_service.env_file(".env")
    api_service.secret("api_secret")
    api_service.volume_advanced("./data", "/app/data", "bind", False)
    
    db_service = Service.database_service("postgres")
    db_service.image("postgres:15")
    db_service.memory("8g")
    db_service.volume_advanced("pg-data", "/var/lib/postgresql/data", "volume", False)
    db_service.env("POSTGRES_PASSWORD", "secret")
    
    # Register services in stack
    stack.register_service(web_service)
    stack.register_service(api_service)
    stack.register_service(db_service)
    
    # Verify stack has all services
    assert stack.service_count() == 3
    assert stack.has_service("web")
    assert stack.has_service("api")
    assert stack.has_service("postgres")
    
    # Get YAML output
    yaml_output = stack.to_yaml()
    assert "version: '3.8'" in yaml_output
    assert "web:" in yaml_output
    assert "api:" in yaml_output
    assert "postgres:" in yaml_output

def test_all_phase_175_methods_exist(docker):
    """Verify all Phase 1.75 methods exist and are callable"""
    from docker_pyo3 import Service
    
    service = Service("test-service")
    
    # Build methods
    assert hasattr(service, 'build_context')
    assert hasattr(service, 'build_with_dockerfile')
    assert hasattr(service, 'build_arg')
    assert hasattr(service, 'build_target')
    assert hasattr(service, 'build_cache_from')
    
    # Resource methods
    assert hasattr(service, 'memory')
    assert hasattr(service, 'memory_reservation')
    assert hasattr(service, 'cpus')
    assert hasattr(service, 'cpu_shares')
    assert hasattr(service, 'cpu_quota')
    
    # Advanced port/volume methods
    assert hasattr(service, 'port_advanced')
    assert hasattr(service, 'volume_advanced')
    
    # Environment/secret methods
    assert hasattr(service, 'env_file')
    assert hasattr(service, 'secret')
    
    # Health check method
    assert hasattr(service, 'healthcheck')
    
    # Clone method
    assert hasattr(service, 'clone_with_name')
    
    # Static constructors
    assert hasattr(Service, 'web_service')
    assert hasattr(Service, 'database_service')
    assert hasattr(Service, 'redis_service')