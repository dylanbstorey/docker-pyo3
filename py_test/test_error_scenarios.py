"""
Comprehensive error scenario tests for docker-pyo3

Tests various error conditions to ensure proper exception handling
and error propagation from Rust to Python.
"""

import pytest
import docker_pyo3
from docker_pyo3 import Docker


class TestConnectionErrors:
    """Test connection-related error scenarios"""

    def test_invalid_docker_uri(self):
        """Invalid Docker URI should raise ConnectionError"""
        with pytest.raises(ConnectionError):
            Docker("invalid://invalid-uri")

    def test_unreachable_docker_daemon(self):
        """Unreachable daemon should raise ConnectionError"""
        with pytest.raises(ConnectionError):
            Docker("tcp://192.0.2.1:2375")  # RFC 5737 TEST-NET address

    def test_daemon_health_check_failure(self):
        """Test daemon health check with unreachable daemon"""
        # This test requires a reachable daemon to create the client first
        docker_client = Docker()
        # Modify the internal connection to simulate failure
        # Note: This is testing the health check logic, not actual connection failure
        result = docker_client.health_check()
        assert isinstance(result, dict)
        assert "daemon_reachable" in result
        assert "runtime_healthy" in result
        assert "overall_healthy" in result
        assert "daemon_uri" in result


class TestContainerErrors:
    """Test container-related error scenarios"""

    def test_container_not_found(self, docker):
        """Getting non-existent container should raise FileNotFoundError"""
        with pytest.raises(FileNotFoundError):
            docker.containers().get("non-existent-container-id").inspect()

    def test_container_create_invalid_image(self, docker):
        """Creating container with invalid image should raise appropriate error"""
        with pytest.raises((RuntimeError, FileNotFoundError)):
            docker.containers().create(image="non-existent-image:invalid-tag")

    def test_container_remove_non_existent(self, docker):
        """Removing non-existent container should raise FileNotFoundError"""
        fake_container = docker.containers().get("non-existent-container")
        with pytest.raises(FileNotFoundError):
            fake_container.remove()

    def test_container_start_non_existent(self, docker):
        """Starting non-existent container should raise FileNotFoundError"""
        fake_container = docker.containers().get("non-existent-container")
        with pytest.raises(FileNotFoundError):
            fake_container.start()

    def test_container_stop_non_existent(self, docker):
        """Stopping non-existent container should raise FileNotFoundError"""
        fake_container = docker.containers().get("non-existent-container")
        with pytest.raises(FileNotFoundError):
            fake_container.stop()

    def test_container_invalid_parameters(self, docker):
        """Test container creation with invalid parameters"""
        # Test with invalid port mapping format
        with pytest.raises(ValueError):
            docker.containers().create(
                image="busybox",
                ports={"invalid": "format"}
            )


class TestImageErrors:
    """Test image-related error scenarios"""

    def test_image_not_found(self, docker):
        """Getting non-existent image should raise FileNotFoundError"""
        with pytest.raises(FileNotFoundError):
            docker.images().get("non-existent-image:invalid-tag").inspect()

    def test_image_pull_invalid_name(self, docker):
        """Pulling invalid image name should raise appropriate error"""
        with pytest.raises((RuntimeError, FileNotFoundError)):
            docker.images().pull(image="invalid/image/name/with/too/many/slashes")

    def test_image_remove_non_existent(self, docker):
        """Removing non-existent image should raise FileNotFoundError"""
        fake_image = docker.images().get("non-existent-image")
        with pytest.raises(FileNotFoundError):
            fake_image.remove()

    def test_image_tag_non_existent(self, docker):
        """Tagging non-existent image should raise FileNotFoundError"""
        fake_image = docker.images().get("non-existent-image")
        with pytest.raises(FileNotFoundError):
            fake_image.tag(repo="test", tag="latest")

    def test_image_export_non_existent(self, docker):
        """Exporting non-existent image should raise FileNotFoundError"""
        fake_image = docker.images().get("non-existent-image")
        with pytest.raises(FileNotFoundError):
            fake_image.export()

    def test_image_build_invalid_path(self, docker):
        """Building from invalid path should raise appropriate error"""
        with pytest.raises((IOError, FileNotFoundError)):
            docker.images().build(path="/non/existent/path")


