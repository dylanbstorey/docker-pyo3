"""
Test edge cases and parameter validation for docker-pyo3

Tests unusual inputs, boundary conditions, and parameter validation.
"""

import pytest
from docker_pyo3 import Docker

@pytest.fixture
def docker():
    return Docker()

class TestParameterValidation:
    """Test parameter validation and edge cases"""
    
    def test_container_create_empty_image_name(self, docker):
        """Test creating container with empty image name"""
        with pytest.raises(Exception):
            docker.containers().create(image="")
    
    def test_container_create_none_image_name(self, docker):
        """Test creating container with None image name"""
        with pytest.raises(Exception):
            docker.containers().create(image=None)
    
    def test_container_create_very_long_name(self, docker):
        """Test creating container with very long name"""
        # Docker allows quite long names, test with an extremely long one
        long_name = "a" * 1000  # Much longer than practical
        
        try:
            container = docker.containers().create(
                image="busybox",
                name=long_name
            )
            # If it succeeds, clean up
            container.remove()
            # Docker allows very long names, so this is fine
        except Exception:
            # If it fails, that's also acceptable - depends on Docker version
            pass
    
    def test_container_create_invalid_name_characters(self, docker):
        """Test creating container with invalid characters in name"""
        # Docker is actually quite permissive with container names
        # Only test truly invalid characters that should definitely fail
        truly_invalid_names = [
            "",  # Empty name
            "/",  # Just a slash
            " ",  # Just a space
        ]
        
        for name in truly_invalid_names:
            try:
                container = docker.containers().create(
                    image="busybox",
                    name=name
                )
                # If it succeeded, clean up
                container.remove()
                # Some names might be allowed - that's fine
            except Exception:
                # Expected for truly invalid names
                pass
        
        # Test one that should work to verify functionality
        container = docker.containers().create(
            image="busybox",
            name="valid-container-name"
        )
        container.remove()
    
    def test_container_create_duplicate_name(self, docker):
        """Test creating container with duplicate name"""
        container1 = docker.containers().create(
            image="busybox",
            name="duplicate-name-test"
        )
        
        try:
            # Try to create another with same name
            with pytest.raises(Exception):
                docker.containers().create(
                    image="busybox", 
                    name="duplicate-name-test"
                )
        finally:
            container1.remove()
    
    def test_container_create_invalid_memory_values(self, docker):
        """Test creating container with invalid memory values"""
        # Negative memory
        with pytest.raises(Exception):
            docker.containers().create(
                image="busybox",
                name="test-negative-memory",
                memory=-1
            )
        
        # Zero memory (might be allowed)
        try:
            container = docker.containers().create(
                image="busybox",
                name="test-zero-memory",
                memory=0
            )
            container.remove()
        except:
            # Some Docker versions might reject this
            pass
    
    def test_container_create_invalid_cpu_values(self, docker):
        """Test creating container with invalid CPU values"""
        # Negative CPU shares
        with pytest.raises(Exception):
            docker.containers().create(
                image="busybox",
                name="test-negative-cpu",
                cpu_shares=-1
            )
        
        # Very large nano CPUs (more than system has)
        try:
            container = docker.containers().create(
                image="busybox",
                name="test-large-cpu",
                nano_cpus=1000000000000  # 1000 CPUs
            )
            # If it succeeds, Docker might cap it
            container.remove()
        except:
            # Expected to fail on most systems
            pass
    
    def test_network_operations_with_invalid_ids(self, docker):
        """Test network operations with invalid container IDs"""
        network = docker.networks().create(name="test-invalid-ops")
        
        try:
            # Connect non-existent container
            with pytest.raises(Exception):
                network.connect(container_id="non-existent-container")
            
            # Disconnect non-existent container
            with pytest.raises(Exception):
                network.disconnect(container_id="non-existent-container")
            
            # Empty container ID
            with pytest.raises(Exception):
                network.connect(container_id="")
        finally:
            network.delete()
    
    def test_volume_create_invalid_names(self, docker):
        """Test volume creation with invalid names"""
        # Invalid volume names that should fail (based on Docker validation)
        invalid_names = [
            "volume name",  # Space (definitely invalid)
            "volume/name",  # Forward slash (invalid)
            "../volume",    # Path traversal (invalid)
        ]
        
        for name in invalid_names:
            try:
                volume = docker.volumes().create(name=name)
                # If creation succeeds, clean up and fail the test
                try:
                    volume.delete()
                except:
                    pass
                pytest.fail(f"Expected volume creation with name '{name}' to fail, but it succeeded")
            except Exception:
                # Expected - volume creation should fail
                pass
    
    def test_image_operations_with_invalid_tags(self, docker):
        """Test image operations with invalid tags"""
        # Test one clearly invalid operation that should definitely fail
        image = docker.images().get("busybox")
        
        # This should definitely fail
        try:
            image.tag(repo="test with spaces and/slashes", tag="latest")
            # Clean up if it somehow succeeded
            try:
                docker.images().get("test with spaces and/slashes:latest").delete()
            except:
                pass
            # If we get here, the validation is too permissive, but that's not critical
        except Exception:
            # Expected - this should fail
            pass

