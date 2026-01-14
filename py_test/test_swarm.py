"""Tests for Docker Swarm mode operations.

Note: Swarm mode operations require Docker to be running in swarm mode.
Tests are designed to handle both swarm-enabled and non-swarm environments
gracefully, skipping tests that require swarm when it's not available.
"""
import pytest
from docker_pyo3 import Docker
from docker_pyo3.node import Nodes, Node
from docker_pyo3.service import Services, Service
from docker_pyo3.secret import Secrets, Secret
from docker_pyo3.config import Configs, Config
from docker_pyo3.task import Tasks, Task


@pytest.fixture
def docker():
    return Docker()


@pytest.fixture
def is_swarm_active(docker):
    """Check if swarm mode is active."""
    try:
        info = docker.info()
        swarm_info = info.get("Swarm", {})
        return swarm_info.get("LocalNodeState") == "active"
    except Exception:
        return False


@pytest.fixture
def require_swarm(is_swarm_active):
    """Skip test if swarm is not active."""
    if not is_swarm_active:
        pytest.skip("Swarm mode not active - skipping swarm test")


# =============================================================================
# Node Tests
# =============================================================================

class TestNodes:
    """Tests for Docker Swarm nodes."""

    def test_nodes_init(self, docker):
        """nodes collection accessor returns Nodes"""
        x = docker.nodes()
        assert isinstance(x, Nodes)

    def test_nodes_from_constructor(self, docker):
        """we can create Nodes from constructor"""
        nodes = Nodes(docker)
        assert isinstance(nodes, Nodes)

    def test_nodes_get(self, docker):
        """we can get a node interface by id"""
        nodes = docker.nodes()
        node = nodes.get("test_node_id")
        assert isinstance(node, Node)

    def test_node_from_constructor(self, docker):
        """we can create Node from constructor"""
        node = Node(docker, "test_node_id")
        assert isinstance(node, Node)

    def test_node_id(self, docker):
        """node has an id"""
        node = Node(docker, "test_node_id")
        assert node.id() == "test_node_id"

    def test_nodes_list_without_swarm(self, docker, is_swarm_active):
        """listing nodes without swarm raises error"""
        if is_swarm_active:
            pytest.skip("Swarm is active - cannot test non-swarm behavior")
        nodes = docker.nodes()
        with pytest.raises(SystemError):
            nodes.list()

    def test_nodes_list_with_swarm(self, docker, require_swarm):
        """we can list nodes in swarm mode"""
        nodes = docker.nodes()
        result = nodes.list()
        assert isinstance(result, list)
        # At least one node (the manager) should exist
        assert len(result) > 0
        for node in result:
            assert isinstance(node, dict)
            assert "ID" in node

    def test_node_inspect_nonexistent(self, docker, require_swarm):
        """inspecting non-existent node raises error"""
        node = docker.nodes().get("nonexistent_node_id_12345")
        with pytest.raises(SystemError):
            node.inspect()

    def test_node_inspect_existing(self, docker, require_swarm):
        """we can inspect an existing node"""
        nodes_list = docker.nodes().list()
        if not nodes_list:
            pytest.skip("No nodes available")

        node_id = nodes_list[0].get("ID")
        node = docker.nodes().get(node_id)
        info = node.inspect()

        assert isinstance(info, dict)
        assert "ID" in info
        assert info["ID"] == node_id

    def test_node_delete_nonexistent(self, docker, require_swarm):
        """deleting non-existent node raises error"""
        node = docker.nodes().get("nonexistent_node_id_12345")
        with pytest.raises(SystemError):
            node.delete()


# =============================================================================
# Service Tests
# =============================================================================

class TestServices:
    """Tests for Docker Swarm services."""

    def test_services_init(self, docker):
        """services collection accessor returns Services"""
        x = docker.services()
        assert isinstance(x, Services)

    def test_services_from_constructor(self, docker):
        """we can create Services from constructor"""
        services = Services(docker)
        assert isinstance(services, Services)

    def test_services_get(self, docker):
        """we can get a service interface by id"""
        services = docker.services()
        service = services.get("test_service_id")
        assert isinstance(service, Service)

    def test_service_from_constructor(self, docker):
        """we can create Service from constructor"""
        service = Service(docker, "test_service_id")
        assert isinstance(service, Service)

    def test_service_id(self, docker):
        """service has an id"""
        service = Service(docker, "test_service_id")
        assert service.id() == "test_service_id"

    def test_services_list_without_swarm(self, docker, is_swarm_active):
        """listing services without swarm raises error"""
        if is_swarm_active:
            pytest.skip("Swarm is active - cannot test non-swarm behavior")
        services = docker.services()
        with pytest.raises(SystemError):
            services.list()

    def test_services_list_with_swarm(self, docker, require_swarm):
        """we can list services in swarm mode"""
        services = docker.services()
        result = services.list()
        assert isinstance(result, list)
        # List might be empty if no services exist
        for service in result:
            assert isinstance(service, dict)

    def test_service_inspect_nonexistent(self, docker, require_swarm):
        """inspecting non-existent service raises error"""
        service = docker.services().get("nonexistent_service_id_12345")
        with pytest.raises(SystemError):
            service.inspect()

    def test_service_delete_nonexistent(self, docker, require_swarm):
        """deleting non-existent service raises error"""
        service = docker.services().get("nonexistent_service_id_12345")
        with pytest.raises(SystemError):
            service.delete()

    def test_service_logs_nonexistent(self, docker, require_swarm):
        """getting logs from non-existent service returns empty"""
        service = docker.services().get("nonexistent_service_id_12345")
        # Logs on nonexistent service typically returns empty, not error
        result = service.logs(stdout=True, stderr=True)
        assert isinstance(result, str)


