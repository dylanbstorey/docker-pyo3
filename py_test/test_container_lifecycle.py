"""
Test container lifecycle operations for docker-pyo3

Tests the core container lifecycle methods that were missing from the original test suite.
"""

import pytest
import time
from docker_pyo3 import Docker

@pytest.fixture
def docker():
    return Docker()

@pytest.fixture
def test_container(docker):
    """Create a test container that can be started/stopped"""
    import uuid
    # Use unique name to avoid conflicts
    container_name = f"test-lifecycle-container-{uuid.uuid4().hex[:8]}"
    
    # Clean up any existing container with this name first
    try:
        existing = docker.containers().get(container_name)
        existing.stop()
        existing.remove()
    except:
        pass
    
    container = docker.containers().create(
        image="busybox",
        name=container_name,
        command=["sh", "-c", "while true; do echo hello; sleep 1; done"]
    )
    yield container
    # Cleanup
    try:
        container.stop()
    except:
        pass
    try:
        container.remove()
    except:
        pass

class TestContainerLifecycle:
    """Test container lifecycle operations"""
    
    def test_container_start_stop(self, test_container):
        """Test starting and stopping a container"""
        # Container should be created but not running
        info = test_container.inspect()
        assert info['State']['Running'] is False
        
        # Start the container
        test_container.start()
        time.sleep(1)  # Give it time to start
        
        # Check it's running
        info = test_container.inspect()
        assert info['State']['Running'] is True
        
        # Stop the container
        test_container.stop()
        time.sleep(1)  # Give it time to stop
        
        # Check it's stopped
        info = test_container.inspect()
        assert info['State']['Running'] is False
    
    def test_container_restart(self, test_container):
        """Test restarting a container"""
        # Start the container first
        test_container.start()
        time.sleep(1)
        
        # Get initial start time
        info = test_container.inspect()
        start_time_1 = info['State']['StartedAt']
        
        # Restart the container
        test_container.restart()
        time.sleep(1)
        
        # Check it's still running with a new start time
        info = test_container.inspect()
        assert info['State']['Running'] is True
        start_time_2 = info['State']['StartedAt']
        assert start_time_1 != start_time_2
    
    def test_container_pause_unpause(self, test_container):
        """Test pausing and unpausing a container"""
        # Start the container
        test_container.start()
        time.sleep(1)
        
        # Pause the container
        test_container.pause()
        time.sleep(0.5)
        
        # Check it's paused
        info = test_container.inspect()
        assert info['State']['Paused'] is True
        assert info['State']['Running'] is True  # Still running but paused
        
        # Unpause the container
        test_container.unpause()
        time.sleep(0.5)
        
        # Check it's unpaused
        info = test_container.inspect()
        assert info['State']['Paused'] is False
        assert info['State']['Running'] is True
    
    def test_container_kill(self, test_container):
        """Test killing a container"""
        # Start the container
        test_container.start()
        time.sleep(1)
        
        # Kill the container
        test_container.kill()
        time.sleep(1)
        
        # Check it's not running
        info = test_container.inspect()
        assert info['State']['Running'] is False
    
    def test_container_rename(self, docker):
        """Test renaming a container"""
        # Create a container with a specific name
        container = docker.containers().create(
            image="busybox",
            name="original-name"
        )
        
        try:
            # Rename the container
            container.rename("new-name")
            
            # Check the new name
            info = container.inspect()
            assert info['Name'] == "/new-name"
            
            # Should be able to get it by new name
            same_container = docker.containers().get("new-name")
            assert same_container.id() == container.id()
        finally:
            # Cleanup
            try:
                container.remove()
            except:
                pass
    
    def test_container_wait(self, docker):
        """Test waiting for a container to finish"""
        # Create a container that exits quickly
        container = docker.containers().create(
            image="busybox",
            command=["sh", "-c", "sleep 2 && exit 42"]
        )
        
        try:
            # Start the container
            container.start()
            
            # Wait for it to finish
            result = container.wait()
            
            # Check the exit code
            assert result['StatusCode'] == 42
        finally:
            # Cleanup
            try:
                container.remove()
            except:
                pass
    
    def test_container_exec(self, test_container):
        """Test executing commands in a running container"""
        # Start the container
        test_container.start()
        time.sleep(1)
        
        # Execute a simple command
        test_container.exec(
            command=["echo", "test exec"],
            attach_stdout=True,
            attach_stderr=True
        )
        
        # Execute with environment variables
        test_container.exec(
            command=["sh", "-c", "echo $TEST_VAR"],
            env=["TEST_VAR=hello"],
            attach_stdout=True
        )
        
        # Execute as different user
        test_container.exec(
            command=["whoami"],
            user="nobody",
            attach_stdout=True
        )
    
    def test_container_top(self, test_container):
        """Test getting process list from container"""
        # Start the container
        test_container.start()
        time.sleep(1)
        
        # Get process list
        processes = test_container.top()
        
        # Should have process information
        assert 'Processes' in processes
        assert len(processes['Processes']) > 0
        
        # Get with custom ps arguments
        processes_aux = test_container.top("aux")
        assert 'Processes' in processes_aux

