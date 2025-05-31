#!/usr/bin/env python3
"""Test enhanced container parameters"""

import pytest
from docker_pyo3 import Docker

@pytest.fixture
def docker():
    return Docker()

@pytest.fixture
def cleanup(docker):
    """Cleanup any test containers"""
    containers_to_cleanup = []
    
    # Clean up any existing containers before the test
    for container_name in ["test-volumes", "test-env", "test-restart", "test-hosts", "test-workdir", "test-labels", "test-entrypoint", "test-user"]:
        try:
            c = docker.containers().get(container_name)
            c.stop()
            c.remove(force=True)
        except:
            pass
    
    yield containers_to_cleanup
    
    # Clean up containers created during the test
    for container_name in containers_to_cleanup:
        try:
            c = docker.containers().get(container_name)
            c.stop()
            c.remove(force=True)
        except:
            pass

class TestEnhancedContainerParams:
    """Test newly implemented container creation parameters"""
    
    def test_container_with_environment_variables(self, docker, cleanup):
        """Test creating container with environment variables"""
        cleanup.append("test-env")
        
        c = docker.containers().create(
            image="busybox",
            name="test-env",
            env=["MY_VAR=hello", "ANOTHER_VAR=world"],
            command=["sh", "-c", "echo $MY_VAR $ANOTHER_VAR && sleep 1"],
            auto_remove=True
        )
        c.start()
        c.wait()
        logs = c.logs()
        
        assert "hello world" in logs
    
    def test_container_with_restart_policy(self, docker, cleanup):
        """Test creating container with restart policy"""
        cleanup.append("test-restart")
        
        c = docker.containers().create(
            image="busybox", 
            name="test-restart",
            restart_policy={"name": "on-failure", "maximum_retry_count": 3},
            command=["sh", "-c", "echo 'Hello with restart policy'"],
        )
        
        info = c.inspect()
        restart_policy = info.get('HostConfig', {}).get('RestartPolicy', {})
        
        assert restart_policy.get('Name') == 'on-failure'
        assert restart_policy.get('MaximumRetryCount') == 3
    
    def test_container_with_extra_hosts(self, docker, cleanup):
        """Test creating container with extra hosts entries"""
        cleanup.append("test-hosts")
        
        c = docker.containers().create(
            image="busybox",
            name="test-hosts",
            extra_hosts=["myhost:127.0.0.1", "another:192.168.1.1"],
            command=["cat", "/etc/hosts"]
        )
        c.start()
        c.wait()
        logs = c.logs()
        
        assert "myhost" in logs
        assert "127.0.0.1" in logs
        assert "another" in logs
        assert "192.168.1.1" in logs
    
    def test_container_with_working_directory(self, docker, cleanup):
        """Test creating container with custom working directory"""
        cleanup.append("test-workdir")
        
        c = docker.containers().create(
            image="busybox",
            name="test-workdir",
            working_dir="/tmp",
            command=["pwd"]
        )
        c.start()
        c.wait()
        logs = c.logs().strip()
        
        assert logs == "/tmp"
    
    def test_container_with_labels(self, docker, cleanup):
        """Test creating container with labels"""
        cleanup.append("test-labels")
        
        labels = {
            "app": "test",
            "version": "1.0",
            "environment": "testing"
        }
        
        c = docker.containers().create(
            image="busybox",
            name="test-labels",
            labels=labels,
            command=["sleep", "1"]
        )
        
        info = c.inspect()
        container_labels = info.get('Config', {}).get('Labels', {})
        
        assert container_labels.get('app') == 'test'
        assert container_labels.get('version') == '1.0'
        assert container_labels.get('environment') == 'testing'
    
    def test_container_with_entrypoint(self, docker, cleanup):
        """Test creating container with custom entrypoint"""
        cleanup.append("test-entrypoint")
        
        c = docker.containers().create(
            image="busybox",
            name="test-entrypoint",
            entrypoint=["echo"],
            command=["Hello from entrypoint"]
        )
        c.start()
        c.wait()
        logs = c.logs().strip()
        
        assert logs == "Hello from entrypoint"
    
    def test_container_with_volumes(self, docker, cleanup):
        """Test creating container with volumes"""
        cleanup.append("test-volumes")
        
        # Create a named volume first
        volume_name = "test-data-volume"
        try:
            docker.volumes().create(name=volume_name)
        except:
            pass  # Volume might already exist
        
        c = docker.containers().create(
            image="busybox",
            name="test-volumes",
            volumes=[f"{volume_name}:/data"],
            command=["sh", "-c", "echo 'test data' > /data/test.txt && cat /data/test.txt"]
        )
        c.start()
        c.wait()
        logs = c.logs().strip()
        
        assert logs == "test data"
        
        # Cleanup volume
        try:
            docker.volumes().get(volume_name).delete()
        except:
            pass
    
    def test_container_with_user(self, docker, cleanup):
        """Test creating container with specific user"""
        cleanup.append("test-user")
        
        c = docker.containers().create(
            image="busybox",
            name="test-user",
            user="1000:1000",
            command=["id"]
        )
        c.start()
        c.wait()
        logs = c.logs()
        
        assert "uid=1000" in logs
        assert "gid=1000" in logs
    
    def test_restart_policy_validation(self, docker):
        """Test restart policy validation"""
        with pytest.raises(Exception) as exc_info:
            docker.containers().create(
                image="busybox",
                name="test-invalid-restart",
                restart_policy={"name": "invalid-policy"}
            )
        
        assert "Invalid restart policy" in str(exc_info.value)
    
    def test_port_publish_validation(self, docker):
        """Test port publish parameter validation"""
        # This should validate but not actually work without full implementation
        with pytest.raises(Exception) as exc_info:
            docker.containers().create(
                image="busybox",
                name="test-invalid-port",
                publish=["invalid-port-format"]
            )
        
        assert "Invalid port mapping format" in str(exc_info.value)