# =============================================================================
# Secret Tests
# =============================================================================

class TestSecrets:
    """Tests for Docker Swarm secrets."""

    def test_secrets_init(self, docker):
        """secrets collection accessor returns Secrets"""
        x = docker.secrets()
        assert isinstance(x, Secrets)

    def test_secrets_from_constructor(self, docker):
        """we can create Secrets from constructor"""
        secrets = Secrets(docker)
        assert isinstance(secrets, Secrets)

    def test_secrets_get(self, docker):
        """we can get a secret interface by id"""
        secrets = docker.secrets()
        secret = secrets.get("test_secret_id")
        assert isinstance(secret, Secret)

    def test_secret_from_constructor(self, docker):
        """we can create Secret from constructor"""
        secret = Secret(docker, "test_secret_id")
        assert isinstance(secret, Secret)

    def test_secret_id(self, docker):
        """secret has an id"""
        secret = Secret(docker, "test_secret_id")
        assert secret.id() == "test_secret_id"

    def test_secrets_list_without_swarm(self, docker, is_swarm_active):
        """listing secrets without swarm raises error"""
        if is_swarm_active:
            pytest.skip("Swarm is active - cannot test non-swarm behavior")
        secrets = docker.secrets()
        with pytest.raises(SystemError):
            secrets.list()

    def test_secrets_list_with_swarm(self, docker, require_swarm):
        """we can list secrets in swarm mode"""
        secrets = docker.secrets()
        result = secrets.list()
        assert isinstance(result, list)

    def test_secret_inspect_nonexistent(self, docker, require_swarm):
        """inspecting non-existent secret raises error"""
        secret = docker.secrets().get("nonexistent_secret_id_12345")
        with pytest.raises(SystemError):
            secret.inspect()

    def test_secret_delete_nonexistent(self, docker, require_swarm):
        """deleting non-existent secret raises error"""
        secret = docker.secrets().get("nonexistent_secret_id_12345")
        with pytest.raises(SystemError):
            secret.delete()

    def test_secret_create_inspect_delete(self, docker, require_swarm):
        """we can create, inspect, and delete a secret"""
        import time
        secrets = docker.secrets()

        # Use unique name to avoid conflicts
        secret_name = f"test_secret_{int(time.time())}"
        secret_data = "super_secret_password_123"

        try:
            # Create
            created_secret = secrets.create(
                name=secret_name,
                data=secret_data,
                labels={"test": "true", "app": "docker-pyo3"}
            )
            assert isinstance(created_secret, Secret)

            # Inspect
            info = created_secret.inspect()
            assert isinstance(info, dict)
            assert "ID" in info
            assert info["Spec"]["Name"] == secret_name

            # List should include our secret
            secret_list = secrets.list()
            secret_names = [s.get("Spec", {}).get("Name") for s in secret_list]
            assert secret_name in secret_names

        finally:
            # Delete
            try:
                created_secret.delete()
            except Exception:
                pass


# =============================================================================
# Config Tests
# =============================================================================