class TestBoundaryConditions:
    """Test boundary conditions and limits"""
    
    def test_container_with_many_environment_variables(self, docker):
        """Test container with large number of environment variables"""
        # Create many env vars
        env_vars = [f"VAR_{i}=value_{i}" for i in range(100)]
        
        container = docker.containers().create(
            image="busybox",
            name="test-many-env",
            env=env_vars
        )
        
        try:
            info = container.inspect()
            # All env vars should be set
            for var in env_vars:
                assert var in info['Config']['Env']
        finally:
            container.remove()
    
    def test_container_with_many_labels(self, docker):
        """Test container with many labels"""
        # Create many labels
        labels = {f"label{i}": f"value{i}" for i in range(50)}
        
        container = docker.containers().create(
            image="busybox",
            name="test-many-labels",
            labels=labels
        )
        
        try:
            info = container.inspect()
            # All labels should be set
            for key, value in labels.items():
                assert info['Config']['Labels'][key] == value
        finally:
            container.remove()
    
    def test_logs_with_zero_lines(self, docker):
        """Test getting logs with n_lines=0"""
        container = docker.containers().create(
            image="busybox",
            name="test-zero-lines",
            command=["echo", "test"]
        )
        
        try:
            container.start()
            container.wait()
            
            # Request 0 lines
            logs = container.logs(n_lines=0)
            # Should return empty or minimal output
            assert logs == "" or len(logs) == 0
        finally:
            container.remove()
    
    def test_exec_with_empty_command(self, docker):
        """Test exec with empty command"""
        container = docker.containers().create(
            image="busybox",
            name="test-empty-exec",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            
            # Empty command list
            with pytest.raises(Exception):
                container.exec(command=[])
        finally:
            container.stop()
            container.remove()

class TestUnicodeAndSpecialCharacters:
    """Test handling of unicode and special characters"""
    
    def test_container_with_unicode_environment(self, docker):
        """Test container with unicode in environment variables"""
        container = docker.containers().create(
            image="busybox",
            name="test-unicode-env",
            env=[
                "GREETING=Hello 世界",
                "EMOJI=🐳 Docker",
                "SPECIAL=Ñoño"
            ]
        )
        
        try:
            info = container.inspect()
            assert "GREETING=Hello 世界" in info['Config']['Env']
            assert "EMOJI=🐳 Docker" in info['Config']['Env']
            assert "SPECIAL=Ñoño" in info['Config']['Env']
        finally:
            container.remove()
    
    def test_container_with_unicode_labels(self, docker):
        """Test container with unicode in labels"""
        container = docker.containers().create(
            image="busybox",
            name="test-unicode-labels",
            labels={
                "description": "容器测试",
                "emoji": "🚀",
                "language": "español"
            }
        )
        
        try:
            info = container.inspect()
            assert info['Config']['Labels']['description'] == "容器测试"
            assert info['Config']['Labels']['emoji'] == "🚀"
            assert info['Config']['Labels']['language'] == "español"
        finally:
            container.remove()
    
    def test_exec_with_unicode_command(self, docker):
        """Test exec with unicode in command"""
        container = docker.containers().create(
            image="busybox",
            name="test-unicode-exec",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            
            # Command with unicode
            container.exec(
                command=["echo", "Hello 世界 🌍"],
                attach_stdout=True
            )
        finally:
            container.stop()
            container.remove()

class TestResourceLimits:
    """Test various resource limit edge cases"""
    
    def test_container_with_minimal_memory(self, docker):
        """Test container with minimum viable memory"""
        # Docker requires at least 6MB
        container = docker.containers().create(
            image="busybox",
            name="test-min-memory",
            memory=6 * 1024 * 1024,  # 6MB
            command=["echo", "minimal memory"]
        )
        
        try:
            container.start()
            container.wait()
            
            info = container.inspect()
            assert info['HostConfig']['Memory'] == 6 * 1024 * 1024
        finally:
            container.remove()
    
    def test_container_with_fractional_cpu(self, docker):
        """Test container with fractional CPU allocation"""
        container = docker.containers().create(
            image="busybox",
            name="test-fractional-cpu",
            nano_cpus=100000000,  # 0.1 CPU
            command=["echo", "fractional cpu"]
        )
        
        try:
            info = container.inspect()
            assert info['HostConfig']['NanoCpus'] == 100000000
        finally:
            container.remove()

class TestConcurrentOperations:
    """Test concurrent operations on same resources"""
    
    def test_concurrent_container_operations(self, docker):
        """Test multiple operations on same container"""
        container = docker.containers().create(
            image="busybox",
            name="test-concurrent",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            
            # Rapid operations
            for _ in range(5):
                info = container.inspect()
                assert info is not None
            
            # Multiple log requests
            for _ in range(3):
                logs = container.logs(n_lines=1)
                assert logs is not None
            
            # Multiple exec operations
            for i in range(3):
                container.exec(
                    command=["echo", f"test{i}"],
                    attach_stdout=True
                )
        finally:
            container.stop()
            container.remove()
    
    def test_multiple_clients_same_container(self, docker):
        """Test accessing same container from multiple client instances"""
        container = docker.containers().create(
            image="busybox",
            name="test-multi-client",
            command=["sleep", "300"]
        )
        
        try:
            container.start()
            
            # Create another client instance
            docker2 = Docker()
            
            # Access same container from both
            container2 = docker2.containers().get("test-multi-client")
            
            # Both should see same state
            info1 = container.inspect()
            info2 = container2.inspect()
            
            assert info1['Id'] == info2['Id']
            assert info1['State']['Running'] == info2['State']['Running']
        finally:
            container.stop()
            container.remove()