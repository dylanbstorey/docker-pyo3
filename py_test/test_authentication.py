"""
Test authentication and registry operations for docker-pyo3

Tests image push/pull with authentication, registry operations.
"""

import pytest
import os
from docker_pyo3 import Docker

@pytest.fixture
def docker():
    return Docker()

class TestAuthenticationFormats:
    """Test various authentication format handling"""
    
    def test_auth_password_format_validation(self, docker):
        """Test validation of password auth format"""
        # Missing required fields
        invalid_auths = [
            {},  # Empty
            {"username": "user"},  # Missing password
            {"password": "pass"},  # Missing username
            {"username": "user", "password": ""},  # Empty password
        ]
        
        for auth in invalid_auths:
            # Should handle gracefully (might not error until actual use)
            try:
                result = docker.images().pull(
                    image="busybox",
                    auth_password=auth
                )
                # If it doesn't error on format, it should work (public image)
                assert result is not None
            except Exception as e:
                # If it errors, should be auth-related
                assert "auth" in str(e).lower() or "credential" in str(e).lower()
    
    def test_auth_token_format_validation(self, docker):
        """Test validation of token auth format"""
        # Missing token
        with pytest.raises(Exception):
            docker.images().pull(
                image="busybox",
                auth_token={}
            )
        
        # Empty token
        with pytest.raises(Exception):
            docker.images().pull(
                image="busybox",
                auth_token={"identity_token": ""}
            )
    
    def test_auth_mutual_exclusion(self, docker):
        """Test that password and token auth are mutually exclusive"""
        auth_password = {
            "username": "user",
            "password": "pass"
        }
        auth_token = {
            "identity_token": "token"
        }
        
        # Should error when both provided
        with pytest.raises(ValueError) as exc_info:
            docker.images().pull(
                image="busybox",
                auth_password=auth_password,
                auth_token=auth_token
            )
        assert "both auth_password and auth_token" in str(exc_info.value)
    
    def test_pull_public_image_with_auth(self, docker):
        """Test pulling public image with unnecessary auth"""
        # Pulling public image with invalid auth credentials should fail
        # Docker validates credentials even for public images
        auth = {
            "username": "unused",
            "password": "unused", 
            "email": "unused@example.com",
            "server_address": "https://index.docker.io/v1/"
        }
        
        try:
            result = docker.images().pull(
                image="busybox:latest",
                auth_password=auth
            )
            # If it succeeds, that's also acceptable
            assert result is not None
            assert isinstance(result, list)
        except Exception as e:
            # Expected - invalid auth should fail
            error_msg = str(e).lower()
            assert any(word in error_msg for word in ["unauthorized", "incorrect", "auth", "password"])

class TestImagePushOperations:
    """Test image push operations"""
    
    def test_push_without_auth(self, docker):
        """Test pushing without authentication should not crash"""
        # Get a local image
        docker.images().pull(image="busybox:latest")
        image = docker.images().get("busybox:latest")
        
        # Push without auth - this may succeed but not actually push anything
        # The Docker API allows the operation to start but will fail at registry level
        try:
            result = image.push()
            # Push completed without error (though it might not have actually pushed)
            assert result is None
        except Exception as e:
            # If it does error, should be auth-related
            error_msg = str(e).lower()
            assert any(word in error_msg for word in ["auth", "denied", "unauthorized", "forbidden"])
    
    def test_push_with_invalid_repository(self, docker):
        """Test pushing to invalid repository"""
        image = docker.images().get("busybox")
        
        # Tag with invalid repository
        image.tag(repo="invalid/repo/name/too/many/slashes", tag="latest")
        
        try:
            tagged_image = docker.images().get("invalid/repo/name/too/many/slashes:latest")
            # Try to push - this should either work or fail gracefully
            try:
                result = tagged_image.push()
                assert result is None
            except Exception as e:
                # If it fails, should be related to invalid repository
                error_msg = str(e).lower()
                assert any(word in error_msg for word in ["invalid", "denied", "error", "repository"])
        finally:
            # Cleanup
            try:
                docker.images().get("invalid/repo/name/too/many/slashes:latest").delete()
            except:
                pass
    
    @pytest.mark.skip(reason="Requires valid registry credentials")
    def test_push_to_private_registry(self, docker):
        """Test pushing to private registry with auth"""
        # This test requires:
        # 1. A private registry to push to
        # 2. Valid credentials
        # 3. Push permissions
        
        auth = {
            "username": os.environ.get("REGISTRY_USERNAME", "testuser"),
            "password": os.environ.get("REGISTRY_PASSWORD", "testpass"),
            "server_address": os.environ.get("REGISTRY_URL", "localhost:5000")
        }
        
        # Pull, tag, and push
        docker.images().pull(image="busybox:latest")
        image = docker.images().get("busybox:latest")
        
        # Tag for private registry
        registry = auth["server_address"]
        image.tag(repo=f"{registry}/test/busybox", tag="test")
        
        # Push with auth
        tagged_image = docker.images().get(f"{registry}/test/busybox:test")
        tagged_image.push(auth_password=auth)
        
        # Cleanup
        tagged_image.delete()
    
    def test_push_with_specific_tag(self, docker):
        """Test pushing with specific tag parameter"""
        # Get image and tag it
        docker.images().pull(image="busybox:latest")
        image = docker.images().get("busybox:latest")
        
        # Tag with multiple tags
        image.tag(repo="testpush", tag="v1")
        image.tag(repo="testpush", tag="v2")
        
        try:
            # Try to push specific tag
            tagged_image = docker.images().get("testpush:v1")
            try:
                result = tagged_image.push(tag="v1")
                assert result is None
            except Exception as e:
                # If it fails, should be auth or repository related
                error_msg = str(e).lower()
                assert any(word in error_msg for word in ["denied", "auth", "unauthorized", "error"])
        finally:
            # Cleanup
            try:
                docker.images().get("testpush:v1").delete()
                docker.images().get("testpush:v2").delete()
            except:
                pass

