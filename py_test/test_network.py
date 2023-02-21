from docker_pyo3.network import Networks,Network
import datetime
import pytest


def test_networks(docker):
    """networks interface exists"""
    assert isinstance(docker.networks(), Networks)


def test_networks_create(docker):
    """we can create a network"""
    try:
        docker.networks().create(name="test_network")
        n = docker.networks().get("test_network")
        assert isinstance(n,Network)
    except Exception as e:
        raise e
    finally:
        n.delete()


def test_networks_list(docker):
    """we can list network"""
    try:
        n = docker.networks().create(name="test_network")
        ns = docker.networks().list()
        assert isinstance(ns, list)
        assert len(ns) > 0
    except Exception as e:
        raise e
    finally:
        n.delete()

def test_networks_prune(docker):
    """we can prune networks"""
    docker.networks().prune()

def test_network_id(running_network):
    """networks have an id"""
    assert isinstance(running_network.id(),str)

def test_network_inspect(running_network):
    """we can inspect the network"""
    running_network.inspect()
    
def test_network_connect(running_network, running_container):
    """ we can connect and disconnect from network"""
    running_network.connect(running_container.id())
    running_network.disconnect(running_container.id())