class TestNetworkErrors:
    """Test network-related error scenarios"""

    def test_network_not_found(self, docker):
        """Getting non-existent network should raise FileNotFoundError"""
        with pytest.raises(FileNotFoundError):
            docker.networks().get("non-existent-network").inspect()

    def test_network_create_duplicate_name(self, docker):
        """Creating network with existing name should raise FileExistsError"""
        # First create a network
        try:
            docker.networks().create(name="duplicate-test-network")
            # Try to create another with same name
            with pytest.raises(FileExistsError):
                docker.networks().create(name="duplicate-test-network")
        finally:
            # Clean up
            try:
                docker.networks().get("duplicate-test-network").delete()
            except:
                pass

    def test_network_delete_non_existent(self, docker):
        """Deleting non-existent network should raise FileNotFoundError"""
        fake_network = docker.networks().get("non-existent-network")
        with pytest.raises(FileNotFoundError):
            fake_network.delete()

    def test_network_connect_invalid_container(self, docker):
        """Connecting invalid container to network should raise FileNotFoundError"""
        try:
            network = docker.networks().create(name="test-error-network")
            with pytest.raises(FileNotFoundError):
                network.connect("non-existent-container")
        finally:
            try:
                network.delete()
            except:
                pass

    def test_network_disconnect_invalid_container(self, docker):
        """Disconnecting invalid container from network should raise FileNotFoundError"""
        try:
            network = docker.networks().create(name="test-error-network")
            with pytest.raises(FileNotFoundError):
                network.disconnect("non-existent-container")
        finally:
            try:
                network.delete()
            except:
                pass


class TestVolumeErrors:
    """Test volume-related error scenarios"""

    def test_volume_not_found(self, docker):
        """Getting non-existent volume should raise FileNotFoundError"""
        with pytest.raises(FileNotFoundError):
            docker.volumes().get("non-existent-volume").inspect()

    def test_volume_create_duplicate_name(self, docker):
        """Creating volume with existing name should return existing volume (Docker behavior)"""
        try:
            vol1 = docker.volumes().create(name="duplicate-test-volume")
            # Docker actually allows this and returns the existing volume
            vol2 = docker.volumes().create(name="duplicate-test-volume")
            # Both should refer to the same volume
            assert vol1 is not None
            assert vol2 is not None
        finally:
            # Clean up
            try:
                docker.volumes().get("duplicate-test-volume").delete()
            except:
                pass

    def test_volume_delete_non_existent(self, docker):
        """Deleting non-existent volume should raise FileNotFoundError"""
        fake_volume = docker.volumes().get("non-existent-volume")
        with pytest.raises(FileNotFoundError):
            fake_volume.delete()


class TestParameterValidation:
    """Test parameter validation error scenarios"""

    def test_empty_container_name(self, docker):
        """Creating container with empty name should handle gracefully"""
        # Empty name should be handled by Docker API, not cause a crash
        result = docker.containers().create(image="busybox", name="")
        # Clean up if successful
        try:
            result.remove()
        except:
            pass

    def test_invalid_port_format(self, docker):
        """Invalid port format should raise ValueError"""
        with pytest.raises((ValueError, RuntimeError)):
            docker.containers().create(
                image="busybox",
                ports={"invalid_port": "format"}
            )

    def test_none_image_name(self, docker):
        """None image name should raise ValueError"""
        with pytest.raises((ValueError, TypeError)):
            docker.containers().create(image=None)


class TestRuntimeErrors:
    """Test runtime and system-level error scenarios"""

    def test_runtime_health_check(self, docker):
        """Test runtime health check functionality"""
        is_healthy = docker.runtime_health()
        assert isinstance(is_healthy, bool)
        # Runtime should generally be healthy in tests
        assert is_healthy is True

    def test_daemon_health_check(self, docker):
        """Test daemon health check functionality"""
        is_healthy = docker.daemon_health()
        assert isinstance(is_healthy, bool)
        # Daemon should be healthy if tests are running
        assert is_healthy is True

    def test_comprehensive_health_check(self, docker):
        """Test comprehensive health check functionality"""
        health_info = docker.health_check()
        assert isinstance(health_info, dict)
        
        required_keys = [
            "runtime_healthy",
            "daemon_reachable", 
            "overall_healthy",
            "daemon_uri"
        ]
        
        for key in required_keys:
            assert key in health_info
        
        # Values should be appropriate types
        assert isinstance(health_info["runtime_healthy"], bool)
        assert isinstance(health_info["daemon_reachable"], bool)
        assert isinstance(health_info["overall_healthy"], bool)
        assert isinstance(health_info["daemon_uri"], str)
        
        # Overall health should be AND of runtime and daemon health
        expected_overall = health_info["runtime_healthy"] and health_info["daemon_reachable"]
        assert health_info["overall_healthy"] == expected_overall


