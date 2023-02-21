import pytest
import docker_pyo3
import os

try:
    pw = os.environ.get("DOCKER_PASSWORD", None)
    un = os.environ.get("DOCKER_USERNAME",None)
    if pw and un:
        print("PULLING WITH ENVIRONMENTAL VARIABLES")
        docker.images().pull(image='busybox',auth_password = dict(username=un,password=pw))
    else:
        docker.images().pull(image='busybox')
except Exception as e:
    print("might fail because of docker pull limits/ container availability")


def pytest_itemcollected(item):
    """
    use test doc strings as messages for the testing suite
    :param item:
    :return:
    """
    if item._obj.__doc__:
        item._nodeid = f"{item.obj.__doc__.strip().ljust(50,' ')[:50]}{str(item._nodeid).ljust(100,' ')[:50]}"

@pytest.fixture
def docker():
    return docker_pyo3.Docker()


@pytest.fixture
def image_pull():
    docker_pyo3.Docker().images().get('busybox')
    yield
    
    

@pytest.fixture
def running_container():
    image = docker_pyo3.Docker().images().get('busybox')
    container = docker_pyo3.Docker().containers().create(image='busybox',name="busybox")
    yield container
    container.delete()
    

@pytest.fixture
def running_network():
    docker_pyo3.Docker().networks().create(name="test_network")
    n = docker_pyo3.Docker().networks().get("test_network")
    yield n
    n.delete()