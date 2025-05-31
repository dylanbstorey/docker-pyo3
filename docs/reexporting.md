# Re-exporting docker-pyo3 in Your Own Python Package

This guide explains how to integrate docker-pyo3 into your own Rust-based Python package and re-export it under your own namespace.

## Overview

docker-pyo3 provides a `register_module()` function that allows other Rust crates to register all docker-pyo3 functionality under a custom namespace. This is useful when you want to:

- Bundle docker-pyo3 as part of a larger Python package
- Provide Docker functionality under your own module hierarchy
- Avoid namespace conflicts with other packages

## Basic Usage

### 1. Add docker-pyo3 as a Dependency

In your `Cargo.toml`:

```toml
[dependencies]
docker-pyo3 = "0.1"  # Use the appropriate version
pyo3 = "0.20"  # Match the PyO3 version used by docker-pyo3
```

### 2. Create Your Python Module

In your Rust code:

```rust
use pyo3::prelude::*;

#[pymodule]
fn my_package(py: Python, m: &PyModule) -> PyResult<()> {
    // Create a submodule for docker functionality
    let docker_module = PyModule::new(py, "docker")?;
    
    // Register all docker-pyo3 functionality
    docker_pyo3::register_module(py, docker_module, "my_package.docker")?;
    
    // Add the submodule to your package
    m.add_submodule(docker_module)?;
    
    // Add your own functionality
    // m.add_class::<MyClass>()?;
    // m.add_function(wrap_pyfunction!(my_function, m)?)?;
    
    Ok(())
}
```

### 3. Use in Python

After building your package, users can import it as:

```python
from my_package.docker import Docker

# All docker-pyo3 functionality is now available under your namespace
docker = Docker()
containers = docker.containers()
```

## Advanced Example: Integration Package

Here's a more complete example showing how to create an integration package:

```rust
use pyo3::prelude::*;

/// Your custom configuration class
#[pyclass]
struct DockerConfig {
    uri: String,
    timeout: u64,
}

#[pymethods]
impl DockerConfig {
    #[new]
    fn new(uri: Option<String>, timeout: Option<u64>) -> Self {
        DockerConfig {
            uri: uri.unwrap_or_else(|| "unix:///var/run/docker.sock".to_string()),
            timeout: timeout.unwrap_or(30),
        }
    }
}

/// Custom Docker factory that uses your configuration
#[pyfunction]
fn create_docker_client(config: &DockerConfig) -> PyResult<docker_pyo3::Pyo3Docker> {
    // You can add custom logic here
    Python::with_gil(|py| {
        let docker_class = py.import("docker_pyo3")?.getattr("Docker")?;
        let docker = docker_class.call1((config.uri.clone(),))?;
        docker.extract::<docker_pyo3::Pyo3Docker>()
    })
}

#[pymodule]
fn angreal_integrations(py: Python, m: &PyModule) -> PyResult<()> {
    // Create a docker submodule
    let docker_submodule = PyModule::new(py, "docker")?;
    
    // Register all docker-pyo3 functionality
    docker_pyo3::register_module(py, docker_submodule, "angreal_integrations.docker")?;
    
    // Add your custom classes and functions
    docker_submodule.add_class::<DockerConfig>()?;
    docker_submodule.add_function(wrap_pyfunction!(create_docker_client, docker_submodule)?)?;
    
    // Add the submodule
    m.add_submodule(docker_submodule)?;
    
    Ok(())
}
```

Usage in Python:

```python
from angreal_integrations.docker import Docker, DockerConfig, create_docker_client

# Use the standard Docker class
docker = Docker()

# Or use your custom factory
config = DockerConfig(timeout=60)
docker = create_docker_client(config)

# All functionality works the same
containers = docker.containers()
images = docker.images()
```

## Multiple Submodules Example

You can also organize docker-pyo3 alongside other integrations:

```rust
#[pymodule]
fn my_devtools(py: Python, m: &PyModule) -> PyResult<()> {
    // Docker integration
    let docker_mod = PyModule::new(py, "docker")?;
    docker_pyo3::register_module(py, docker_mod, "my_devtools.docker")?;
    m.add_submodule(docker_mod)?;
    
    // Kubernetes integration (hypothetical)
    let k8s_mod = PyModule::new(py, "kubernetes")?;
    // kubernetes_pyo3::register_module(py, k8s_mod, "my_devtools.kubernetes")?;
    m.add_submodule(k8s_mod)?;
    
    // AWS integration (hypothetical)
    let aws_mod = PyModule::new(py, "aws")?;
    // aws_pyo3::register_module(py, aws_mod, "my_devtools.aws")?;
    m.add_submodule(aws_mod)?;
    
    Ok(())
}
```

Python usage:

```python
from my_devtools.docker import Docker
from my_devtools.kubernetes import KubeClient
from my_devtools.aws import S3Client

# All integrations under one namespace
docker = Docker()
kube = KubeClient()
s3 = S3Client()
```

## Important Notes

1. **Version Compatibility**: Ensure your PyO3 version matches the one used by docker-pyo3 to avoid ABI compatibility issues.

2. **Module Naming**: The `module_name` parameter in `register_module()` should match the full Python import path to avoid issues with submodule imports.

3. **Re-exporting Types**: If you need to re-export specific types (like `Pyo3Docker`), you may need to add docker-pyo3 types to your crate's public API:
   ```rust
   pub use docker_pyo3::{Pyo3Docker, Pyo3Container, Pyo3Stack};
   ```

4. **Error Handling**: docker-pyo3 errors will be automatically converted to Python exceptions. You can wrap them in your own error types if needed.

## Testing Your Integration

```python
import pytest
from my_package.docker import Docker

def test_docker_integration():
    # Test that the Docker class is available
    docker = Docker()
    assert docker is not None
    
    # Test that methods are available
    assert hasattr(docker, 'containers')
    assert hasattr(docker, 'images')
    assert hasattr(docker, 'networks')
    assert hasattr(docker, 'volumes')
    
    # Test submodule imports work
    from my_package.docker.container import Container
    from my_package.docker.image import Image
```

## Complete Example Project Structure

```
my-integration-package/
├── Cargo.toml
├── pyproject.toml
├── src/
│   └── lib.rs
├── python/
│   └── my_package/
│       └── __init__.py
└── tests/
    └── test_docker.py
```

This approach allows you to seamlessly integrate docker-pyo3 into your own Python packages while maintaining full control over the namespace and adding your own functionality.