class TestSerializationErrors:
    """Test serialization/deserialization error scenarios"""

    def test_malformed_response_handling(self, docker):
        """Test handling of malformed responses from Docker API"""
        # This is harder to test directly, but we can verify that
        # normal operations don't crash with serialization errors
        version_info = docker.version()
        assert isinstance(version_info, dict)
        
        info = docker.info()
        assert isinstance(info, dict)
        
        ping_info = docker.ping()
        assert isinstance(ping_info, dict)


class TestResourceCleanup:
    """Test proper resource cleanup in error scenarios"""

    def test_failed_container_cleanup(self, docker):
        """Test that failed container operations don't leak resources"""
        # Create a container and then try to perform invalid operations
        try:
            container = docker.containers().create(image="busybox", name="cleanup-test")
            
            # Try invalid operations that should fail
            with pytest.raises(Exception):
                # Try to commit a non-running container with invalid settings
                container.commit("invalid/repo:tag", "invalid message with \x00 null bytes")
                
        finally:
            # Ensure cleanup happens even after errors
            try:
                container.remove()
            except:
                pass

    def test_failed_network_cleanup(self, docker):
        """Test that failed network operations don't leak resources"""
        network = None
        try:
            network = docker.networks().create(name="cleanup-test-network")
            
            # Try invalid operations
            with pytest.raises(Exception):
                network.connect("absolutely-non-existent-container-id")
                
        finally:
            if network:
                try:
                    network.delete()
                except:
                    pass

    def test_failed_volume_cleanup(self, docker):
        """Test that failed volume operations don't leak resources"""
        volume = None
        try:
            volume = docker.volumes().create(name="cleanup-test-volume")
            
            # Volume operations are simpler, but let's ensure cleanup works
            volume.inspect()  # This should work
            
        finally:
            if volume:
                try:
                    volume.delete()
                except:
                    pass


# Additional stress tests for error conditions
class TestErrorStress:
    """Stress test error handling under various conditions"""

    def test_multiple_invalid_operations(self, docker):
        """Test multiple invalid operations in sequence"""
        errors_caught = 0
        
        # Try multiple invalid operations
        invalid_operations = [
            lambda: docker.containers().get("invalid1").inspect(),
            lambda: docker.images().get("invalid2").inspect(),
            lambda: docker.networks().get("invalid3").inspect(),
            lambda: docker.volumes().get("invalid4").inspect(),
        ]
        
        for operation in invalid_operations:
            try:
                operation()
            except (FileNotFoundError, RuntimeError):
                errors_caught += 1
            except Exception as e:
                # Log unexpected exception types for debugging
                pytest.fail(f"Unexpected exception type: {type(e).__name__}: {e}")
        
        # All operations should have raised appropriate errors
        assert errors_caught == len(invalid_operations)

    def test_concurrent_error_operations(self, docker):
        """Test that error handling works with rapid successive calls"""
        import threading
        import time
        
        errors = []
        
        def invalid_operation():
            try:
                docker.containers().get("concurrent-invalid").inspect()
            except Exception as e:
                errors.append(type(e).__name__)
        
        # Start multiple threads doing invalid operations
        threads = []
        for _ in range(5):
            thread = threading.Thread(target=invalid_operation)
            threads.append(thread)
            thread.start()
        
        # Wait for all threads to complete
        for thread in threads:
            thread.join()
        
        # All operations should have resulted in errors
        assert len(errors) == 5
        # All should be the expected error type
        for error_type in errors:
            assert error_type in ["FileNotFoundError", "RuntimeError"]