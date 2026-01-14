# docker-pyo3

Python bindings to the Rust `docker_api` crate.

## Installation

```bash
pip install docker_pyo3
```

## Quick Start

```python
from docker_pyo3 import Docker

# Connect to the daemon
docker = Docker()

# Pull an image
docker.images().pull(image='busybox')

# Build an image
docker.images().build(path="path/to/dockerfile", dockerfile='Dockerfile', tag='test-image')

# Create and start a container
container = docker.containers().create(image='busybox', name='my-container')
container.start()

# List running containers
containers = docker.containers().list()

# Stop and remove the container
container.stop()
container.delete()
```

## API Reference

### Docker Client

The main entry point for interacting with the Docker daemon.

#### `Docker(uri=None)`

Create a new Docker client.

**Parameters:**
- `uri` (str, optional): URI to connect to the Docker daemon. Defaults to system default:
  - Unix: `unix:///var/run/docker.sock`
  - Windows: `tcp://localhost:2375`

**Example:**
```python
# Connect to default socket
docker = Docker()

# Connect to custom socket
docker = Docker("unix:///custom/docker.sock")

# Connect to TCP endpoint
docker = Docker("tcp://localhost:2375")
```

#### Methods

##### `version()`
Get Docker version information.

**Returns:** `dict` - Version information including API version, OS, architecture, etc.

```python
version_info = docker.version()
print(version_info)
```

##### `info()`
Get Docker system information.

**Returns:** `dict` - System information including containers count, images count, storage driver, etc.

```python
info = docker.info()
print(f"Total containers: {info['Containers']}")
```

##### `ping()`
Ping the Docker daemon to verify connectivity.

**Returns:** `dict` - Ping response from the daemon

```python
response = docker.ping()
```

##### `data_usage()`
Get data usage information for Docker objects.

**Returns:** `dict` - Data usage statistics for containers, images, volumes, and build cache

```python
usage = docker.data_usage()
print(f"Images size: {usage['Images']}")
```

##### `containers()`
Get the Containers interface for managing containers.

**Returns:** `Containers` - Interface for container operations

```python
containers = docker.containers()
```

##### `images()`
Get the Images interface for managing images.

**Returns:** `Images` - Interface for image operations

```python
images = docker.images()
```

##### `networks()`
Get the Networks interface for managing networks.

**Returns:** `Networks` - Interface for network operations

```python
networks = docker.networks()
```

##### `volumes()`
Get the Volumes interface for managing volumes.

**Returns:** `Volumes` - Interface for volume operations

```python
volumes = docker.volumes()
```

##### `nodes()`
Get the Nodes interface for managing Swarm nodes.

**Returns:** `Nodes` - Interface for node operations (requires Swarm mode)

```python
nodes = docker.nodes()
```

##### `services()`
Get the Services interface for managing Swarm services.

**Returns:** `Services` - Interface for service operations (requires Swarm mode)

```python
services = docker.services()
```

##### `tasks()`
Get the Tasks interface for managing Swarm tasks.

**Returns:** `Tasks` - Interface for task operations (requires Swarm mode)

```python
tasks = docker.tasks()
```

##### `secrets()`
Get the Secrets interface for managing Swarm secrets.

**Returns:** `Secrets` - Interface for secret operations (requires Swarm mode)

```python
secrets = docker.secrets()
```

##### `configs()`
Get the Configs interface for managing Swarm configs.

**Returns:** `Configs` - Interface for config operations (requires Swarm mode)

```python
configs = docker.configs()
```

##### `plugins()`
Get the Plugins interface for managing Docker plugins.

**Returns:** `Plugins` - Interface for plugin operations

```python
plugins = docker.plugins()
```

---

### Containers

Interface for managing Docker containers.

#### `get(id)`
Get a specific container by ID or name.

**Parameters:**
- `id` (str): Container ID or name

**Returns:** `Container` - Container instance

```python
container = docker.containers().get("my-container")
```

#### `list(all=None, since=None, before=None, sized=None)`
List containers.

**Parameters:**
- `all` (bool, optional): Show all containers (default shows only running)
- `since` (str, optional): Show containers created since this container ID
- `before` (str, optional): Show containers created before this container ID
- `sized` (bool, optional): Include size information

**Returns:** `list[dict]` - List of container information dictionaries

```python
# List only running containers
running = docker.containers().list()

# List all containers
all_containers = docker.containers().list(all=True)

# List with size information
containers_with_size = docker.containers().list(all=True, sized=True)
```

#### `prune()`
Remove stopped containers.

**Returns:** `dict` - Prune results including containers deleted and space reclaimed

```python
result = docker.containers().prune()
print(f"Space reclaimed: {result['SpaceReclaimed']}")
```

#### `create(image, **kwargs)`
Create a new container.

