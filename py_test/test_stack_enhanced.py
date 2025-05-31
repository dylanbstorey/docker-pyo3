"""
Test enhanced stack functionality with composable Services for docker-pyo3

Tests both the new independent Service class and Stack.register_service() functionality.
"""

import pytest
from docker_pyo3 import Docker, Stack, Service

@pytest.fixture
def docker():
    return Docker()

class TestServiceIndependent:
    """Test independent Service class functionality"""
    
    def test_service_creation(self):
        """Test creating independent services"""
        service = Service("web")
        assert service.name == "web"
    
    def test_service_fluent_api(self):
        """Test fluent API for service configuration"""
        service = Service("api")
        service.image("fastapi:latest")
        service.ports(["8080:8000"])
        service.env("ENV", "production")
        service.volume("./logs:/app/logs")
        service.network("app_network")
        service.depends_on_service("db")
        service.restart_policy("unless-stopped")
        service.hostname("api-server")
        service.label("tier", "backend")
        service.replicas(3)
        service.memory("1GB")
        
        assert service.name == "api"
    
    def test_service_convenience_constructors(self):
        """Test convenience constructors for common service types"""
        web = Service.web_service("frontend")
        db = Service.database_service("postgres")
        cache = Service.redis_service("redis")
        
        assert web.name == "frontend"
        assert db.name == "postgres" 
        assert cache.name == "redis"
    
    def test_service_cloning(self):
        """Test cloning services with new names"""
        base_service = Service.web_service("web")
        base_service.image("nginx:latest")
        base_service.ports(["80:80"])
        
        dev_service = base_service.clone_with_name("web-dev")
        
        assert base_service.name == "web"
        assert dev_service.name == "web-dev"
        
        # Both should maintain the same configuration
        # (This would need more sophisticated testing once we can inspect config)

class TestStackServiceRegistration:
    """Test Stack service registration functionality"""
    
    def test_register_single_service(self, docker):
        """Test registering a single service to stack"""
        stack = Stack(docker, "test-stack")
        
        web_service = Service("web")
        web_service.image("nginx:latest")
        
        stack.register_service(web_service)
        
        assert stack.service_count() == 1
        assert stack.has_service("web") == True
        assert "web" in stack.get_registered_services()
    
    def test_register_multiple_services(self, docker):
        """Test registering multiple services to stack"""
        stack = Stack(docker, "multi-service")
        
        # Create multiple services
        web = Service("web")
        web.image("nginx:latest")
        
        api = Service("api") 
        api.image("fastapi:latest")
        
        db = Service("db")
        db.image("postgres:13")
        
        # Register all services
        stack.register_service(web)
        stack.register_service(api)
        stack.register_service(db)
        
        assert stack.service_count() == 3
        assert stack.has_service("web") == True
        assert stack.has_service("api") == True
        assert stack.has_service("db") == True
        
        services = stack.get_registered_services()
        assert "web" in services
        assert "api" in services
        assert "db" in services
    
    def test_register_duplicate_service_fails(self, docker):
        """Test that registering duplicate service names fails"""
        stack = Stack(docker, "duplicate-test")
        
        service1 = Service("web")
        service1.image("nginx:latest")
        
        service2 = Service("web")  # Same name
        service2.image("apache:latest")
        
        # First registration should succeed
        stack.register_service(service1)
        assert stack.service_count() == 1
        
        # Second registration should fail
        with pytest.raises(ValueError):
            stack.register_service(service2)
        
        # Should still have only one service
        assert stack.service_count() == 1
    
    def test_unregister_service(self, docker):
        """Test unregistering services from stack"""
        stack = Stack(docker, "unregister-test")
        
        web = Service("web")
        web.image("nginx:latest")
        stack.register_service(web)
        
        assert stack.service_count() == 1
        assert stack.has_service("web") == True
        
        # Unregister the service
        removed = stack.unregister_service("web")
        assert removed == True
        assert stack.service_count() == 0
        assert stack.has_service("web") == False
        
        # Try to unregister non-existent service
        removed = stack.unregister_service("nonexistent")
        assert removed == False

