# docker-pyo3

Python bindings the the rust `docker_api` crate.


## Basic Usage
`pip install docker_pyo3`

```python

from docker_pyo3 import Docker

# Connecto the daemon
docker = Docker()

# pull an image
docker.images().pull(image='busybox')

# build an image
docker.images().build(path="path/to/dockerfile",dockerfile='Dockerfile',tag='test-image')

# run a container
c = docker.containers().create(image='busybox',name='weee')
```

Full api examples can be seen in the `py_test` folder.


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