**Parameters:**
- `image` (str): Image name to use for the container
- `attach_stderr` (bool, optional): Attach to stderr
- `attach_stdin` (bool, optional): Attach to stdin
- `attach_stdout` (bool, optional): Attach to stdout
- `auto_remove` (bool, optional): Automatically remove the container when it exits
- `capabilities` (list[str], optional): Linux capabilities to add (e.g., `["NET_ADMIN", "SYS_TIME"]`)
- `command` (list[str], optional): Command to run (e.g., `["/bin/sh", "-c", "echo hello"]`)
- `cpu_shares` (int, optional): CPU shares (relative weight)
- `cpus` (float, optional): Number of CPUs
- `devices` (list[dict], optional): Device mappings, each a dict with `PathOnHost`, `PathInContainer`, `CgroupPermissions`
- `entrypoint` (list[str], optional): Entrypoint (e.g., `["/bin/sh"]`)
- `env` (list[str], optional): Environment variables (e.g., `["VAR=value"]`)
- `expose` (list[dict], optional): Port mappings to expose (e.g., `[{"srcport": 8080, "hostport": 8000, "protocol": "tcp"}]`)
- `extra_hosts` (list[str], optional): Extra host-to-IP mappings (e.g., `["hostname:192.168.1.1"]`)
- `labels` (dict, optional): Labels (e.g., `{"app": "myapp", "env": "prod"}`)
- `links` (list[str], optional): Links to other containers
- `log_driver` (str, optional): Logging driver (e.g., "json-file", "syslog")
- `memory` (int, optional): Memory limit in bytes
- `memory_swap` (int, optional): Total memory limit (memory + swap)
- `name` (str, optional): Container name
- `nano_cpus` (int, optional): CPU quota in units of 10^-9 CPUs
- `network_mode` (str, optional): Network mode (e.g., "bridge", "host", "none")
- `privileged` (bool, optional): Give extended privileges
- `publish` (list[dict], optional): Ports to publish (e.g., `[{"port": 8080, "protocol": "tcp"}]`)
- `publish_all_ports` (bool, optional): Publish all exposed ports to random ports
- `restart_policy` (dict, optional): Restart policy with `name` and `maximum_retry_count`
- `security_options` (list[str], optional): Security options (e.g., `["label=user:USER"]`)
- `stop_signal` (str, optional): Signal to stop the container
- `stop_signal_num` (int, optional): Signal number to stop the container
- `stop_timeout` (timedelta, optional): Timeout for stopping the container
- `tty` (bool, optional): Allocate a pseudo-TTY
- `user` (str, optional): Username or UID
- `userns_mode` (str, optional): User namespace mode
- `volumes` (list[str], optional): Volume bindings (e.g., `["/host:/container:rw"]`)
- `volumes_from` (list[str], optional): Mount volumes from other containers
- `working_dir` (str, optional): Working directory inside the container

**Returns:** `Container` - Created container instance

```python
# Simple container
container = docker.containers().create(
    image='busybox',
    name='my-container'
)

# Container with environment variables
container = docker.containers().create(
    image='busybox',
    name='my-app',
    env=["API_KEY=secret", "ENV=production"],
    command=["/bin/sh", "-c", "echo $ENV"]
)

# Container with port mapping and volumes
container = docker.containers().create(
    image='nginx',
    name='web-server',
    expose=[{"srcport": 80, "hostport": 8080, "protocol": "tcp"}],
    volumes=["/data:/usr/share/nginx/html:ro"]
)

# Container with labels and restart policy
container = docker.containers().create(
    image='redis',
    name='cache',
    labels={"app": "cache", "version": "1.0"},
    restart_policy={"name": "on-failure", "maximum_retry_count": 3}
)

# Container with devices
container = docker.containers().create(
    image='ubuntu',
    devices=[
        {"PathOnHost": "/dev/null", "PathInContainer": "/dev/null1", "CgroupPermissions": "rwm"}
    ]
)
```

---

### Container

Represents an individual Docker container.

#### `id()`
Get the container ID.

**Returns:** `str` - Container ID

```python
container_id = container.id()
```

#### `inspect()`
Inspect the container to get detailed information.

**Returns:** `dict` - Detailed container information including config, state, mounts, etc.

```python
info = container.inspect()
print(f"Status: {info['State']['Status']}")
print(f"IP Address: {info['NetworkSettings']['IPAddress']}")
```

#### `logs(stdout=None, stderr=None, timestamps=None, n_lines=None, all=None, since=None)`
Get container logs.

**Parameters:**
- `stdout` (bool, optional): Include stdout
- `stderr` (bool, optional): Include stderr
- `timestamps` (bool, optional): Include timestamps
- `n_lines` (int, optional): Number of lines to return from the end of logs
- `all` (bool, optional): Return all logs
- `since` (datetime, optional): Only return logs since this datetime

**Returns:** `str` - Container logs

```python
# Get all logs
logs = container.logs(stdout=True, stderr=True)

# Get last 100 lines
logs = container.logs(stdout=True, n_lines=100)
```

#### `start()`
Start the container.

**Returns:** `None`

```python
container.start()
```

#### `stop(wait=None)`
Stop the container.

**Parameters:**
- `wait` (timedelta, optional): Time to wait before killing the container

**Returns:** `None`

```python
from datetime import timedelta

# Stop immediately
container.stop()

# Wait 30 seconds before force killing
container.stop(wait=timedelta(seconds=30))
```

#### `restart(wait=None)`
Restart the container.

**Parameters:**
- `wait` (timedelta, optional): Time to wait before killing the container

**Returns:** `None`

```python
container.restart()
```

#### `kill(signal=None)`
Kill the container by sending a signal.

**Parameters:**
- `signal` (str, optional): Signal to send (e.g., "SIGKILL", "SIGTERM")

**Returns:** `None`

```python
# Send SIGTERM
container.kill(signal="SIGTERM")

# Send SIGKILL
container.kill(signal="SIGKILL")
```

#### `pause()`
Pause the container.

**Returns:** `None`

```python
container.pause()
```

#### `unpause()`
Unpause the container.

**Returns:** `None`

```python
container.unpause()
```

#### `rename(name)`
Rename the container.

**Parameters:**
- `name` (str): New name for the container