class TestConfigs:
    """Tests for Docker Swarm configs."""

    def test_configs_init(self, docker):
        """configs collection accessor returns Configs"""
        x = docker.configs()
        assert isinstance(x, Configs)

    def test_configs_from_constructor(self, docker):
        """we can create Configs from constructor"""
        configs = Configs(docker)
        assert isinstance(configs, Configs)

    def test_configs_get(self, docker):
        """we can get a config interface by id"""
        configs = docker.configs()
        config = configs.get("test_config_id")
        assert isinstance(config, Config)

    def test_config_from_constructor(self, docker):
        """we can create Config from constructor"""
        config = Config(docker, "test_config_id")
        assert isinstance(config, Config)

    def test_config_id(self, docker):
        """config has an id"""
        config = Config(docker, "test_config_id")
        assert config.id() == "test_config_id"

    def test_configs_list_without_swarm(self, docker, is_swarm_active):
        """listing configs without swarm raises error"""
        if is_swarm_active:
            pytest.skip("Swarm is active - cannot test non-swarm behavior")
        configs = docker.configs()
        with pytest.raises(SystemError):
            configs.list()

    def test_configs_list_with_swarm(self, docker, require_swarm):
        """we can list configs in swarm mode"""
        configs = docker.configs()
        result = configs.list()
        assert isinstance(result, list)

    def test_config_inspect_nonexistent(self, docker, require_swarm):
        """inspecting non-existent config raises error"""
        config = docker.configs().get("nonexistent_config_id_12345")
        with pytest.raises(SystemError):
            config.inspect()

    def test_config_delete_nonexistent(self, docker, require_swarm):
        """deleting non-existent config raises error"""
        config = docker.configs().get("nonexistent_config_id_12345")
        with pytest.raises(SystemError):
            config.delete()

    def test_config_create_inspect_delete(self, docker, require_swarm):
        """we can create, inspect, and delete a config"""
        import time
        configs = docker.configs()

        # Use unique name to avoid conflicts
        config_name = f"test_config_{int(time.time())}"
        config_data = "server.host=localhost\nserver.port=8080"

        try:
            # Create
            created_config = configs.create(
                name=config_name,
                data=config_data,
                labels={"test": "true", "app": "docker-pyo3"}
            )
            assert isinstance(created_config, Config)

            # Inspect
            info = created_config.inspect()
            assert isinstance(info, dict)
            assert "ID" in info
            assert info["Spec"]["Name"] == config_name

            # List should include our config
            config_list = configs.list()
            config_names = [c.get("Spec", {}).get("Name") for c in config_list]
            assert config_name in config_names

        finally:
            # Delete
            try:
                created_config.delete()
            except Exception:
                pass


# =============================================================================
# Task Tests
# =============================================================================

class TestTasks:
    """Tests for Docker Swarm tasks."""

    def test_tasks_init(self, docker):
        """tasks collection accessor returns Tasks"""
        x = docker.tasks()
        assert isinstance(x, Tasks)

    def test_tasks_from_constructor(self, docker):
        """we can create Tasks from constructor"""
        tasks = Tasks(docker)
        assert isinstance(tasks, Tasks)

    def test_tasks_get(self, docker):
        """we can get a task interface by id"""
        tasks = docker.tasks()
        task = tasks.get("test_task_id")
        assert isinstance(task, Task)

    def test_task_from_constructor(self, docker):
        """we can create Task from constructor"""
        task = Task(docker, "test_task_id")
        assert isinstance(task, Task)

    def test_task_id(self, docker):
        """task has an id"""
        task = Task(docker, "test_task_id")
        assert task.id() == "test_task_id"

    def test_tasks_list_without_swarm(self, docker, is_swarm_active):
        """listing tasks without swarm raises error"""
        if is_swarm_active:
            pytest.skip("Swarm is active - cannot test non-swarm behavior")
        tasks = docker.tasks()
        with pytest.raises(SystemError):
            tasks.list()

    def test_tasks_list_with_swarm(self, docker, require_swarm):
        """we can list tasks in swarm mode"""
        tasks = docker.tasks()
        result = tasks.list()
        assert isinstance(result, list)
        # List might be empty if no services exist

    def test_task_inspect_nonexistent(self, docker, require_swarm):
        """inspecting non-existent task raises error"""
        task = docker.tasks().get("nonexistent_task_id_12345")
        with pytest.raises(SystemError):
            task.inspect()

    def test_task_logs_nonexistent(self, docker, require_swarm):
        """getting logs from non-existent task returns empty"""
        task = docker.tasks().get("nonexistent_task_id_12345")
        # Logs on nonexistent task typically returns empty, not error
        result = task.logs(stdout=True, stderr=True)
        assert isinstance(result, str)


# =============================================================================
# Integration Tests (require full swarm setup)
# =============================================================================

class TestSwarmIntegration:
    """Integration tests that require a fully configured swarm."""

    def test_node_update(self, docker, require_swarm):
        """we can update node labels"""
        nodes_list = docker.nodes().list()
        if not nodes_list:
            pytest.skip("No nodes available")

        node_id = nodes_list[0].get("ID")
        node = docker.nodes().get(node_id)
        info = node.inspect()

        # Get the version for the update
        version = str(info.get("Version", {}).get("Index", 0))

        try:
            # Update with a test label
            node.update(
                version=version,
                labels={"test_label": "test_value"}
            )

            # Verify the update
            updated_info = node.inspect()
            labels = updated_info.get("Spec", {}).get("Labels", {})
            assert "test_label" in labels
            assert labels["test_label"] == "test_value"

        finally:
            # Clean up the test label
            try:
                new_info = node.inspect()
                new_version = str(new_info.get("Version", {}).get("Index", 0))
                labels = new_info.get("Spec", {}).get("Labels", {})
                if "test_label" in labels:
                    del labels["test_label"]
                node.update(version=new_version, labels=labels)
            except Exception:
                pass
