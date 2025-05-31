# docker-pyo3

Python bindings for the Rust `docker_api` crate, providing high-performance Docker operations without subprocess overhead.

## Features

- **Container Management**: Complete lifecycle management with advanced parameters
- **Stack Orchestration**: Multi-container application deployment with service scaling
- **Image Operations**: Pull, build, push, and manage Docker images with registry authentication
- **Network Management**: Create and manage Docker networks with isolation and connectivity
- **Volume Management**: Persistent data storage and sharing between containers
- **Health Monitoring**: Real-time container health status and resource monitoring
- **No Subprocess Overhead**: Direct Docker API communication through Rust bindings

## Quick Start

```python
from docker_pyo3 import Docker

# Initialize Docker client
docker = Docker()

# Container operations
container = docker.containers().create(
    image="nginx:latest",
    name="my-nginx",
    ports={"80": "8080"},
    env=["ENV=production"]
)
container.start()

# Stack orchestration
stack = docker.create_stack("myapp")
web_service = docker.create_service("web")
web_service.image("nginx:latest")
web_service.ports(["80:8080"])
stack.register_service(web_service)
stack.up()

# Service scaling
stack.scale("web", 3)
status = stack.status()  # Get detailed health information

# Image management
image = docker.images().pull("python:3.11")
docker.images().build(path=".", tag="myapp:latest")
```

## Re-exporting in Your Own Package

docker-pyo3 can be integrated into other Rust-based Python packages and re-exported under custom namespaces:

```rust
use pyo3::prelude::*;

#[pymodule]
fn my_package(py: Python, m: &PyModule) -> PyResult<()> {
    // Re-export all docker-pyo3 functionality under your namespace
    docker_pyo3::register_module(py, m, "my_package")?;
    
    // Add your own additional functionality
    Ok(())
}
```

Python usage:
```python
from my_package import Docker  # Same API, different namespace
docker = Docker()
```

See [docs/reexporting.md](docs/reexporting.md) for detailed integration examples.

## Documentation

**📚 [Complete Documentation](docs/README.md)**

- [Container Management](docs/containers.md) - Lifecycle, configuration, monitoring
- [Stack Orchestration](docs/stacks.md) - Multi-container deployments, scaling
- [Service Definition](docs/services.md) - Fluent API, templates, reusable components
- [Image Management](docs/images.md) - Building, pushing, registry operations
- [Network Management](docs/networks.md) - Networking, isolation, connectivity
- [Volume Management](docs/volumes.md) - Data persistence, backup, sharing

## Installation

```bash
pip install docker-pyo3
```

## Examples

See the [`py_test`](py_test/) folder for comprehensive examples and test cases.


## Python has `docker` already, why does this exist ?

Good question. In short, because this is meant to be built into rust projects that expose python as a plugin interface. If you just need docker in python, use `pip install docker`, if you just need 
docker in rust use the `docker_api` crate. If you need to add a python interface to containers to a rust library/binary via `pyo3`- this will get you most of the way. 

## Cool how do i do that ?

See the below example. But basically just follow the instructions in `pyo3` to register a module and set the package state. This creates the following namespaces
and classes within them
- `root_module._integrations.docker`, `Docker`
- `root_module._integrations.image`, `Image` `Images`
- `root_module._integrations.container`, `Container` `Containers`
- `root_module._integrations.network`, `Network` `Networks`
- `root_module._integrations.volume`, `Volume` `Volumes`

```python
#[pymodule]
fn root_module(_py: Python, m: &PyModule) -> PyResult<()> {
    py_logger::register();
    m.add_function(wrap_pyfunction!(main, m)?)?;
    task::register(_py, m)?;
    utils::register(_py, m)?;

    
    m.add_wrapped(wrap_pymodule!(_integrations))?;

    let sys = PyModule::import(_py, "sys")?;
    let sys_modules: &PyDict = sys.getattr("modules")?.downcast()?;
    sys_modules.set_item("root_module._integrations", m.getattr("_integrations")?)?;
    sys_modules.set_item("root_module._integrations.docker", m.getattr("_integrations")?.getattr("docker")?)?;

    sys_modules.set_item("root_module._integrations.docker.image", m.getattr("_integrations")?.getattr("docker")?.getattr("image")?)?;
    sys_modules.set_item("root_module._integrations.docker.container", m.getattr("_integrations")?.getattr("docker")?.getattr("container")?)?;
    sys_modules.set_item("root_module._integrations.docker.network", m.getattr("_integrations")?.getattr("docker")?.getattr("network")?)?;
    sys_modules.set_item("root_module._integrations.docker.volume", m.getattr("_integrations")?.getattr("docker")?.getattr("volume")?)?;
    Ok(())
}

#[pymodule]
fn _integrations(_py: Python, m:&PyModule) -> PyResult<()>{
    m.add_wrapped(wrap_pymodule!(docker))?;
    Ok(())
}

#[pymodule]
fn docker(_py: Python, m:&PyModule) -> PyResult<()>{
    m.add_class::<docker_pyo3::Pyo3Docker>()?;
    m.add_wrapped(wrap_pymodule!(docker_pyo3::image::image))?;
    m.add_wrapped(wrap_pymodule!(docker_pyo3::container::container))?;
    m.add_wrapped(wrap_pymodule!(docker_pyo3::network::network))?;
    m.add_wrapped(wrap_pymodule!(docker_pyo3::volume::volume))?;
    Ok(())
}
```


