"""
Demo test for Phase 2.0 Stack Deployment functionality

A simple test to demonstrate that Phase 2.0 stack functionality compiles and basic operations work.
"""

import pytest
from docker_pyo3 import Docker

@pytest.fixture
def docker():
    return Docker()

def test_phase2_stack_basic_functionality(docker):
    """Test that Phase 2.0 stack functionality works at a basic level"""
    # Create a stack
    stack = docker.create_stack("demo-stack")
    
    # Verify initial state
    assert stack.service_count() == 0
    assert stack.name == "demo-stack"
    
    # Create and register a service
    service = docker.create_service("demo-service")
    service.image("busybox")
    service.command(["echo", "hello world"])
    
    stack.register_service(service)
    
    # Verify service registration
    assert stack.service_count() == 1
    assert stack.has_service("demo-service")
    assert "demo-service" in stack.get_registered_services()
    
    # Test status when not deployed
    status = stack.status()
    assert status['status'] == 'not_deployed'
    assert status['total_containers'] == 0
    
    print("✅ Phase 2.0 Stack basic functionality test passed!")
    print(f"   - Stack created: {stack.name}")
    print(f"   - Services registered: {stack.service_count()}")
    print(f"   - Initial status: {status['status']}")

def test_phase2_stack_methods_exist(docker):
    """Test that all Phase 2.0 methods exist and are callable"""
    stack = docker.create_stack("method-test-stack")
    
    # Test that all Phase 2.0 methods exist
    assert hasattr(stack, 'up'), "stack.up() method missing"
    assert hasattr(stack, 'down'), "stack.down() method missing"
    assert hasattr(stack, 'logs'), "stack.logs() method missing"
    assert hasattr(stack, 'status'), "stack.status() method missing"
    assert hasattr(stack, 'scale'), "stack.scale() method missing"
    assert hasattr(stack, 'restart_service'), "stack.restart_service() method missing"
    
    print("✅ All Phase 2.0 methods are available!")
    print("   - stack.up() ✓")
    print("   - stack.down() ✓")
    print("   - stack.logs() ✓")
    print("   - stack.status() ✓")
    print("   - stack.scale() ✓")
    print("   - stack.restart_service() ✓")