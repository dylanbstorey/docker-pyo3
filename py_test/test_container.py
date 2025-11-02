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

def test_container_with_env(docker, image_pull):
    """we can create containers with environment variables"""
    env = ["TEST_VAR=hello", "ANOTHER_VAR=world"]
    c = docker.containers().create(image='busybox', name='test_env', env=env, command=["/bin/sh", "-c", "sleep 1"])
    info = c.inspect()
    assert "TEST_VAR=hello" in info["Config"]["Env"]
    assert "ANOTHER_VAR=world" in info["Config"]["Env"]
    c.delete()

def test_container_with_labels(docker, image_pull):
    """we can create containers with labels"""
    labels = {"test": "label", "app": "docker-pyo3"}
    c = docker.containers().create(image='busybox', name='test_labels', labels=labels)
    info = c.inspect()
    assert info["Config"]["Labels"]["test"] == "label"
    assert info["Config"]["Labels"]["app"] == "docker-pyo3"
    c.delete()

def test_container_with_command(docker, image_pull):
    """we can create containers with custom commands"""
    command = ["/bin/sh", "-c", "echo hello"]
    c = docker.containers().create(image='busybox', name='test_command', command=command)
    info = c.inspect()
    assert "/bin/sh" in info["Config"]["Cmd"]
    c.delete()

def test_container_with_entrypoint(docker, image_pull):
    """we can create containers with custom entrypoint"""
    entrypoint = ["/bin/sh"]
    c = docker.containers().create(image='busybox', name='test_entrypoint', entrypoint=entrypoint)
    info = c.inspect()
    assert info["Config"]["Entrypoint"] == ["/bin/sh"]
    c.delete()

def test_container_with_volumes(docker, image_pull):
    """we can create containers with volume mounts"""
    volumes = ["/tmp:/tmp:rw"]
    c = docker.containers().create(image='busybox', name='test_volumes', volumes=volumes)
    info = c.inspect()
    assert "/tmp:/tmp:rw" in info["HostConfig"]["Binds"]
    c.delete()

def test_container_with_restart_policy(docker, image_pull):
    """we can create containers with restart policy"""
    restart_policy = {"name": "on-failure", "maximum_retry_count": 3}
    c = docker.containers().create(image='busybox', name='test_restart', restart_policy=restart_policy)
    info = c.inspect()
    assert info["HostConfig"]["RestartPolicy"]["Name"] == "on-failure"
    assert info["HostConfig"]["RestartPolicy"]["MaximumRetryCount"] == 3
    c.delete()

def test_container_with_capabilities(docker, image_pull):
    """we can create containers with capabilities"""
    capabilities = ["NET_ADMIN", "SYS_TIME"]
    c = docker.containers().create(image='busybox', name='test_caps', capabilities=capabilities)
    info = c.inspect()
    assert "NET_ADMIN" in info["HostConfig"]["CapAdd"]
    assert "SYS_TIME" in info["HostConfig"]["CapAdd"]
    c.delete()

def test_container_with_devices(docker, image_pull):
    """we can create containers with device mappings"""
    devices = [
        {"PathOnHost": "/dev/null", "PathInContainer": "/dev/null1", "CgroupPermissions": "rwm"},
        {"PathOnHost": "/dev/zero", "PathInContainer": "/dev/zero1", "CgroupPermissions": "r"}
    ]
    c = docker.containers().create(image='busybox', name='test_devices', devices=devices)
    info = c.inspect()
    # Check that devices were added
    assert len(info["HostConfig"]["Devices"]) == 2
    assert info["HostConfig"]["Devices"][0]["PathOnHost"] == "/dev/null"
    assert info["HostConfig"]["Devices"][1]["PathOnHost"] == "/dev/zero"
    c.delete()


