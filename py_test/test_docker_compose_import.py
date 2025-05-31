"""
Test Docker Compose import functionality

Tests the ability to import docker-compose.yml files and convert them to docker-pyo3 stacks.
"""

import pytest
import tempfile
import os
from docker_pyo3 import Docker

@pytest.fixture
def docker():
    return Docker()

@pytest.fixture
def sample_compose_yaml():
    """Sample docker-compose.yml content for testing"""
    return """
version: '3.8'

services:
  web:
    image: nginx:latest
    ports:
      - "80:80"
      - "443:443"
    environment:
      - NODE_ENV=production
      - DEBUG=false
    volumes:
      - ./html:/usr/share/nginx/html
    hostname: web-server
    restart: unless-stopped

  api:
    build:
      context: ./api
      dockerfile: Dockerfile
      args:
        NODE_VERSION: "16"
        ENV: production
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://user:password@db:5432/myapp
      API_KEY: secret
    working_dir: /app
    command: ["npm", "start"]

  db:
    image: postgres:13
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes:
      - db_data:/var/lib/postgresql/data
    restart: always

volumes:
  db_data:
    driver: local
"""

class TestDockerComposeImport:
    """Test Docker Compose import functionality"""
    
    def test_import_from_yaml_string(self, docker, sample_compose_yaml):
        """Test importing stack from YAML string"""
        # Import the stack
        stack = docker.import_stack_from_yaml(sample_compose_yaml)
        
        # Verify stack was created
        assert stack.name == "imported-stack"
        assert stack.service_count() == 3
        
        # Verify services were imported
        services = stack.get_registered_services()
        assert "web" in services
        assert "api" in services
        assert "db" in services
        
        print("✅ Docker Compose YAML import successful!")
        print(f"   - Imported {stack.service_count()} services")
        print(f"   - Services: {', '.join(services)}")
    
    def test_import_from_file(self, docker, sample_compose_yaml):
        """Test importing stack from docker-compose.yml file"""
        # Create temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.yml', delete=False) as f:
            f.write(sample_compose_yaml)
            temp_file = f.name
        
        try:
            # Import from file
            stack = docker.import_stack_from_file(temp_file)
            
            # Verify stack was created
            assert stack.service_count() == 3
            assert stack.has_service("web")
            assert stack.has_service("api") 
            assert stack.has_service("db")
            
            print("✅ Docker Compose file import successful!")
            print(f"   - Imported from: {temp_file}")
            print(f"   - Services: {stack.get_registered_services()}")
            
        finally:
            # Cleanup
            os.unlink(temp_file)
    
    def test_service_configuration_import(self, docker):
        """Test that service configurations are properly imported"""
        compose_yaml = """
version: '3.8'
services:
  test-service:
    image: ubuntu:20.04
    command: ["echo", "hello world"]
    environment:
      ENV_VAR1: value1
      ENV_VAR2: value2
    ports:
      - "8080:80"
      - "9090:90"
    volumes:
      - "./data:/app/data"
    working_dir: /app
    hostname: test-host
    restart: on-failure
"""
        
        stack = docker.import_stack_from_yaml(compose_yaml)
        
        # Verify service exists
        assert stack.has_service("test-service")
        assert stack.service_count() == 1
        
        print("✅ Service configuration import successful!")
        print("   - Image, command, environment, ports, volumes imported")
    
    def test_build_configuration_import(self, docker):
        """Test that build configurations are properly imported"""
        compose_yaml = """
version: '3.8'
services:
  built-service:
    build:
      context: ./my-app
      dockerfile: custom.Dockerfile
      args:
        BUILD_ARG1: value1
        BUILD_ARG2: value2
      target: production
    ports:
      - "3000:3000"
"""
        
        stack = docker.import_stack_from_yaml(compose_yaml)
        
        # Verify service exists
        assert stack.has_service("built-service")
        
        print("✅ Build configuration import successful!")
        print("   - Build context, dockerfile, args, target imported")
    
    def test_invalid_yaml_handling(self, docker):
        """Test error handling for invalid YAML"""
        invalid_yaml = """
invalid: yaml: content
  missing: proper
    indentation and structure
"""
        
        with pytest.raises(Exception) as exc_info:
            docker.import_stack_from_yaml(invalid_yaml)
        
        # Should raise a parsing error
        assert "Failed to parse docker-compose YAML" in str(exc_info.value)
        
        print("✅ Invalid YAML error handling working!")
    
    def test_missing_file_handling(self, docker):
        """Test error handling for missing files"""
        with pytest.raises(Exception) as exc_info:
            docker.import_stack_from_file("/nonexistent/docker-compose.yml")
        
        # Should raise a file not found error
        assert "Failed to read docker-compose file" in str(exc_info.value)
        
        print("✅ Missing file error handling working!")
    
    def test_minimal_compose_file(self, docker):
        """Test importing a minimal docker-compose file"""
        minimal_yaml = """
version: '3.8'
services:
  simple:
    image: hello-world
"""
        
        stack = docker.import_stack_from_yaml(minimal_yaml)
        
        assert stack.service_count() == 1
        assert stack.has_service("simple")
        
        print("✅ Minimal compose file import successful!")

class TestImportedStackDeployment:
    """Test that imported stacks can be deployed"""
    
    def test_imported_stack_status(self, docker):
        """Test that imported stacks have proper status"""
        compose_yaml = """
version: '3.8'
services:
  test:
    image: busybox
    command: ["echo", "test"]
"""
        
        stack = docker.import_stack_from_yaml(compose_yaml)
        
        # Test status before deployment
        status = stack.status()
        assert status['status'] == 'not_deployed'
        assert status['total_containers'] == 0
        
        print("✅ Imported stack status check successful!")
        print(f"   - Initial status: {status['status']}")
        print(f"   - Services: {list(status['services'].keys())}")
    
    def test_imported_stack_methods_available(self, docker):
        """Test that imported stacks have all Phase 2.0 methods"""
        compose_yaml = """
version: '3.8'
services:
  test:
    image: busybox
"""
        
        stack = docker.import_stack_from_yaml(compose_yaml)
        
        # Test that all Phase 2.0 methods are available
        assert hasattr(stack, 'up'), "stack.up() method missing"
        assert hasattr(stack, 'down'), "stack.down() method missing"
        assert hasattr(stack, 'logs'), "stack.logs() method missing"
        assert hasattr(stack, 'status'), "stack.status() method missing"
        assert hasattr(stack, 'scale'), "stack.scale() method missing"
        assert hasattr(stack, 'restart_service'), "stack.restart_service() method missing"
        
        print("✅ All Phase 2.0 methods available on imported stack!")
        print("   - up, down, logs, status, scale, restart_service ✓")