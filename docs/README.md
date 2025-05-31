# Docker-PyO3 Documentation

Docker-PyO3 provides Python bindings for Docker operations without subprocess overhead, designed for embedding in Rust applications with Python bindings.

## Quick Start

```python
from docker_pyo3 import Docker

# Initialize Docker client
docker = Docker()

# Basic container operations
container = docker.containers().create(
    image="nginx:latest",
    name="my-nginx",
    ports={"80": "8080"}
)
container.start()

# Stack orchestration
stack = docker.create_stack("myapp")
web_service = docker.create_service("web")
web_service.image("nginx:latest")
web_service.ports(["80:8080"])
stack.register_service(web_service)
stack.up()
```

## Core Features

### [Container Management](containers.md)
- Create, start, stop, and manage containers
- Execute commands and access logs
- Copy files to/from containers
- Advanced container parameters

### [Stack Orchestration](stacks.md)
- Multi-container application deployment
- Service scaling and health monitoring
- Centralized logging and status reporting

### [Service Definition](services.md)
- Fluent API for defining services
- Reusable service templates
- Environment variables and volumes

### [Image Management](images.md)
- Pull, build, and push images
- Image inspection and tagging
- Registry authentication

### [Network Management](networks.md)
- Create and manage networks
- Connect containers to networks
- Network inspection and cleanup

### [Volume Management](volumes.md)
- Create and manage volumes
- Volume mounting and persistence
- Volume inspection and cleanup

### [Re-exporting](reexporting.md)
- Integrate docker-pyo3 into your own Python packages
- Custom namespace registration
- Extension patterns and examples

## Examples

### [Basic Examples](examples/basic.md)
- Simple container operations
- Image building and management

### [Stack Examples](examples/stacks.md)
- Multi-service applications
- Service scaling scenarios
- Health monitoring

### [Advanced Examples](examples/advanced.md)
- Custom networking
- Volume management
- Registry operations

## API Reference

### [Docker Client](api/docker.md)
### [Containers](api/containers.md)
### [Stacks](api/stacks.md)
### [Services](api/services.md)
### [Images](api/images.md)
### [Networks](api/networks.md)
### [Volumes](api/volumes.md)