**Returns:** `None`

```python
container.rename("new-container-name")
```

#### `wait()`
Wait for the container to stop.

**Returns:** `dict` - Wait response including status code

```python
result = container.wait()
print(f"Exit code: {result['StatusCode']}")
```

#### `exec(command, env=None, attach_stdout=None, attach_stderr=None, detach_keys=None, tty=None, privileged=None, user=None, working_dir=None)`
Execute a command in the running container.

**Parameters:**
- `command` (list[str]): Command to execute (e.g., `["/bin/sh", "-c", "ls"]`)
- `env` (list[str], optional): Environment variables (e.g., `["VAR=value"]`)
- `attach_stdout` (bool, optional): Attach to stdout
- `attach_stderr` (bool, optional): Attach to stderr
- `detach_keys` (str, optional): Override key sequence for detaching
- `tty` (bool, optional): Allocate a pseudo-TTY
- `privileged` (bool, optional): Run with extended privileges
- `user` (str, optional): Username or UID
- `working_dir` (str, optional): Working directory for the exec session

**Returns:** `None`

```python
# Execute a simple command
container.exec(command=["/bin/sh", "-c", "ls -la"])

# Execute with environment variables
container.exec(
    command=["printenv"],
    env=["MY_VAR=hello"]
)
```

#### `delete()`
Delete the container.

**Returns:** `None`

```python
container.delete()
```

---

### Images

Interface for managing Docker images.

#### `get(name)`
Get a specific image by name, ID, or tag.

**Parameters:**
- `name` (str): Image name, ID, or tag (e.g., "busybox", "busybox:latest")

**Returns:** `Image` - Image instance

```python
image = docker.images().get("busybox:latest")
```

#### `list(all=None, digests=None, filter=None)`
List images.

**Parameters:**
- `all` (bool, optional): Show all images (default hides intermediate images)
- `digests` (bool, optional): Show digests
- `filter` (dict, optional): Filter images by:
  - `{"type": "dangling"}` - dangling images
  - `{"type": "label", "key": "foo", "value": "bar"}` - by label
  - `{"type": "before", "value": "image:tag"}` - images before specified
  - `{"type": "since", "value": "image:tag"}` - images since specified

**Returns:** `list[dict]` - List of image information dictionaries

```python
# List all images
all_images = docker.images().list(all=True)

# List dangling images
dangling = docker.images().list(filter={"type": "dangling"})

# List images with specific label
labeled = docker.images().list(filter={"type": "label", "key": "app", "value": "web"})
```

#### `prune()`
Remove unused images.

**Returns:** `dict` - Prune results including images deleted and space reclaimed

```python
result = docker.images().prune()
print(f"Space reclaimed: {result['SpaceReclaimed']}")
```

#### `build(path, **kwargs)`
Build an image from a Dockerfile.

**Parameters:**
- `path` (str): Path to build context directory
- `dockerfile` (str, optional): Path to Dockerfile relative to build context
- `tag` (str, optional): Tag for the built image (e.g., "myimage:latest")
- `extra_hosts` (str, optional): Extra hosts to add to /etc/hosts
- `remote` (str, optional): Remote repository URL
- `quiet` (bool, optional): Suppress build output
- `nocahe` (bool, optional): Do not use cache when building
- `pull` (str, optional): Attempt to pull newer version of base image
- `rm` (bool, optional): Remove intermediate containers after build
- `forcerm` (bool, optional): Always remove intermediate containers
- `memory` (int, optional): Memory limit in bytes
- `memswap` (int, optional): Total memory limit (memory + swap)
- `cpu_shares` (int, optional): CPU shares (relative weight)
- `cpu_set_cpus` (str, optional): CPUs to allow execution (e.g., "0-3", "0,1")
- `cpu_period` (int, optional): CPU CFS period in microseconds
- `cpu_quota` (int, optional): CPU CFS quota in microseconds
- `shm_size` (int, optional): Size of /dev/shm in bytes
- `squash` (bool, optional): Squash newly built layers into single layer
- `network_mode` (str, optional): Network mode (e.g., "bridge", "host", "none")
- `platform` (str, optional): Target platform (e.g., "linux/amd64")
- `target` (str, optional): Build stage to target
- `outputs` (str, optional): Output configuration
- `labels` (dict, optional): Labels (e.g., `{"version": "1.0"}`)

**Returns:** `dict` - Build result information

```python
# Simple build
docker.images().build(
    path="/path/to/context",
    dockerfile="Dockerfile",
    tag="myapp:latest"
)

# Build with labels and no cache
docker.images().build(
    path="/path/to/context",
    tag="myapp:v1.0",
    nocahe=True,
    labels={"version": "1.0", "env": "production"}
)

# Build with resource limits
docker.images().build(
    path="/path/to/context",
    tag="myapp:latest",
    memory=1073741824,  # 1GB
    cpu_shares=512
)
```

#### `pull(image=None, src=None, repo=None, tag=None, auth_password=None, auth_token=None)`
Pull an image from a registry.

**Parameters:**
- `image` (str, optional): Image name to pull (e.g., "busybox", "ubuntu:latest")
- `src` (str, optional): Source repository
- `repo` (str, optional): Repository to pull from
- `tag` (str, optional): Tag to pull
- `auth_password` (dict, optional): Password authentication with `username`, `password`, `email`, `server_address`
- `auth_token` (dict, optional): Token authentication with `identity_token`

**Returns:** `dict` - Pull result information