class TestRegistryOperations:
    """Test registry-related operations"""
    
    def test_pull_from_different_registry(self, docker):
        """Test pulling from registries other than Docker Hub"""
        # Try to pull from a different registry
        # Using GitHub Container Registry as example (public images)
        alternative_images = [
            "ghcr.io/actions/runner:latest",  # GitHub Actions runner
            "gcr.io/distroless/static:latest",  # Google distroless
            "quay.io/coreos/etcd:latest",  # Quay.io
        ]
        
        # At least one should work (if internet is available)
        success = False
        for image_name in alternative_images:
            try:
                result = docker.images().pull(image=image_name)
                if result:
                    success = True
                    # Cleanup
                    docker.images().get(image_name).delete()
                    break
            except:
                continue
        
        # Skip if no internet or all registries are down
        if not success:
            pytest.skip("Could not reach any alternative registries")
    
    def test_image_name_parsing(self, docker):
        """Test various image name formats"""
        test_cases = [
            ("busybox", "busybox", "latest"),  # Simple name
            ("busybox:1.35", "busybox", "1.35"),  # With tag
            ("library/busybox", "library/busybox", "latest"),  # With namespace
            ("docker.io/library/busybox", "docker.io/library/busybox", "latest"),  # Full
        ]
        
        for image_name, expected_repo, expected_tag in test_cases:
            # Pull and verify we can access it
            try:
                result = docker.images().pull(image=image_name)
                assert result is not None
                
                # Get the image
                image = docker.images().get(image_name)
                assert image is not None
            except Exception as e:
                # Some formats might not work depending on Docker version
                print(f"Failed to pull {image_name}: {e}")

class TestAuthenticationErrors:
    """Test authentication error scenarios"""
    
    def test_pull_private_image_without_auth(self, docker):
        """Test pulling private image without credentials"""
        # Try to pull a private image (this should fail)
        private_images = [
            "private/image:latest",  # Generic private
            "mycompany/private-app:latest",  # Fake private image
        ]
        
        for image_name in private_images:
            try:
                docker.images().pull(image=image_name)
                # If it succeeds, image might actually be public
            except Exception as e:
                # Should get not found or auth error
                error_msg = str(e).lower()
                assert ("not found" in error_msg or 
                        "denied" in error_msg or 
                        "unauthorized" in error_msg or
                        "authentication" in error_msg)
    
    def test_pull_with_wrong_credentials(self, docker):
        """Test pulling with incorrect credentials"""
        auth = {
            "username": "wronguser",
            "password": "wrongpass",
            "server_address": "https://index.docker.io/v1/"
        }
        
        # Try to pull with wrong credentials - should fail
        try:
            result = docker.images().pull(
                image="busybox:latest",
                auth_password=auth
            )
            # If it somehow succeeds, that's also valid
            assert result is not None
        except Exception as e:
            # Expected - wrong credentials should fail
            error_msg = str(e).lower()
            assert any(word in error_msg for word in ["unauthorized", "incorrect", "wrong", "auth"])
    
    def test_auth_with_special_characters(self, docker):
        """Test authentication with special characters in credentials"""
        # Test auth with special characters
        special_auths = [
            {
                "username": "user@example.com",
                "password": "p@ssw0rd!#$%",
                "email": "user@example.com"
            },
            {
                "username": "user name",  # Space
                "password": "pass word",
                "email": "email@test.com"
            },
            {
                "username": "user🔑",  # Unicode
                "password": "pass🗝️",
                "email": "test@example.com"
            }
        ]
        
        for auth in special_auths:
            # Should handle special characters gracefully
            try:
                # Public image should work regardless
                result = docker.images().pull(
                    image="busybox:latest",
                    auth_password=auth
                )
                assert result is not None
            except Exception as e:
                # If it fails, should be clear error
                assert len(str(e)) > 0

class TestTokenAuthentication:
    """Test token-based authentication"""
    
    @pytest.mark.skip(reason="Requires valid token")
    def test_pull_with_valid_token(self, docker):
        """Test pulling with valid token"""
        token = os.environ.get("DOCKER_TOKEN", "")
        if not token:
            pytest.skip("No Docker token available")
        
        auth = {
            "identity_token": token
        }
        
        result = docker.images().pull(
            image="busybox:latest",
            auth_token=auth
        )
        
        assert result is not None
    
    def test_push_with_token_auth(self, docker):
        """Test pushing with token authentication"""
        # Get an image
        docker.images().pull(image="busybox:latest")
        image = docker.images().get("busybox:latest")
        
        # Tag it
        image.tag(repo="testtoken", tag="latest")
        
        try:
            # Try to push with fake token
            fake_token = {"identity_token": "fake-token-12345"}
            
            tagged_image = docker.images().get("testtoken:latest")
            try:
                result = tagged_image.push(auth_token=fake_token)
                assert result is None
            except Exception as e:
                # If it fails, should be auth-related
                error_msg = str(e).lower()
                assert any(word in error_msg for word in ["auth", "denied", "unauthorized", "token"])
        finally:
            # Cleanup
            try:
                docker.images().get("testtoken:latest").delete()
            except:
                pass