class TestStackYAMLGeneration:
    """Test YAML generation from registered services"""
    
    def test_empty_stack_yaml(self, docker):
        """Test YAML generation for empty stack"""
        stack = Stack(docker, "empty-stack")
        yaml_output = stack.to_yaml()
        
        assert "version: '3.8'" in yaml_output
        assert "services:" in yaml_output
    
    def test_single_service_yaml(self, docker):
        """Test YAML generation for single service"""
        stack = Stack(docker, "single-service")
        
        web = Service("web")
        web.image("nginx:latest")
        web.ports(["80:80"])
        web.env("ENV", "production")
        
        stack.register_service(web)
        yaml_output = stack.to_yaml()
        
        assert "version: '3.8'" in yaml_output
        assert "services:" in yaml_output
        assert "web:" in yaml_output
        assert "image: nginx:latest" in yaml_output
        assert "ports:" in yaml_output
        assert '"80:80"' in yaml_output
    
    def test_multi_service_yaml(self, docker):
        """Test YAML generation for multiple services"""
        stack = Stack(docker, "multi-service")
        
        web = Service("web")
        web.image("nginx:latest")
        web.ports(["80:80"])
        
        api = Service("api")
        api.image("fastapi:latest")
        api.ports(["8080:8000"])
        api.depends_on_service("db")
        
        db = Service("db")
        db.image("postgres:13")
        db.env("POSTGRES_PASSWORD", "secret")
        
        stack.register_service(web)
        stack.register_service(api)
        stack.register_service(db)
        
        yaml_output = stack.to_yaml()
        
        # Check that all services are present
        assert "web:" in yaml_output
        assert "api:" in yaml_output
        assert "db:" in yaml_output
        
        # Check specific configurations
        assert "image: nginx:latest" in yaml_output
        assert "image: fastapi:latest" in yaml_output
        assert "image: postgres:13" in yaml_output
        assert "depends_on:" in yaml_output

class TestComposableWorkflows:
    """Test real-world composable service workflows"""
    
    def test_reusable_service_templates(self, docker):
        """Test creating reusable service templates"""
        # Create base templates
        base_web = Service.web_service("web")
        base_web.image("nginx:latest")
        base_web.restart_policy("unless-stopped")
        
        base_db = Service.database_service("db")
        base_db.image("postgres:13")
        base_db.env("POSTGRES_PASSWORD", "secret")
        
        # Create environment-specific stacks
        dev_stack = Stack(docker, "dev")
        prod_stack = Stack(docker, "prod")
        
        # Customize for dev
        dev_web = base_web.clone_with_name("web-dev")
        dev_web.ports(["8080:80"])
        dev_web.env("ENV", "development")
        
        dev_db = base_db.clone_with_name("db-dev")
        dev_db.env("POSTGRES_DB", "dev_db")
        
        # Customize for prod
        prod_web = base_web.clone_with_name("web-prod")
        prod_web.ports(["443:80"])
        prod_web.env("ENV", "production")
        prod_web.replicas(3)
        
        prod_db = base_db.clone_with_name("db-prod")
        prod_db.env("POSTGRES_DB", "prod_db")
        prod_db.memory("4GB")
        
        # Register to respective stacks
        dev_stack.register_service(dev_web)
        dev_stack.register_service(dev_db)
        
        prod_stack.register_service(prod_web)
        prod_stack.register_service(prod_db)
        
        # Verify both stacks have services
        assert dev_stack.service_count() == 2
        assert prod_stack.service_count() == 2
        
        assert dev_stack.has_service("web-dev")
        assert dev_stack.has_service("db-dev")
        assert prod_stack.has_service("web-prod")
        assert prod_stack.has_service("db-prod")
    
    def test_service_library_pattern(self, docker):
        """Test building a library of reusable services"""
        # Service library - common services that can be reused
        nginx_web = Service.web_service("web")
        nginx_web.image("nginx:latest")
        nginx_web.ports(["80:80"])
        
        postgres_db = Service.database_service("db")
        postgres_db.image("postgres:13")
        postgres_db.env("POSTGRES_PASSWORD", "secret")
        
        redis_cache = Service.redis_service("cache")  # Uses built-in defaults
        
        # Different application stacks using the library
        blog_stack = Stack(docker, "blog")
        blog_stack.register_service(nginx_web.clone_with_name("blog-web"))
        blog_stack.register_service(postgres_db.clone_with_name("blog-db"))
        
        ecommerce_stack = Stack(docker, "ecommerce")
        ecommerce_stack.register_service(nginx_web.clone_with_name("shop-web"))
        ecommerce_stack.register_service(postgres_db.clone_with_name("shop-db"))
        ecommerce_stack.register_service(redis_cache.clone_with_name("shop-cache"))
        
        # Verify stacks are independent but use same templates
        assert blog_stack.service_count() == 2
        assert ecommerce_stack.service_count() == 3
        
        assert blog_stack.has_service("blog-web")
        assert ecommerce_stack.has_service("shop-web")
        assert ecommerce_stack.has_service("shop-cache")
        assert not blog_stack.has_service("shop-cache")

class TestHybridAPI:
    """Test mixing registered services with other Stack functionality"""
    
    def test_registered_services_with_existing_features(self, docker):
        """Test that registered services work with existing Stack features"""
        stack = Stack(docker, "hybrid-test")
        
        # Register a service
        web = Service("web")
        web.image("nginx:latest")
        stack.register_service(web)
        
        # Test existing functionality still works
        assert stack.name == "hybrid-test"
        assert stack.test_docker_compose_integration() == True
        assert stack.service_count() == 1
        
        # Should be able to generate YAML
        yaml_output = stack.to_yaml()
        assert "web:" in yaml_output
        assert "nginx:latest" in yaml_output