```python
# Pull public image
docker.images().pull(image="busybox:latest")

# Pull with authentication
docker.images().pull(
    image="myregistry.com/myapp:latest",
    auth_password={
        "username": "user",
        "password": "pass",
        "server_address": "myregistry.com"
    }
)

# Pull with token authentication
docker.images().pull(
    image="myregistry.com/myapp:latest",
    auth_token={"identity_token": "my-token"}
)
```

---

### Image

Represents an individual Docker image.

#### `name()`
Get the image name.

**Returns:** `str` - Image name

```python
name = image.name()
```

#### `inspect()`
Inspect the image to get detailed information.

**Returns:** `dict` - Detailed image information including config, layers, etc.

```python
info = image.inspect()
print(f"Size: {info['Size']}")
print(f"Architecture: {info['Architecture']}")
```

#### `delete()`
Delete the image.

**Returns:** `str` - Deletion result information

```python
result = image.delete()
```

#### `history()`
Get the image history.

**Returns:** `str` - Image history information

```python
history = image.history()
```

#### `export(path=None)`
Export the image to a tar file.

**Parameters:**
- `path` (str, optional): Path to save the exported tar file

**Returns:** `str` - Path to the exported file

```python
exported_path = image.export(path="/tmp/myimage.tar")
```

#### `tag(repo=None, tag=None)`
Tag the image with a new name and/or tag.

**Parameters:**
- `repo` (str, optional): Repository name (e.g., "myrepo/myimage")
- `tag` (str, optional): Tag name (e.g., "v1.0", "latest")

**Returns:** `None`

```python
# Tag with new repository
image.tag(repo="myrepo/myimage", tag="latest")

# Tag with version
image.tag(repo="myrepo/myimage", tag="v1.0")
```

#### `push(auth_password=None, auth_token=None, tag=None)`
Push the image to a registry.

**Parameters:**
- `auth_password` (dict, optional): Password authentication with `username`, `password`, `email`, `server_address`
- `auth_token` (dict, optional): Token authentication with `identity_token`
- `tag` (str, optional): Tag to push

**Returns:** `None`

```python
# Push with authentication
image.push(
    auth_password={
        "username": "user",
        "password": "pass",
        "server_address": "myregistry.com"
    },
    tag="latest"
)
```

---

### Networks

Interface for managing Docker networks.

#### `get(id)`
Get a specific network by ID or name.

**Parameters:**
- `id` (str): Network ID or name

**Returns:** `Network` - Network instance

```python
network = docker.networks().get("my-network")
```

#### `list()`
List all networks.

**Returns:** `list[dict]` - List of network information dictionaries

```python
networks = docker.networks().list()
for network in networks:
    print(f"{network['Name']}: {network['Driver']}")
```

#### `prune()`
Remove unused networks.

**Returns:** `dict` - Prune results including networks deleted

```python
result = docker.networks().prune()
```

#### `create(name, **kwargs)`
Create a new network.

**Parameters:**
- `name` (str): Network name
- `check_duplicate` (bool, optional): Check for duplicate networks with the same name
- `driver` (str, optional): Network driver (e.g., "bridge", "overlay")
- `internal` (bool, optional): Restrict external access to the network
- `attachable` (bool, optional): Enable manual container attachment
- `ingress` (bool, optional): Create an ingress network
- `enable_ipv6` (bool, optional): Enable IPv6 networking
- `options` (dict, optional): Driver-specific options
- `labels` (dict, optional): Labels (e.g., `{"env": "prod"}`)

**Returns:** `Network` - Created network instance

```python
# Simple bridge network
network = docker.networks().create(name="my-network")

# Custom network with options
network = docker.networks().create(
    name="app-network",
    driver="bridge",
    labels={"app": "myapp", "env": "production"},
    options={"com.docker.network.bridge.name": "docker1"}
)

# Internal network
network = docker.networks().create(
    name="internal-network",
    driver="bridge",
    internal=True
)
```

---

### Network

Represents an individual Docker network.

#### `id()`
Get the network ID.

**Returns:** `str` - Network ID

```python
network_id = network.id()
```

#### `inspect()`
Inspect the network to get detailed information.

**Returns:** `dict` - Detailed network information including config, containers, etc.

```python
info = network.inspect()
print(f"Subnet: {info['IPAM']['Config'][0]['Subnet']}")
```

#### `delete()`
Delete the network.

**Returns:** `None`

```python
network.delete()
```

#### `connect(container_id, **kwargs)`
Connect a container to this network.

**Parameters:**
- `container_id` (str): Container ID or name to connect
- `aliases` (list[str], optional): Network aliases for the container
- `links` (list[str], optional): Links to other containers
- `network_id` (str, optional): Network ID
- `endpoint_id` (str, optional): Endpoint ID
- `gateway` (str, optional): IPv4 gateway address
- `ipv4` (str, optional): IPv4 address for the container
- `prefix_len` (int, optional): IPv4 prefix length
- `ipv6_gateway` (str, optional): IPv6 gateway address
- `ipv6` (str, optional): IPv6 address for the container
- `ipv6_prefix_len` (int, optional): IPv6 prefix length
- `mac` (str, optional): MAC address
- `driver_opts` (dict, optional): Driver-specific options
- `ipam_config` (dict, optional): IPAM configuration with `ipv4`, `ipv6`, `link_local_ips`

**Returns:** `None`

