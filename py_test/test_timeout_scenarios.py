"""
Test timeout and cancellation scenarios for docker-pyo3

Tests how the library handles timeouts, long-running operations, and cancellations.
"""

import pytest
import time
from datetime import timedelta
from docker_pyo3 import Docker

@pytest.fixture
def docker():
    return Docker()

class TestTimeoutScenarios:
    """Test various timeout scenarios"""
    
    def test_stop_with_timeout(self, docker):
        """Test stopping container with custom timeout"""
        container = docker.containers().create(
            image="busybox",
            name="test-stop-timeout",
            command=["sh", "-c", "trap 'echo Caught signal' TERM; while true; do sleep 1; done"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Stop with short timeout (converted to timedelta)
            from datetime import timedelta
            td = timedelta(seconds=2)
            container.stop(wait=td)
            
            # Container should be stopped
            info = container.inspect()
            assert info['State']['Running'] is False
        finally:
            try:
                container.remove(force=True)
            except:
                pass
    
    def test_restart_with_timeout(self, docker):
        """Test restarting container with timeout"""
        import uuid
        container_name = f"test-restart-timeout-{uuid.uuid4().hex[:8]}"
        
        container = docker.containers().create(
            image="busybox",
            name=container_name,
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Restart with timeout
            from datetime import timedelta
            td = timedelta(seconds=5)
            container.restart(wait=td)
            
            # Container should be running
            info = container.inspect()
            assert info['State']['Running'] is True
        finally:
            container.stop()
            container.remove()
    
    def test_wait_on_long_running_container(self, docker):
        """Test waiting on a container that takes time to finish"""
        container = docker.containers().create(
            image="busybox",
            name="test-wait-long",
            command=["sh", "-c", "sleep 3 && exit 42"]
        )
        
        try:
            container.start()
            
            start_time = time.time()
            result = container.wait()
            elapsed = time.time() - start_time
            
            # Should have waited at least 3 seconds
            assert elapsed >= 3
            assert result['StatusCode'] == 42
        finally:
            container.remove()
    
    def test_image_pull_timeout_behavior(self, docker):
        """Test how image pull handles slow downloads"""
        # Pull a small image to test basic timeout behavior
        # Note: We can't truly test network timeouts without network control
        
        start_time = time.time()
        result = docker.images().pull(image="busybox:latest")
        elapsed = time.time() - start_time
        
        assert result is not None
        # Just verify it completes in reasonable time
        assert elapsed < 60  # Should complete within a minute
    
    def test_build_timeout_behavior(self, docker):
        """Test how build handles long operations"""
        import tempfile
        import os
        
        with tempfile.TemporaryDirectory() as tmpdir:
            # Create Dockerfile with time-consuming operations
            dockerfile_content = """
FROM busybox
RUN echo "Starting build..." && sleep 2
RUN echo "Still building..." && sleep 2
RUN echo "Almost done..." && sleep 2
RUN echo "Build complete!"
"""
            dockerfile_path = os.path.join(tmpdir, "Dockerfile")
            with open(dockerfile_path, 'w') as f:
                f.write(dockerfile_content)
            
            start_time = time.time()
            result = docker.images().build(
                path=tmpdir,
                tag="test-timeout-build:latest"
            )
            elapsed = time.time() - start_time
            
            # Should take at least 6 seconds (3 sleeps of 2 seconds)
            assert elapsed >= 6
            assert result is not None
            
            # Cleanup
            try:
                image = docker.images().get("test-timeout-build:latest")
                image.delete()
            except:
                pass

class TestLongRunningOperations:
    """Test handling of long-running operations"""
    
    def test_exec_long_running_command(self, docker):
        """Test exec with long-running command"""
        container = docker.containers().create(
            image="busybox",
            name="test-exec-long",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Execute a command that takes time
            container.exec(
                command=["sh", "-c", "for i in 1 2 3; do echo Step $i; sleep 1; done"],
                attach_stdout=True
            )
            
            # Should complete without hanging
            # Note: Current implementation doesn't wait for completion
        finally:
            container.stop()
            container.remove()
    
    def test_copy_large_file(self, docker):
        """Test copying large files to/from container"""
        container = docker.containers().create(
            image="busybox",
            name="test-copy-large",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Create a large file in container
            container.exec(
                command=["sh", "-c", "dd if=/dev/zero of=/tmp/large.dat bs=1M count=10"],
                attach_stdout=True
            )
            
            # Copy it out
            import tempfile
            with tempfile.TemporaryDirectory() as tmpdir:
                # This tests the copy operation with larger data
                container.copy_from("/tmp/large.dat", tmpdir)
                
                import os
                copied_file = os.path.join(tmpdir, "large.dat")
                assert os.path.exists(copied_file)
                assert os.path.getsize(copied_file) > 0
        finally:
            container.stop()
            container.remove()
    
    def test_logs_from_long_running_container(self, docker):
        """Test getting logs from container running for extended time"""
        import uuid
        container_name = f"test-logs-long-{uuid.uuid4().hex[:8]}"
        
        container = docker.containers().create(
            image="busybox",
            name=container_name,
            command=["sh", "-c", "while true; do date; sleep 5; done"]
        )
        
        try:
            container.start()
            
            # Let it run for a bit
            time.sleep(20)
            
            # Get logs - should handle large amount of output
            logs = container.logs(all=True)
            lines = logs.strip().split('\n')
            
            # Should have multiple log entries
            assert len(lines) >= 4  # At least 4 entries in 20 seconds
        finally:
            container.stop()
            container.remove()

class TestCancellationScenarios:
    """Test scenarios where operations might be cancelled"""
    
    def test_remove_running_container_force(self, docker):
        """Test force removing a running container"""
        container = docker.containers().create(
            image="busybox",
            name="test-force-remove",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Force remove while running
            container.remove(force=True)
            
            # Container should be gone
            with pytest.raises(Exception):
                docker.containers().get("test-force-remove").inspect()
        except:
            # If remove failed, clean up
            try:
                container.stop()
                container.remove()
            except:
                pass
    
    def test_stop_unresponsive_container(self, docker):
        """Test stopping container that ignores signals"""
        container = docker.containers().create(
            image="busybox",
            name="test-unresponsive",
            command=["sh", "-c", "trap '' TERM; while true; do sleep 1; done"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Try to stop - it will ignore TERM signal
            from datetime import timedelta
            td = timedelta(seconds=3)
            container.stop(wait=td)
            
            # Container should eventually be killed
            info = container.inspect()
            assert info['State']['Running'] is False
        finally:
            try:
                container.remove(force=True)
            except:
                pass
    
    def test_multiple_operations_same_container(self, docker):
        """Test running multiple operations on same container"""
        container = docker.containers().create(
            image="busybox",
            name="test-concurrent-ops",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            time.sleep(1)
            
            # Run multiple operations
            container.pause()
            info1 = container.inspect()
            assert info1['State']['Paused'] is True
            
            container.unpause()
            info2 = container.inspect()
            assert info2['State']['Paused'] is False
            
            # Multiple exec operations
            for i in range(5):
                container.exec(
                    command=["echo", f"Command {i}"],
                    attach_stdout=True
                )
            
            # Get logs multiple times
            for _ in range(3):
                logs = container.logs(n_lines=10)
                assert logs is not None
        finally:
            container.stop()
            container.remove()

class TestResourceCleanupTimeouts:
    """Test resource cleanup with timeouts"""
    
    def test_prune_with_multiple_stopped_containers(self, docker):
        """Test pruning with multiple stopped containers"""
        containers = []
        
        try:
            # Create multiple containers
            for i in range(5):
                container = docker.containers().create(
                    image="busybox",
                    name=f"test-prune-{i}",
                    command=["echo", f"Container {i}"]
                )
                container.start()
                containers.append(container)
            
            # Wait for all to finish
            time.sleep(2)
            
            # Prune stopped containers
            result = docker.containers().prune()
            
            # Should have pruned containers
            assert 'ContainersDeleted' in result or 'SpaceReclaimed' in result
        finally:
            # Clean up any remaining
            for container in containers:
                try:
                    container.remove(force=True)
                except:
                    pass
    
    def test_image_prune_with_dangling(self, docker):
        """Test pruning dangling images"""
        # This is difficult to test without creating dangling images
        # Just verify the prune operation works
        result = docker.images().prune()
        
        # Should return prune info even if nothing pruned
        assert result is not None
        assert 'SpaceReclaimed' in result or 'Deleted' in result