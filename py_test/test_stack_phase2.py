"""
Test Phase 2.0 Stack Deployment functionality for docker-pyo3

Tests the Phase 2.0 features: stack deployment (up/down), service management, and logs.
"""

import pytest
import time
from docker_pyo3 import Docker

@pytest.fixture
def docker():
    return Docker()

@pytest.fixture  
def test_stack(docker):
    """Create a test stack with some simple services"""
    # Clean up any existing network before starting
    try:
        network = docker.networks().get("test-stack_default")
        network.delete()
    except:
        pass
    
    stack = docker.create_stack("test-stack")
    
    # Create a simple web service
    web_service = docker.create_service("web")
    web_service.image("busybox")
    web_service.command(["sh", "-c", "while true; do echo 'web service running'; sleep 5; done"])
    stack.register_service(web_service)
    
    # Create a simple app service  
    app_service = docker.create_service("app")
    app_service.image("busybox")
    app_service.command(["sh", "-c", "while true; do echo 'app service running'; sleep 3; done"])
    stack.register_service(app_service)
    
    yield stack
    
    # Cleanup
    try:
        stack.down()
    except:
        pass

class TestStackPhase2Deployment:
    """Test Phase 2.0 stack deployment operations"""
    
    def test_stack_up_down(self, test_stack):
        """Test basic stack deployment and teardown"""
        # Initially no services running
        assert test_stack.service_count() == 2
        
        # Deploy the stack
        test_stack.up()
        time.sleep(2)  # Give services time to start
        
        # Check status
        status = test_stack.status()
        assert status['status'] == 'running'
        assert status['total_containers'] == 2
        assert 'web' in status['services']
        assert 'app' in status['services']
        assert status['services']['web']['replicas'] == 1
        assert status['services']['app']['replicas'] == 1
        
        # Bring down the stack
        test_stack.down()
        time.sleep(1)
        
        # Check status after down
        status = test_stack.status()
        assert status['status'] == 'not_deployed'
        assert status['total_containers'] == 0
    
    def test_stack_logs(self, test_stack):
        """Test getting logs from stack services"""
        # Deploy and let it run for a bit
        test_stack.up()
        time.sleep(3)
        
        # Get logs from all services
        logs = test_stack.logs()
        assert "[web]" in logs
        assert "[app]" in logs
        assert "web service running" in logs
        assert "app service running" in logs
        
        # Get logs from specific service
        web_logs = test_stack.logs(["web"])
        assert "[web]" in web_logs
        assert "[app]" not in web_logs
        
        # Cleanup
        test_stack.down()
    
    def test_service_scaling(self, test_stack):
        """Test scaling services up and down"""
        # Deploy initial stack
        test_stack.up()
        time.sleep(1)
        
        # Check initial state
        status = test_stack.status()
        assert status['services']['web']['replicas'] == 1
        
        # Scale web service to 3 replicas
        test_stack.scale("web", 3)
        time.sleep(1)
        
        # Check scaled state
        status = test_stack.status()
        assert status['services']['web']['replicas'] == 3
        assert status['total_containers'] == 4  # 3 web + 1 app
        
        # Scale back down to 1
        test_stack.scale("web", 1)
        time.sleep(1)
        
        # Check scaled down state
        status = test_stack.status()
        assert status['services']['web']['replicas'] == 1
        assert status['total_containers'] == 2  # 1 web + 1 app
        
        # Cleanup
        test_stack.down()
    
    def test_service_restart(self, test_stack):
        """Test restarting individual services"""
        # Deploy stack
        test_stack.up()
        time.sleep(1)
        
        # Get initial status
        initial_status = test_stack.status()
        assert initial_status['services']['web']['replicas'] == 1
        
        # Restart web service
        test_stack.restart_service("web")
        time.sleep(1)
        
        # Check that service is still running
        status = test_stack.status()
        assert status['services']['web']['replicas'] == 1
        assert status['total_containers'] == 2
        
        # Cleanup
        test_stack.down()
    
    def test_stack_error_handling(self, test_stack):
        """Test error handling in stack operations"""
        # Try to scale non-existent service
        test_stack.up()
        
        with pytest.raises(Exception):
            test_stack.scale("nonexistent", 2)
        
        with pytest.raises(Exception):
            test_stack.restart_service("nonexistent")
        
        # Cleanup
        test_stack.down()

class TestStackServiceRegistration:
    """Test service registration and management within stacks"""
    
    def test_service_registration(self, docker):
        """Test registering and unregistering services"""
        stack = docker.create_stack("reg-test")
        
        # Create a service
        service = docker.create_service("test-service")
        service.image("busybox")
        
        # Register it
        stack.register_service(service)
        assert stack.service_count() == 1
        assert stack.has_service("test-service")
        assert "test-service" in stack.get_registered_services()
        
        # Try to register duplicate (should fail)
        with pytest.raises(Exception):
            stack.register_service(service)
        
        # Unregister it
        assert stack.unregister_service("test-service") is True
        assert stack.service_count() == 0
        assert not stack.has_service("test-service")
        
        # Try to unregister non-existent (should return False)
        assert stack.unregister_service("nonexistent") is False