```python
# Simple connect
network.connect("my-container")

# Connect with custom IP and aliases
network.connect(
    "my-container",
    ipv4="172.20.0.5",
    aliases=["app", "web"]
)

# Connect with IPAM configuration
network.connect(
    "my-container",
    ipam_config={
        "ipv4": "172.20.0.10",
        "ipv6": "2001:db8::10"
    }
)
```

#### `disconnect(container_id, force=None)`
Disconnect a container from this network.

**Parameters:**
- `container_id` (str): Container ID or name to disconnect
- `force` (bool, optional): Force disconnect even if container is running

**Returns:** `None`

```python
# Graceful disconnect
network.disconnect("my-container")

# Force disconnect
network.disconnect("my-container", force=True)
```

---

### Volumes

Interface for managing Docker volumes.

#### `get(name)`
Get a specific volume by name.

**Parameters:**
- `name` (str): Volume name

**Returns:** `Volume` - Volume instance

```python
volume = docker.volumes().get("my-volume")
```

#### `list()`
List all volumes.

**Returns:** `dict` - Volume list information

```python
volumes_info = docker.volumes().list()
for volume in volumes_info['Volumes']:
    print(f"{volume['Name']}: {volume['Driver']}")
```

#### `prune()`
Remove unused volumes.

**Returns:** `dict` - Prune results including volumes deleted and space reclaimed

```python
result = docker.volumes().prune()
print(f"Space reclaimed: {result['SpaceReclaimed']}")
```

#### `create(name=None, driver=None, driver_opts=None, labels=None)`
Create a new volume.

**Parameters:**
- `name` (str, optional): Volume name
- `driver` (str, optional): Volume driver (e.g., "local")
- `driver_opts` (dict, optional): Driver-specific options
- `labels` (dict, optional): Labels (e.g., `{"env": "prod"}`)

**Returns:** `dict` - Created volume information

```python
# Simple volume
volume_info = docker.volumes().create(name="my-volume")

# Volume with labels
volume_info = docker.volumes().create(
    name="app-data",
    labels={"app": "myapp", "env": "production"}
)

# Volume with driver options
volume_info = docker.volumes().create(
    name="tmpfs-volume",
    driver="local",
    driver_opts={"type": "tmpfs", "device": "tmpfs"}
)
```

---

### Volume

Represents an individual Docker volume.

#### `name()`
Get the volume name.

**Returns:** `str` - Volume name

```python
volume_name = volume.name()
```

#### `inspect()`
Inspect the volume to get detailed information.

**Returns:** `dict` - Detailed volume information including driver, mountpoint, etc.

```python
info = volume.inspect()
print(f"Mountpoint: {info['Mountpoint']}")
print(f"Driver: {info['Driver']}")
```

#### `delete()`
Delete the volume.

**Returns:** `None`

```python
volume.delete()
```

---

### Docker Compose

The compose module provides Docker Compose-like functionality for managing multi-container applications.

#### Parsing Compose Files

##### `parse_compose_file(path)`
Parse a Docker Compose file.

**Parameters:**
- `path` (str): Path to the compose file (docker-compose.yml)

**Returns:** `ComposeFile` - Parsed compose file instance

```python
from docker_pyo3.compose import parse_compose_file

compose = parse_compose_file("/path/to/docker-compose.yml")
```

##### `parse_compose_string(content)`
Parse Docker Compose content from a string.

**Parameters:**
- `content` (str): YAML content of the compose file

**Returns:** `ComposeFile` - Parsed compose file instance

```python
from docker_pyo3.compose import parse_compose_string

compose_content = """
version: '3.8'
services:
  web:
    image: nginx
    ports:
      - "8080:80"
  db:
    image: postgres
    environment:
      POSTGRES_PASSWORD: secret
"""
compose = parse_compose_string(compose_content)
```

#### ComposeFile

Represents a parsed Docker Compose file.

##### `service_names()`
Get list of service names defined in the compose file.

**Returns:** `list[str]` - List of service names

```python
services = compose.service_names()  # ['web', 'db']
```

##### `network_names()`
Get list of network names defined in the compose file.

**Returns:** `list[str]` - List of network names

##### `volume_names()`
Get list of volume names defined in the compose file.

**Returns:** `list[str]` - List of volume names

##### `get_service(name)`
Get configuration for a specific service.

**Parameters:**
- `name` (str): Service name

**Returns:** `dict` or `None` - Service configuration or None if not found

```python
web_config = compose.get_service("web")
print(web_config["image"])  # 'nginx'
```

##### `to_dict()`
Convert the compose file to a dictionary.

**Returns:** `dict` - The full compose configuration

#### ComposeProject

Manages a Docker Compose project (networks, volumes, containers).

##### `ComposeProject(docker, compose_file, project_name)`
Create a new compose project.

**Parameters:**
- `docker` (Docker): Docker client instance
- `compose_file` (ComposeFile): Parsed compose file
- `project_name` (str): Name prefix for all created resources

```python
from docker_pyo3 import Docker
from docker_pyo3.compose import parse_compose_file, ComposeProject

docker = Docker()
compose = parse_compose_file("docker-compose.yml")
project = ComposeProject(docker, compose, "myapp")
```

##### `up(detach=None)`
Bring up the compose project (create networks, volumes, containers).

**Parameters:**
- `detach` (bool, optional): Run containers in background (default: True)

**Returns:** `dict` - Results including created network IDs, volume names, and container IDs

```python
result = project.up()
print(f"Created containers: {result['containers']}")
```

##### `down(remove_volumes=None, remove_networks=None, timeout=None)`
Bring down the compose project.

