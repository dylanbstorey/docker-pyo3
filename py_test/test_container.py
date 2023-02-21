from docker_pyo3.container import Containers,Container
import datetime
import pytest



def test_containers(docker):
    """containers is a containers instance"""
    assert isinstance(docker.containers(), Containers)

def test_containers_list(docker):
    """ we can list containers"""
    assert isinstance(docker.containers().list(all=True), list)


def test_create_container(docker, image_pull):
    """ we can create/delete a container"""
    c = docker.containers().create(image='busybox',name='weee')
    c.delete()
    pass
    
def test_containers_list(running_container,docker):
    """we can list container"""
    x = docker.containers().list(since='30s',sized=True, all=True)
    assert isinstance(x, list)

def test_containers_get(running_container):
    """we can get a container"""
    assert isinstance(running_container,Container)
    
def test_container_logs(running_container):
    """we can get container logs"""
    since = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(hours=100)
    logs = running_container.logs(stdout=True, stderr=True, timestamps=True, since=since)
    assert isinstance(logs, str)
    
def test_container_inspect(docker, running_container):
    """we can inspect a container"""
    assert isinstance(running_container.inspect(),dict)
    

    