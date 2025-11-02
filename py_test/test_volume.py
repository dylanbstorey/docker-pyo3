from docker_pyo3.volume import Volumes,Volume
import datetime
import pytest


def test_volumes(docker):
    """volumes interface exists"""
    assert isinstance(docker.volumes(), Volumes)

def test_volumes(docker):
    """we can list volumes"""
    docker.volumes().create(name="test_volumes")
    vs = docker.volumes().list()
    assert isinstance(vs, dict)

def test_volumes_create(docker):
    """we can create&delete volumes"""
    docker.volumes().create(name="test_volumes")
    v = docker.volumes().get("test_volumes")
    assert isinstance(v, Volume)
    v.delete()

def test_volume_inspect(docker):
    """we can inspect a volume"""
    docker.volumes().create(name="test_volumes")
    v = docker.volumes().get("test_volumes")
    v.inspect()
    assert isinstance(v, Volume)
    v.delete()

def test_volume_with_labels(docker):
    """we can create volumes with labels"""
    labels = {"test": "label", "environment": "testing"}
    docker.volumes().create(name="test_volume_labels", labels=labels)
    v = docker.volumes().get("test_volume_labels")
    info = v.inspect()
    assert info["Labels"]["test"] == "label"
    assert info["Labels"]["environment"] == "testing"
    v.delete()

def test_volume_with_driver_opts(docker):
    """we can create volumes with driver options"""
    driver_opts = {"type": "tmpfs", "device": "tmpfs"}
    docker.volumes().create(name="test_volume_opts", driver="local", driver_opts=driver_opts)
    v = docker.volumes().get("test_volume_opts")
    info = v.inspect()
    assert isinstance(info, dict)
    v.delete()