**Parameters:**
- `remove_volumes` (bool, optional): Also remove named volumes (default: False)
- `remove_networks` (bool, optional): Also remove networks (default: True)
- `timeout` (int, optional): Timeout in seconds for stopping containers (default: 10)

**Returns:** `dict` - Results including removed resources

```python
project.down(remove_volumes=True)
```

##### `ps()`
List container IDs for this project.

**Returns:** `list[str]` - List of container IDs

##### `ps_detailed()`
Get detailed information about project containers.

**Returns:** `list[dict]` - List of container info with id, name, service, state, status, image

```python
containers = project.ps_detailed()
for c in containers:
    print(f"{c['service']}: {c['state']}")
```

##### `start()`
Start all stopped containers in the project.

**Returns:** `list[str]` - List of started container IDs

##### `stop(timeout=None)`
Stop all running containers.

**Parameters:**
- `timeout` (int, optional): Timeout in seconds (default: 10)

**Returns:** `list[str]` - List of stopped container IDs

##### `restart(timeout=None)`
Restart all containers.

**Parameters:**
- `timeout` (int, optional): Timeout in seconds (default: 10)

**Returns:** `list[str]` - List of restarted container IDs

##### `pause()`
Pause all running containers.

**Returns:** `list[str]` - List of paused container IDs

##### `unpause()`
Unpause all paused containers.

**Returns:** `list[str]` - List of unpaused container IDs

##### `pull()`
Pull images for all services.

**Returns:** `list[str]` - List of pulled images

##### `build(no_cache=None, pull=None)`
Build images for services with build configurations.

**Parameters:**
- `no_cache` (bool, optional): Do not use cache (default: False)
- `pull` (bool, optional): Pull newer base images (default: False)

**Returns:** `list[str]` - List of built services

##### `logs(service=None, tail=None, timestamps=None)`
Get logs from containers.

**Parameters:**
- `service` (str, optional): Only get logs from this service
- `tail` (int, optional): Number of lines from end
- `timestamps` (bool, optional): Include timestamps (default: False)

**Returns:** `dict[str, str]` - Mapping of container ID to logs

```python
logs = project.logs(service="web", tail=100)
```

##### `top(ps_args=None)`
Get running processes from containers.

**Parameters:**
- `ps_args` (str, optional): Arguments to pass to ps command

**Returns:** `dict[str, dict]` - Mapping of container ID to process info

##### `config()`
Get the compose configuration as a dictionary.

**Returns:** `dict` - The compose configuration

##### `exec(service, command, user=None, workdir=None, env=None, privileged=None, tty=None)`
Execute a command in a running service container.

**Parameters:**
- `service` (str): Service name
- `command` (list[str]): Command to execute
- `user` (str, optional): User to run as
- `workdir` (str, optional): Working directory
- `env` (list[str], optional): Environment variables (e.g., `["VAR=value"]`)
- `privileged` (bool, optional): Extended privileges (default: False)
- `tty` (bool, optional): Allocate pseudo-TTY (default: False)

**Returns:** `str` - Command output

```python
output = project.exec("web", ["ls", "-la", "/app"])

# With environment variables
output = project.exec(
    "web",
    ["sh", "-c", "echo $MY_VAR"],
    env=["MY_VAR=hello"]
)
```

##### `run(service, command=None, user=None, workdir=None, env=None, rm=None, detach=None)`
Run a one-off command in a new container.

**Parameters:**
- `service` (str): Service name
- `command` (list[str], optional): Command to execute (uses service default if not provided)
- `user` (str, optional): User to run as
- `workdir` (str, optional): Working directory
- `env` (list[str], optional): Additional environment variables
- `rm` (bool, optional): Remove container after exit (default: True)
- `detach` (bool, optional): Run in background (default: False)

**Returns:** `dict` - Result with container_id, output (if not detached), exit_code

```python
# Run a one-off command
result = project.run("web", ["python", "manage.py", "migrate"])
print(result["output"])

# Run detached
result = project.run("worker", ["celery", "worker"], detach=True)
print(f"Container ID: {result['container_id']}")
```

---

### Plugins

Interface for managing Docker plugins.

#### `Plugins.get(name)`
Get a specific plugin by name.

**Parameters:**
- `name` (str): Plugin name (e.g., "vieux/sshfs:latest")

**Returns:** `Plugin` - Plugin instance

```python
plugins = docker.plugins()
plugin = plugins.get("vieux/sshfs:latest")
```

#### `Plugins.list()`
List all installed plugins.

**Returns:** `list[dict]` - List of plugin information

```python
plugins_list = docker.plugins().list()
for p in plugins_list:
    print(f"{p['Name']}: {'enabled' if p['Enabled'] else 'disabled'}")
```

#### `Plugins.list_by_capability(capability)`
List plugins filtered by capability.

**Parameters:**
- `capability` (str): Capability filter (e.g., "volumedriver", "networkdriver")

**Returns:** `list[dict]` - List of matching plugins

```python
volume_plugins = docker.plugins().list_by_capability("volumedriver")
```

#### `Plugin.name()`
Get the plugin name.

**Returns:** `str` - Plugin name

#### `Plugin.inspect()`
Inspect the plugin for detailed information.

**Returns:** `dict` - Plugin details including settings, config, enabled state

```python
info = plugin.inspect()
print(f"Enabled: {info['Enabled']}")
```

#### `Plugin.enable(timeout=None)`
Enable the plugin.

**Parameters:**
- `timeout` (int, optional): Timeout in seconds

**Returns:** `None`

```python
plugin.enable()
```

