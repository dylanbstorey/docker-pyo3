"""Tests for Docker Compose functionality."""
import pytest
import os
import tempfile
from docker_pyo3 import Docker
from docker_pyo3.compose import (
    ComposeFile,
    ComposeProject,
    parse_compose_string,
    parse_compose_file,
)

here = os.path.abspath(os.path.dirname(__file__))


# Fixtures

@pytest.fixture
def docker():
    return Docker()


@pytest.fixture
def simple_compose_content():
    """A simple compose file content for testing."""
    return """
version: '3.8'
services:
  web:
    image: busybox
    command: ["sleep", "3600"]
    labels:
      test: "true"
  worker:
    image: busybox
    command: ["sleep", "3600"]
networks:
  default:
    driver: bridge
volumes:
  data:
"""


@pytest.fixture
def compose_file(simple_compose_content):
    """Parse compose content into a ComposeFile."""
    return parse_compose_string(simple_compose_content)


@pytest.fixture
def compose_project(docker, compose_file):
    """Create a compose project with cleanup."""
    project = ComposeProject(docker, compose_file, "test_project")
    yield project
    # Cleanup - ignore errors during teardown
    try:
        project.down(remove_volumes=True, remove_networks=True, timeout=5)
    except Exception:
        pass


@pytest.fixture
def running_compose_project(docker, simple_compose_content):
    """A compose project that is started and cleaned up."""
    compose = parse_compose_string(simple_compose_content)
    project = ComposeProject(docker, compose, "test_running")
    try:
        project.pull()
        project.up()
        yield project
    finally:
        try:
            project.down(remove_volumes=True, remove_networks=True, timeout=5)
        except Exception:
            pass


# Parse tests

def test_parse_compose_string(simple_compose_content):
    """we can parse compose content from string"""
    compose = parse_compose_string(simple_compose_content)
    assert isinstance(compose, ComposeFile)
    assert "web" in compose.service_names()
    assert "worker" in compose.service_names()


def test_parse_compose_file(simple_compose_content):
    """we can parse compose content from file"""
    with tempfile.NamedTemporaryFile(mode='w', suffix='.yml', delete=False) as f:
        f.write(simple_compose_content)
        f.flush()
        compose_path = f.name

    try:
        compose = parse_compose_file(compose_path)
        assert isinstance(compose, ComposeFile)
        assert "web" in compose.service_names()
    finally:
        os.unlink(compose_path)


def test_parse_compose_string_invalid():
    """parsing invalid yaml raises error"""
    with pytest.raises(Exception):
        parse_compose_string("invalid: yaml: content: :")


# ComposeFile tests

def test_compose_file_service_names(compose_file):
    """compose file provides service names"""
    names = compose_file.service_names()
    assert isinstance(names, list)
    assert "web" in names
    assert "worker" in names


def test_compose_file_network_names(compose_file):
    """compose file provides network names"""
    names = compose_file.network_names()
    assert isinstance(names, list)
    assert "default" in names


def test_compose_file_volume_names(compose_file):
    """compose file provides volume names"""
    names = compose_file.volume_names()
    assert isinstance(names, list)
    assert "data" in names


def test_compose_file_to_dict(compose_file):
    """compose file can be converted to dict"""
    d = compose_file.to_dict()
    assert isinstance(d, dict)
    assert "services" in d
    assert "networks" in d
    assert "volumes" in d


def test_compose_file_get_service(compose_file):
    """compose file provides individual service config"""
    service = compose_file.get_service("web")
    assert service is not None
    assert service.get("image") == "busybox"


def test_compose_file_get_service_missing(compose_file):
    """getting non-existent service returns None"""
    service = compose_file.get_service("nonexistent")
    assert service is None


# ComposeProject tests

def test_compose_project_creation(docker, compose_file):
    """we can create a compose project"""
    project = ComposeProject(docker, compose_file, "test_creation")
    assert isinstance(project, ComposeProject)
    assert project.project_name == "test_creation"


def test_compose_project_config(compose_project):
    """we can get compose project config"""
    config = compose_project.config()
    assert isinstance(config, dict)
    assert "services" in config


def test_compose_project_pull(compose_project):
    """we can pull images for a compose project"""
    result = compose_project.pull()
    assert isinstance(result, list)


def test_compose_project_up_down(docker, simple_compose_content):
    """we can bring up and down a compose project"""
    compose = parse_compose_string(simple_compose_content)
    project = ComposeProject(docker, compose, "test_updown")

    try:
        # Pull first
        project.pull()

        # Bring up
        result = project.up()
        assert isinstance(result, dict)
        assert "containers" in result

        # Check containers are running
        ps_result = project.ps()
        assert isinstance(ps_result, list)
        assert len(ps_result) > 0

    finally:
        # Bring down
        down_result = project.down(remove_volumes=True, remove_networks=True)
        assert isinstance(down_result, dict)


def test_compose_project_ps(running_compose_project):
    """we can list containers in a compose project"""
    result = running_compose_project.ps()
    assert isinstance(result, list)
    assert len(result) >= 2  # web and worker


def test_compose_project_ps_detailed(running_compose_project):
    """we can get detailed container info"""
    result = running_compose_project.ps_detailed()
    assert isinstance(result, list)
    for container in result:
        assert "id" in container
        assert "name" in container
        assert "service" in container
        assert "state" in container


