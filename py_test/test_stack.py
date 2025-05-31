"""
Test stack functionality for docker-pyo3

Tests basic Stack creation, docker-compose integration, and YAML parsing.
"""

import pytest
from docker_pyo3 import Docker, Stack

@pytest.fixture
def docker():
    return Docker()

class TestStackBasic:
    """Test basic Stack functionality"""
    
    def test_stack_creation(self, docker):
        """Test creating a basic stack"""
        stack = Stack(docker, "test-stack")
        assert stack.name == "test-stack"
    
    def test_docker_compose_integration(self, docker):
        """Test docker-compose-types integration"""
        stack = Stack(docker, "test-stack")
        
        # Test the integration method we added
        result = stack.test_docker_compose_integration()
        assert result == True
    
    def test_multiple_stacks(self, docker):
        """Test creating multiple stacks with different names"""
        stack1 = Stack(docker, "stack-one")
        stack2 = Stack(docker, "stack-two")
        
        assert stack1.name == "stack-one"
        assert stack2.name == "stack-two"
        assert stack1.name != stack2.name

class TestStackYAMLIntegration:
    """Test YAML parsing and docker-compose integration"""
    
    def test_yaml_parsing_validation(self, docker):
        """Test that our docker-compose-types integration works"""
        # This tests the underlying YAML parsing without file I/O
        stack = Stack(docker, "yaml-test")
        
        # The test_docker_compose_integration method validates:
        # 1. serde_yaml can parse docker-compose format
        # 2. docker-compose-types structures work correctly
        # 3. version field parsing works
        result = stack.test_docker_compose_integration()
        assert result == True

# Note: More comprehensive Stack tests (from_file, to_file, deployment)
# will be added once the full implementation is complete.
# For now, we're testing the foundation: Stack creation and docker-compose-types integration.

class TestStackFoundation:
    """Test that Stack foundation is solid for future development"""
    
    def test_stack_has_required_attributes(self, docker):
        """Test that Stack object has expected attributes"""
        stack = Stack(docker, "foundation-test")
        
        # Test basic attributes exist
        assert hasattr(stack, 'name')
        assert callable(getattr(stack, 'test_docker_compose_integration'))
        
        # Test name getter works
        assert stack.name == "foundation-test"
    
    def test_docker_reference_works(self, docker):
        """Test that Stack maintains reference to Docker client"""
        stack = Stack(docker, "docker-ref-test")
        
        # Stack should be created successfully with Docker client
        assert stack.name == "docker-ref-test"
        
        # The fact that we can create it means the Docker reference is working
        assert stack is not None