#### `Plugin.disable()`
Disable the plugin.

**Returns:** `None`

```python
plugin.disable()
```

#### `Plugin.remove()`
Remove the plugin (must be disabled first).

**Returns:** `dict` - Information about removed plugin

```python
plugin.disable()
plugin.remove()
```

#### `Plugin.force_remove()`
Forcefully remove the plugin (even if enabled).

**Returns:** `dict` - Information about removed plugin

#### `Plugin.push()`
Push the plugin to a registry.

**Returns:** `None`

#### `Plugin.create(path)`
Create a plugin from a tar archive.

**Parameters:**
- `path` (str): Path to tar archive with rootfs and config.json

**Returns:** `None`

---

### Swarm Mode Operations

These operations require Docker to be running in Swarm mode.

#### Nodes

Interface for managing Swarm nodes.

##### `Nodes.get(id)`
Get a specific node by ID or name.

**Parameters:**
- `id` (str): Node ID or name

**Returns:** `Node` - Node instance

##### `Nodes.list()`
List all nodes in the swarm.

**Returns:** `list[dict]` - List of node information

```python
nodes = docker.nodes().list()
for node in nodes:
    print(f"{node['ID']}: {node['Status']['State']}")
```

##### `Node.id()`
Get the node ID.

**Returns:** `str` - Node ID

##### `Node.inspect()`
Inspect the node for detailed information.

**Returns:** `dict` - Node details including status, spec, description

```python
info = node.inspect()
print(f"Role: {info['Spec']['Role']}")
print(f"Availability: {info['Spec']['Availability']}")
```

##### `Node.delete()`
Delete the node from the swarm.

**Returns:** `None`

##### `Node.force_delete()`
Force delete the node from the swarm.

**Returns:** `None`

##### `Node.update(version, name=None, role=None, availability=None, labels=None)`
Update node configuration.

**Parameters:**
- `version` (str): Node version (from inspect)
- `name` (str, optional): Node name
- `role` (str, optional): Role ("worker" or "manager")
- `availability` (str, optional): Availability ("active", "pause", or "drain")
- `labels` (dict, optional): Node labels

```python
info = node.inspect()
version = str(info['Version']['Index'])
node.update(version, availability="drain", labels={"env": "production"})
```

#### Services

Interface for managing Swarm services.

##### `Services.get(id)`
Get a specific service by ID or name.

**Parameters:**
- `id` (str): Service ID or name

**Returns:** `Service` - Service instance

##### `Services.list()`
List all services in the swarm.

**Returns:** `list[dict]` - List of service information

```python
services = docker.services().list()
for svc in services:
    print(f"{svc['Spec']['Name']}: {svc['Spec']['Mode']}")
```

##### `Service.id()`
Get the service ID.

**Returns:** `str` - Service ID

##### `Service.inspect()`
Inspect the service for detailed information.

**Returns:** `dict` - Service details including spec, endpoint, update status

##### `Service.delete()`
Delete the service from the swarm.

**Returns:** `None`

##### `Service.logs(stdout=None, stderr=None, timestamps=None, n_lines=None, all=None, since=None)`
Get service logs.

**Parameters:**
- `stdout` (bool, optional): Include stdout
- `stderr` (bool, optional): Include stderr
- `timestamps` (bool, optional): Include timestamps
- `n_lines` (int, optional): Number of lines from end
- `all` (bool, optional): Return all logs
- `since` (datetime, optional): Only logs since this time

**Returns:** `str` - Service logs

#### Secrets

Interface for managing Swarm secrets.

##### `Secrets.get(id)`
Get a specific secret by ID or name.

**Parameters:**
- `id` (str): Secret ID or name

**Returns:** `Secret` - Secret instance

##### `Secrets.list()`
List all secrets in the swarm.

**Returns:** `list[dict]` - List of secret information

##### `Secrets.create(name, data, labels=None)`
Create a new secret.

**Parameters:**
- `name` (str): Secret name
- `data` (str): Secret data (base64 encoded automatically)
- `labels` (dict, optional): Labels

**Returns:** `Secret` - Created secret instance

```python
secret = docker.secrets().create(
    name="db_password",
    data="super_secret_123",
    labels={"app": "myapp"}
)
```

##### `Secret.id()`
Get the secret ID.

**Returns:** `str` - Secret ID

##### `Secret.inspect()`
Inspect the secret (data not returned for security).

**Returns:** `dict` - Secret metadata

##### `Secret.delete()`
Delete the secret.

**Returns:** `None`

#### Configs

Interface for managing Swarm configs (non-sensitive configuration data).

##### `Configs.get(id)`
Get a specific config by ID or name.

**Parameters:**
- `id` (str): Config ID or name

**Returns:** `Config` - Config instance

##### `Configs.list()`
List all configs in the swarm.

**Returns:** `list[dict]` - List of config information

##### `Configs.create(name, data, labels=None)`
Create a new config.

**Parameters:**
- `name` (str): Config name
- `data` (str): Config data (base64 encoded automatically)
- `labels` (dict, optional): Labels

**Returns:** `Config` - Created config instance

```python
config = docker.configs().create(
    name="nginx_config",
    data="server { listen 80; }",
    labels={"app": "web"}
)
```

##### `Config.id()`
Get the config ID.

**Returns:** `str` - Config ID

##### `Config.inspect()`
Inspect the config.

**Returns:** `dict` - Config details

##### `Config.delete()`
Delete the config.

**Returns:** `None`

#### Tasks

Interface for managing Swarm tasks (container instances of services).

