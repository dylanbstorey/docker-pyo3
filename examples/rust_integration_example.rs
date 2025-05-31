// Example showing how to use docker-pyo3 in another Rust crate
// This would be in a separate crate that depends on docker-pyo3

use pyo3::prelude::*;

/// Example: Basic re-export under custom namespace
#[pymodule]
fn my_ops_tools(py: Python, m: &PyModule) -> PyResult<()> {
    // Re-export docker-pyo3 functionality
    docker_pyo3::register_module(py, m, "my_ops_tools")?;
    
    // Add your own custom functionality
    #[pyfunction]
    fn get_docker_info() -> String {
        "Docker integration provided by my_ops_tools".to_string()
    }
    
    m.add_function(wrap_pyfunction!(get_docker_info, m)?)?;
    Ok(())
}

/// Example: Re-export as a submodule
#[pymodule] 
fn angreal_integrations(py: Python, m: &PyModule) -> PyResult<()> {
    // Create docker submodule
    let docker_module = PyModule::new(py, "docker")?;
    docker_pyo3::register_module(py, docker_module, "angreal_integrations.docker")?;
    m.add_submodule(docker_module)?;
    
    // Could add other integrations
    // let k8s_module = PyModule::new(py, "kubernetes")?;
    // m.add_submodule(k8s_module)?;
    
    Ok(())
}

/// Example: Wrapper with additional functionality
#[pymodule]
fn enhanced_docker(py: Python, m: &PyModule) -> PyResult<()> {
    // First register all docker-pyo3 functionality
    docker_pyo3::register_module(py, m, "enhanced_docker")?;
    
    // Add convenience functions
    #[pyfunction]
    fn quick_run(image: String, command: Vec<String>) -> PyResult<String> {
        Python::with_gil(|py| {
            // This is pseudo-code - actual implementation would use docker-pyo3 types
            let docker = py.import("enhanced_docker")?.getattr("Docker")?.call0()?;
            let containers = docker.call_method0("containers")?;
            let container = containers.call_method(
                "create",
                (),
                Some([("image", image), ("command", command)].into_py_dict(py))
            )?;
            container.call_method0("start")?;
            let logs = container.call_method0("logs")?;
            container.call_method0("remove")?;
            logs.extract::<String>()
        })
    }
    
    #[pyfunction]
    fn cleanup_all() -> PyResult<()> {
        Python::with_gil(|py| {
            let docker = py.import("enhanced_docker")?.getattr("Docker")?.call0()?;
            
            // Clean up containers
            let containers = docker.call_method0("containers")?;
            let all_containers = containers.call_method("list", (true,), None)?;
            for container in all_containers.iter()? {
                let container = container?;
                container.call_method0("stop")?;
                container.call_method0("remove")?;
            }
            
            // Prune system
            containers.call_method0("prune")?;
            
            Ok(())
        })
    }
    
    m.add_function(wrap_pyfunction!(quick_run, m)?)?;
    m.add_function(wrap_pyfunction!(cleanup_all, m)?)?;
    
    Ok(())
}

/// Example: Custom configuration wrapper
#[pyclass]
struct ManagedDocker {
    docker: Py<PyAny>,
    auto_cleanup: bool,
}

#[pymethods]
impl ManagedDocker {
    #[new]
    fn new(uri: Option<String>, auto_cleanup: Option<bool>) -> PyResult<Self> {
        Python::with_gil(|py| {
            let docker_module = py.import("enterprise_docker")?;
            let docker_class = docker_module.getattr("Docker")?;
            let docker = match uri {
                Some(uri) => docker_class.call1((uri,))?,
                None => docker_class.call0()?,
            };
            
            Ok(ManagedDocker {
                docker: docker.into(),
                auto_cleanup: auto_cleanup.unwrap_or(true),
            })
        })
    }
    
    fn get_docker(&self) -> Py<PyAny> {
        self.docker.clone()
    }
    
    fn __enter__(&mut self) -> PyResult<Py<PyAny>> {
        Ok(self.docker.clone())
    }
    
    fn __exit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_val: Option<&PyAny>,
        _exc_tb: Option<&PyAny>,
    ) -> PyResult<bool> {
        if self.auto_cleanup {
            Python::with_gil(|py| {
                // Cleanup logic here
                println!("Auto-cleaning up Docker resources...");
            });
        }
        Ok(false) // Don't suppress exceptions
    }
}

#[pymodule]
fn enterprise_docker(py: Python, m: &PyModule) -> PyResult<()> {
    // Register docker-pyo3
    docker_pyo3::register_module(py, m, "enterprise_docker")?;
    
    // Add managed wrapper
    m.add_class::<ManagedDocker>()?;
    
    // Add configuration constants
    m.add("DEFAULT_TIMEOUT", 30)?;
    m.add("MAX_RETRIES", 3)?;
    
    Ok(())
}

// Example usage from Python after building any of the above modules:
/*

# Basic re-export
from my_ops_tools import Docker, Container, Image
docker = Docker()
print(my_ops_tools.get_docker_info())

# Submodule approach  
from angreal_integrations.docker import Docker
docker = Docker()

# Enhanced version
from enhanced_docker import Docker, quick_run, cleanup_all
output = quick_run("alpine", ["echo", "Hello from Docker!"])
cleanup_all()

# Enterprise version with context manager
from enterprise_docker import ManagedDocker
with ManagedDocker(auto_cleanup=True) as docker:
    containers = docker.containers()
    # ... do work ...
# Auto cleanup happens here

*/