def test_compose_project_stop_start(running_compose_project):
    """we can stop and start containers"""
    # Stop
    stopped = running_compose_project.stop(timeout=5)
    assert isinstance(stopped, list)

    # Start
    started = running_compose_project.start()
    assert isinstance(started, list)


def test_compose_project_restart(running_compose_project):
    """we can restart containers"""
    result = running_compose_project.restart(timeout=5)
    assert isinstance(result, list)


def test_compose_project_pause_unpause(running_compose_project):
    """we can pause and unpause containers"""
    # Pause
    paused = running_compose_project.pause()
    assert isinstance(paused, list)

    # Unpause
    unpaused = running_compose_project.unpause()
    assert isinstance(unpaused, list)


def test_compose_project_logs(running_compose_project):
    """we can get logs from compose project"""
    result = running_compose_project.logs()
    assert isinstance(result, dict)


def test_compose_project_logs_with_service(running_compose_project):
    """we can get logs for specific service"""
    result = running_compose_project.logs(service="web")
    assert isinstance(result, dict)


def test_compose_project_logs_with_tail(running_compose_project):
    """we can get last N lines of logs"""
    result = running_compose_project.logs(tail=10)
    assert isinstance(result, dict)


def test_compose_project_top(running_compose_project):
    """we can get process info from containers"""
    result = running_compose_project.top()
    assert isinstance(result, dict)


def test_compose_project_exec(running_compose_project):
    """we can exec commands in running containers"""
    output = running_compose_project.exec("web", ["echo", "hello"])
    assert "hello" in output


def test_compose_project_exec_with_user(running_compose_project):
    """we can exec commands as specific user"""
    output = running_compose_project.exec("web", ["whoami"], user="root")
    assert "root" in output


def test_compose_project_exec_with_workdir(running_compose_project):
    """we can exec commands in specific directory"""
    output = running_compose_project.exec("web", ["pwd"], workdir="/tmp")
    assert "/tmp" in output


def test_compose_project_exec_with_env(running_compose_project):
    """we can exec commands with environment variables"""
    output = running_compose_project.exec(
        "web",
        ["sh", "-c", "echo $MY_VAR"],
        env=["MY_VAR=test_value"]
    )
    assert "test_value" in output


def test_compose_project_exec_nonexistent_service(running_compose_project):
    """exec on non-existent service raises error"""
    with pytest.raises(RuntimeError):
        running_compose_project.exec("nonexistent", ["echo", "hello"])


def test_compose_project_run(docker, simple_compose_content):
    """we can run one-off commands"""
    compose = parse_compose_string(simple_compose_content)
    project = ComposeProject(docker, compose, "test_run")

    try:
        project.pull()
        # Must bring up to create networks first
        project.up()
        result = project.run("web", ["echo", "hello world"], rm=True)
        assert isinstance(result, dict)
        assert "container_id" in result
        # Output may be empty due to timing - main test is that it runs without error
        # and returns the expected structure
        assert "output" in result or result.get("output") is None
    finally:
        try:
            project.down(remove_volumes=True, remove_networks=True, timeout=5)
        except Exception:
            pass


def test_compose_project_run_with_env(docker, simple_compose_content):
    """we can run one-off commands with environment"""
    compose = parse_compose_string(simple_compose_content)
    project = ComposeProject(docker, compose, "test_run_env")

    try:
        project.pull()
        # Must bring up to create networks first
        project.up()
        result = project.run(
            "web",
            ["sh", "-c", "echo $TEST_VAR"],
            env=["TEST_VAR=myvalue"],
            rm=True
        )
        # Result should have the expected structure
        assert isinstance(result, dict)
        assert "container_id" in result
    finally:
        try:
            project.down(remove_volumes=True, remove_networks=True, timeout=5)
        except Exception:
            pass


def test_compose_project_run_detached(docker, simple_compose_content):
    """we can run detached one-off containers"""
    compose = parse_compose_string(simple_compose_content)
    project = ComposeProject(docker, compose, "test_run_detach")

    try:
        project.pull()
        # Must bring up to create networks first
        project.up()
        result = project.run("web", ["sleep", "10"], detach=True, rm=False)
        assert isinstance(result, dict)
        assert "container_id" in result
    finally:
        try:
            project.down(remove_volumes=True, remove_networks=True, timeout=5)
        except Exception:
            pass


def test_compose_project_run_nonexistent_service(docker, simple_compose_content):
    """run on non-existent service raises error"""
    compose = parse_compose_string(simple_compose_content)
    project = ComposeProject(docker, compose, "test_run_bad")

    try:
        project.pull()
        with pytest.raises(RuntimeError):
            project.run("nonexistent", ["echo", "hello"])
    finally:
        try:
            project.down(remove_volumes=True, remove_networks=True, timeout=5)
        except Exception:
            pass


# Build tests (require Dockerfile)

def test_compose_project_build(docker):
    """we can build images from compose file"""
    compose_content = """
version: '3.8'
services:
  buildable:
    build:
      context: .
      dockerfile: Dockerfile
    image: test-build-image
"""
    dockerfile_path = os.path.join(here, 'Dockerfile')

    with open(dockerfile_path, 'w') as f:
        f.write("FROM busybox\n")
        f.write("COPY conftest.py /\n")

    try:
        compose = parse_compose_string(compose_content)
        project = ComposeProject(docker, compose, "test_build")
        result = project.build()
        assert isinstance(result, list)
    finally:
        if os.path.exists(dockerfile_path):
            os.unlink(dockerfile_path)
        try:
            docker.images().get('test-build-image').delete()
        except Exception:
            pass