##### `Tasks.get(id)`
Get a specific task by ID.

**Parameters:**
- `id` (str): Task ID

**Returns:** `Task` - Task instance

##### `Tasks.list()`
List all tasks in the swarm.

**Returns:** `list[dict]` - List of task information

```python
tasks = docker.tasks().list()
for task in tasks:
    print(f"{task['ID']}: {task['Status']['State']}")
```

##### `Task.id()`
Get the task ID.

**Returns:** `str` - Task ID

##### `Task.inspect()`
Inspect the task for detailed information.

**Returns:** `dict` - Task details including status, spec, assigned node

##### `Task.logs(stdout=None, stderr=None, timestamps=None, n_lines=None, all=None, since=None)`
Get task logs.

**Parameters:**
- Same as Service.logs()

**Returns:** `str` - Task logs

---

## Complete Examples

### Building and Running a Web Application

```python
from docker_pyo3 import Docker

docker = Docker()

# Build the application image
docker.images().build(
    path="/path/to/app",
    dockerfile="Dockerfile",
    tag="myapp:latest",
    labels={"version": "1.0", "env": "production"}
)

# Create a custom network
network = docker.networks().create(
    name="app-network",
    driver="bridge"
)

# Create a volume for data persistence
docker.volumes().create(
    name="app-data",
    labels={"app": "myapp"}
)

# Create and start the application container
app_container = docker.containers().create(
    image="myapp:latest",
    name="myapp-instance",
    env=["ENV=production", "PORT=8080"],
    expose=[{"srcport": 8080, "hostport": 8080, "protocol": "tcp"}],
    volumes=["app-data:/data:rw"],
    labels={"app": "myapp", "tier": "web"},
    restart_policy={"name": "on-failure", "maximum_retry_count": 3}
)

# Connect to the custom network
network.connect("myapp-instance")

# Start the container
app_container.start()

# Check logs
logs = app_container.logs(stdout=True, stderr=True, n_lines=50)
print(logs)

# Inspect running container
info = app_container.inspect()
print(f"Container IP: {info['NetworkSettings']['Networks']['app-network']['IPAddress']}")

# Stop and cleanup
app_container.stop()
app_container.delete()
network.disconnect("myapp-instance")
network.delete()
```

### Managing Multiple Containers

```python
from docker_pyo3 import Docker

docker = Docker()

# Pull required images
docker.images().pull(image="nginx:latest")
docker.images().pull(image="redis:latest")

# Create a custom network for the application
network = docker.networks().create(name="app-tier")

# Create Redis container
redis = docker.containers().create(
    image="redis:latest",
    name="redis",
    labels={"tier": "cache"}
)
redis.start()
network.connect("redis", aliases=["cache"])

# Create Nginx container linked to Redis
nginx = docker.containers().create(
    image="nginx:latest",
    name="web",
    expose=[{"srcport": 80, "hostport": 8080, "protocol": "tcp"}],
    labels={"tier": "web"},
    links=["redis:cache"]
)
nginx.start()
network.connect("web")

# List all running containers
containers = docker.containers().list()
for container in containers:
    print(f"{container['Names'][0]}: {container['Status']}")

# Cleanup
for container_name in ["web", "redis"]:
    c = docker.containers().get(container_name)
    c.stop()
    c.delete()

network.delete()
```

---

## Why does this exist?

Python already has the `docker` package, so why create another one?

This library is designed specifically for **Rust projects that expose Python as a plugin interface**. If you:
- Just need Docker in Python → Use `pip install docker`
- Just need Docker in Rust → Use the `docker_api` crate
- Need to add a Python interface to containers in a Rust library/binary via `pyo3` → This library provides ready-to-use bindings

## Embedding in Rust Projects

You can embed `docker-pyo3` in your Rust application using PyO3. Here's an example:

```rust
use pyo3::prelude::*;
use pyo3::wrap_pymodule;

#[pymodule]
fn root_module(_py: Python, m: &PyModule) -> PyResult<()> {
    // Register your custom functionality
    m.add_function(wrap_pyfunction!(main, m)?)?;

    // Add docker-pyo3 as a submodule
    m.add_wrapped(wrap_pymodule!(_integrations))?;

    // Register submodules in sys.modules for proper imports
    let sys = PyModule::import(_py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("root_module._integrations", m.getattr("_integrations")?)?;
    sys_modules.set_item("root_module._integrations.docker", m.getattr("_integrations")?.getattr("docker")?)?;

    Ok(())
}

#[pymodule]
fn _integrations(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(docker))?;
    Ok(())
}

#[pymodule]
fn docker(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<docker_pyo3::Pyo3Docker>()?;
    m.add_wrapped(wrap_pymodule!(docker_pyo3::image::image))?;
    m.add_wrapped(wrap_pymodule!(docker_pyo3::container::container))?;
    m.add_wrapped(wrap_pymodule!(docker_pyo3::network::network))?;
    m.add_wrapped(wrap_pymodule!(docker_pyo3::volume::volume))?;
    Ok(())
}
```

This creates the following Python namespace structure:
- `root_module._integrations.docker.Docker`
- `root_module._integrations.docker.image.Images`, `Image`
- `root_module._integrations.docker.container.Containers`, `Container`
- `root_module._integrations.docker.network.Networks`, `Network`
- `root_module._integrations.docker.volume.Volumes`, `Volume`

## License

GPL-3.0-only

## Contributing

Contributions are welcome! Please see the test suite in `py_test/` for examples of the full API in action.