class TestContainerFileOperations:
    """Test container file operations"""
    
    def test_container_copy_file_into(self, docker):
        """Test copying files into a container"""
        container = docker.containers().create(
            image="busybox",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Create a test file
            import tempfile
            with tempfile.NamedTemporaryFile(mode='w', delete=False) as f:
                f.write("Hello from host!")
                temp_file = f.name
            
            # Copy file into container
            container.copy_file_into(temp_file, "/tmp/test.txt")
            
            # Verify file was copied
            result = container.exec(
                command=["cat", "/tmp/test.txt"],
                attach_stdout=True
            )
            
            # Cleanup temp file
            import os
            os.unlink(temp_file)
        finally:
            container.stop()
            container.remove()
    
    def test_container_stat_file(self, docker):
        """Test getting file stats from container"""
        container = docker.containers().create(
            image="busybox",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Create a file in the container
            container.exec(
                command=["sh", "-c", "echo 'test' > /tmp/test.txt"],
                attach_stdout=True
            )
            
            # Get file stats
            stats = container.stat_file("/tmp/test.txt")
            assert stats is not None
        finally:
            container.stop()
            container.remove()
    
    def test_container_copy_from(self, docker):
        """Test copying files from a container"""
        container = docker.containers().create(
            image="busybox", 
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Create a file in the container
            container.exec(
                command=["sh", "-c", "echo 'Hello from container!' > /tmp/test.txt"],
                attach_stdout=True
            )
            
            # Copy file from container
            import tempfile
            with tempfile.TemporaryDirectory() as tmpdir:
                container.copy_from("/tmp/test.txt", tmpdir)
                
                # Verify file was copied
                import os
                copied_file = os.path.join(tmpdir, "test.txt")
                assert os.path.exists(copied_file)
                
                with open(copied_file, 'r') as f:
                    content = f.read()
                    assert "Hello from container!" in content
        finally:
            container.stop()
            container.remove()

class TestContainerCommit:
    """Test container commit operations"""
    
    def test_container_commit(self, docker):
        """Test committing a container to an image"""
        container = docker.containers().create(
            image="busybox",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Make changes to the container
            container.exec(
                command=["sh", "-c", "echo 'Modified!' > /modified.txt"],
                attach_stdout=True
            )
            
            # Commit the container
            image_id = container.commit(
                repository="test-commit",
                tag="latest",
                message="Test commit"
            )
            
            assert image_id is not None
            
            # Verify the image exists
            image = docker.images().get("test-commit:latest")
            image.inspect()
            
            # Cleanup - remove the committed image
            try:
                image.delete()
            except:
                pass
        finally:
            container.stop()
            container.remove()