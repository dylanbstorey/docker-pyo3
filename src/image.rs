use std::fs::OpenOptions;

use crate::{get_runtime, Pyo3Docker};
use crate::error::DockerPyo3Error;
use docker_api::models::{
    ImageDeleteResponseItem, ImageHistory200Response, ImageInspect, ImagePrune200Response,
    ImageSummary,
};
use docker_api::opts::{
    ImageBuildOpts, ImageListOpts, ImagePushOpts, PullOpts, RegistryAuth, TagOpts,
};

use docker_api::{Image, Images};
use futures_util::StreamExt;
use pyo3::exceptions::{self, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::pythonize;
use std::{collections::HashMap, io::Write};

#[pymodule]
pub fn image(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pyo3Images>()?;
    m.add_class::<Pyo3Image>()?;
    Ok(())
}

#[derive(Debug)]
#[pyclass(name = "Images")]
pub struct Pyo3Images(pub Images);

#[derive(Debug)]
#[pyclass(name = "Image")]
pub struct Pyo3Image(pub Image);

#[pymethods]
impl Pyo3Images {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Images(Images::new(docker.0))
    }

    fn get(&self, name: &str) -> Pyo3Image {
        Pyo3Image(self.0.get(name))
    }

    fn list(
        &self,
        all: Option<bool>,
        digests: Option<bool>,
        _filter: Option<&str>,
    ) -> PyResult<Py<PyAny>> {
        let mut opts = ImageListOpts::builder();
        bo_setter!(all, opts);
        bo_setter!(digests, opts);

        let rv = __images_list(&self.0, &opts.build());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn prune(&self) -> PyResult<Py<PyAny>> {
        match __images_prune(&self.0) {
            Ok(info) => Ok(pythonize_this!(info)),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn build(
        &self,
        path: &str,
        dockerfile: Option<&str>,
        tag: Option<&str>,
        extra_hosts: Option<&str>,
        remote: Option<&str>,
        quiet: Option<bool>,
        nocahe: Option<bool>,
        pull: Option<&str>,
        rm: Option<bool>,
        forcerm: Option<bool>,
        memory: Option<usize>,
        memswap: Option<usize>,
        cpu_shares: Option<usize>,
        cpu_set_cpus: Option<&str>,
        cpu_period: Option<usize>,
        cpu_quota: Option<usize>,
        shm_size: Option<usize>,
        squash: Option<bool>,
        network_mode: Option<&str>,
        platform: Option<&str>,
        target: Option<&str>,
        outputs: Option<&str>,
        labels: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        let mut bo = ImageBuildOpts::builder(path);

        let labels: Option<HashMap<&str, &str>> = if labels.is_some() {
            Some(labels.unwrap().extract().map_err(|_| {
                DockerPyo3Error::InvalidParameter(
                    "Labels must be a dictionary with string keys and values".to_string()
                )
            })?)
        } else {
            None
        };

        bo_setter!(dockerfile, bo);
        bo_setter!(tag, bo);
        bo_setter!(extra_hosts, bo);
        bo_setter!(remote, bo);
        bo_setter!(quiet, bo);
        bo_setter!(nocahe, bo);
        bo_setter!(pull, bo);
        bo_setter!(rm, bo);
        bo_setter!(forcerm, bo);
        bo_setter!(memory, bo);
        bo_setter!(memswap, bo);
        bo_setter!(cpu_shares, bo);
        bo_setter!(cpu_set_cpus, bo);
        bo_setter!(cpu_period, bo);
        bo_setter!(cpu_quota, bo);
        bo_setter!(shm_size, bo);
        bo_setter!(squash, bo);
        bo_setter!(network_mode, bo);
        bo_setter!(platform, bo);
        bo_setter!(target, bo);
        bo_setter!(outputs, bo);
        bo_setter!(labels, bo);

        let rv = __images_build(&self.0, &bo.build());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    fn search(&self, _term: &str, _limit: Option<u32>) -> PyResult<Py<PyAny>> {
        // TODO: Docker registry search is not available in docker-api crate
        Err(exceptions::PyNotImplementedError::new_err(
            "Image search not available in docker-api crate - use registry API directly",
        ))
    }

    fn pull(
        &self,
        image: Option<&str>,
        src: Option<&str>,
        repo: Option<&str>,
        tag: Option<&str>,
        auth_password: Option<&PyDict>,
        auth_token: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        let mut pull_opts = PullOpts::builder();

        if auth_password.is_some() && auth_token.is_some() {
            let msg = "Got both auth_password and auth_token for image.push(). Only one of these options is allowed";
            return Err(DockerPyo3Error::InvalidParameter(msg.to_string()).into());
        }

        let auth = if auth_password.is_some() && auth_token.is_none() {
            let username = auth_password.unwrap().get_item("username");
            let password = auth_password.unwrap().get_item("password");
            let email = auth_password.unwrap().get_item("email");
            let server_address = auth_password.unwrap().get_item("server_address");

            let username = if username.is_none() {
                None
            } else {
                Some(username.unwrap().extract::<String>().unwrap())
            };

            let password = if password.is_none() {
                None
            } else {
                Some(password.unwrap().extract::<String>().unwrap())
            };

            let email = if email.is_none() {
                None
            } else {
                Some(email.unwrap().extract::<String>().unwrap())
            };

            let server_address = if server_address.is_none() {
                None
            } else {
                Some(server_address.unwrap().extract::<String>().unwrap())
            };

            let mut ra = RegistryAuth::builder();

            bo_setter!(username, ra);
            bo_setter!(password, ra);
            bo_setter!(email, ra);
            bo_setter!(server_address, ra);

            Some(ra.build())
        } else if auth_token.is_some() && auth_password.is_none() {
            let auth_dict = auth_token.unwrap();
            match auth_dict.get_item("identity_token") {
                Some(token_obj) => {
                    match token_obj.extract::<String>() {
                        Ok(token_str) if !token_str.is_empty() => {
                            let token = RegistryAuth::token(token_str);
                            Some(token)
                        }
                        _ => {
                            return Err(PyValueError::new_err("auth_token must contain a non-empty 'identity_token' field"));
                        }
                    }
                }
                None => {
                    return Err(PyValueError::new_err("auth_token must contain an 'identity_token' field"));
                }
            }
        } else {
            Some(RegistryAuth::builder().build())
        };

        bo_setter!(src, pull_opts);
        bo_setter!(repo, pull_opts);
        bo_setter!(tag, pull_opts);
        bo_setter!(image, pull_opts);
        bo_setter!(auth, pull_opts);

        let rv = __images_pull(&self.0, &pull_opts.build());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn export(&self, names: Vec<&str>, output: &str) -> PyResult<()> {
        // Export multiple images to a tar archive
        let rv = __images_export(&self.0, names, output);
        match rv {
            Ok(_) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn import(&self, src: &str, repository: Option<&str>, tag: Option<&str>) -> PyResult<Py<PyAny>> {
        let rv = __images_import(&self.0, src, repository, tag);
        match rv {
            Ok(result) => Ok(pythonize_this!(result)),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }

    fn push(&self, _repository: &str, _tag: Option<&str>) -> PyResult<()> {
        // NOTE: Push should be done on individual Image objects, not the Images collection
        // Use docker.images().get("image_name").push() instead
        Err(exceptions::PyNotImplementedError::new_err(
            "Use individual image.push() method instead of images collection push",
        ))
    }

    fn clear_cache(&self) -> PyResult<()> {
        // Docker builder cache clear - this is typically done via Docker buildx
        // For now, we'll implement basic prune which clears unused images
        let rv = __images_prune(&self.0);
        match rv {
            Ok(_) => Ok(()),
            Err(e) => Err(DockerPyo3Error::from(e).into()),
        }
    }
}

fn __images_list(
    images: &Images,
    opts: &ImageListOpts,
) -> Result<Vec<ImageSummary>, docker_api::Error> {
    get_runtime().block_on(images.list(opts))
}

fn __images_prune(images: &Images) -> Result<ImagePrune200Response, docker_api::Error> {
    get_runtime().block_on(images.prune(&Default::default()))
}

fn __images_build(
    images: &Images,
    opts: &ImageBuildOpts,
) -> Result<Vec<String>, docker_api::Error> {
    get_runtime().block_on(async {
        let mut stream = images.build(opts);
        let mut ok_stream_vec = Vec::new();
        let mut err_message = None;
        while let Some(build_result) = stream.next().await {
            match build_result {
                Ok(output) => ok_stream_vec.push(format!("{output:?}")),
                Err(e) => err_message = Some(e),
            }
        }

        match err_message {
            Some(err_message) => Err(err_message),
            _ => Ok(ok_stream_vec),
        }
    })
}

fn __images_pull(
    images: &Images,
    pull_opts: &PullOpts,
) -> Result<Vec<String>, docker_api::Error> {
    get_runtime().block_on(async {
        let mut stream = images.pull(pull_opts);
        let mut ok_stream_vec = Vec::new();
        let mut err_message = None;
        while let Some(pull_result) = stream.next().await {
            match pull_result {
                Ok(output) => ok_stream_vec.push(format!("{output:?}")),
                Err(e) => err_message = Some(e),
            }
        }

        match err_message {
            Some(err_message) => Err(err_message),
            _ => Ok(ok_stream_vec),
        }
    })
}

#[pymethods]
impl Pyo3Image {
    #[new]
    fn new(docker: Pyo3Docker, name: &str) -> Pyo3Image {
        Pyo3Image(Image::new(docker.0, name))
    }

    fn __repr__(&self) -> String {
        match __image_inspect(&self.0) {
            Ok(inspect) => {
                let id = inspect.id.unwrap_or_else(|| "unknown".to_string());
                format!("Image(id: {}, name: {})", id, self.name())
            },
            Err(_) => format!("Image(name: {}, status: unavailable)", self.name())
        }
    }

    fn __string__(&self) -> String {
        self.__repr__()
    }

    fn name(&self) -> Py<PyAny> {
        let rv = self.0.name();
        pythonize_this!(rv)
    }

    fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __image_inspect(&self.0);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    fn remove(&self) -> PyResult<String> {
        let rv = __image_delete(&self.0);
        match rv {
            Ok(rv) => {
                let mut r_value = "".to_owned();
                for r in rv {
                    let r_str = format!("{r:?}");
                    r_value.push_str(&r_str);
                }
                Ok(r_value)
            }
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    fn delete(&self) -> PyResult<String> {
        let rv = __image_delete(&self.0);
        match rv {
            Ok(rv) => {
                let mut r_value = "".to_owned();
                for r in rv {
                    let r_str = format!("{r:?}");
                    r_value.push_str(&r_str);
                }
                Ok(r_value)
            }
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    fn history(&self) -> PyResult<String> {
        let rv = __image_history(&self.0);

        match rv {
            Ok(rv) => {
                let mut r_value = "".to_owned();
                for r in rv {
                    let r_str = format!("{r:?}");
                    r_value.push_str(&r_str);
                }
                Ok(r_value)
            }
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    fn export(&self, path: Option<&str>) -> PyResult<String> {
        let path = if path.is_none() {
            format!("{:?}", &self.0)
        } else {
            path.unwrap().to_string()
        };

        let rv = __image_export(&self.0, path);

        if rv.is_some() {
            match rv.unwrap() {
                Ok(n) => Ok(n),
                Err(e) => Err(DockerPyo3Error::from(e).into()),
            }
        } else {
            Err(DockerPyo3Error::NotSupported("Unknown error occurred in export".to_string()).into())
        }
    }

    fn tag(&self, repo: Option<&str>, tag: Option<&str>) -> PyResult<()> {
        let mut opts = TagOpts::builder();

        bo_setter!(repo, opts);
        bo_setter!(tag, opts);

        let rv = __image_tag(&self.0, &opts.build());

        match rv {
            Ok(_rv) => Ok(()),
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    fn push(
        &self,
        auth_password: Option<&PyDict>,
        auth_token: Option<&PyDict>,
        tag: Option<&str>,
    ) -> PyResult<()> {
        if auth_password.is_some() && auth_token.is_some() {
            let msg = "Got both auth_password and auth_token for image.push(). Only one of these options is allowed";
            return Err(DockerPyo3Error::InvalidParameter(msg.to_string()).into());
        }

        let auth = if auth_password.is_some() && auth_token.is_none() {
            let username = auth_password.unwrap().get_item("username");
            let password = auth_password.unwrap().get_item("password");
            let email = auth_password.unwrap().get_item("email");
            let server_address = auth_password.unwrap().get_item("server_address");

            let username = if username.is_none() {
                None
            } else {
                Some(username.unwrap().extract::<String>().unwrap())
            };

            let password = if password.is_none() {
                None
            } else {
                Some(password.unwrap().extract::<String>().unwrap())
            };

            let email = if email.is_none() {
                None
            } else {
                Some(email.unwrap().extract::<String>().unwrap())
            };

            let server_address = if server_address.is_none() {
                None
            } else {
                Some(server_address.unwrap().extract::<String>().unwrap())
            };

            let mut ra = RegistryAuth::builder();

            bo_setter!(username, ra);
            bo_setter!(password, ra);
            bo_setter!(email, ra);
            bo_setter!(server_address, ra);

            Some(ra.build())
        } else if auth_token.is_some() && auth_password.is_none() {
            let auth_dict = auth_token.unwrap();
            match auth_dict.get_item("identity_token") {
                Some(token_obj) => {
                    match token_obj.extract::<String>() {
                        Ok(token_str) if !token_str.is_empty() => {
                            let token = RegistryAuth::token(token_str);
                            Some(token)
                        }
                        _ => {
                            return Err(PyValueError::new_err("auth_token must contain a non-empty 'identity_token' field"));
                        }
                    }
                }
                None => {
                    return Err(PyValueError::new_err("auth_token must contain an 'identity_token' field"));
                }
            }
        } else {
            Some(RegistryAuth::builder().build())
        };

        let mut opts = ImagePushOpts::builder();
        bo_setter!(tag, opts);
        bo_setter!(auth, opts);

        let rv = __image_push(&self.0, &opts.build());
        match rv {
            Ok(_rv) => Ok(()),
            Err(rv) => Err(DockerPyo3Error::from(rv).into()),
        }
    }

    fn distribution_inspect(&self) -> PyResult<()> {
        Err(exceptions::PyNotImplementedError::new_err(
            "This method is not available yet.",
        ))
    }
}

fn __image_inspect(image: &Image) -> Result<ImageInspect, docker_api::Error> {
    get_runtime().block_on(image.inspect())
}

fn __image_delete(image: &Image) -> Result<Vec<ImageDeleteResponseItem>, docker_api::Error> {
    get_runtime().block_on(image.delete())
}

fn __image_history(image: &Image) -> Result<ImageHistory200Response, docker_api::Error> {
    get_runtime().block_on(image.history())
}

fn __image_export(image: &Image, path: String) -> Option<Result<String, docker_api::Error>> {
    get_runtime().block_on(async {
        let mut export_file = match OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path) {
                Ok(file) => file,
                Err(e) => return Some(Err(docker_api::Error::from(e))),
            };

        let mut export_stream = image.export();
        let mut total_bytes = 0;
        
        while let Some(chunk_result) = export_stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    match export_file.write(&bytes) {
                        Ok(written) => total_bytes += written,
                        Err(e) => return Some(Err(docker_api::Error::from(e))),
                    }
                },
                Err(e) => return Some(Err(e)),
            }
        }
        
        // Flush the file
        if let Err(e) = export_file.flush() {
            return Some(Err(docker_api::Error::from(e)));
        }
        
        Some(Ok(format!("Exported {} bytes", total_bytes)))
    })
}

fn __image_tag(image: &Image, opts: &TagOpts) -> Result<(), docker_api::Error> {
    get_runtime().block_on(image.tag(opts))
}

fn __image_push(image: &Image, opts: &ImagePushOpts) -> Result<(), docker_api::Error> {
    // For now, just call the original push method
    // TODO: Properly handle streaming push responses and authentication errors
    get_runtime().block_on(image.push(opts))
}


fn __images_export(
    _images: &Images,
    _names: Vec<&str>,
    _output: &str
) -> Result<(), docker_api::Error> {
    // TODO: Implement multi-image export
    // This requires streaming multiple images to a tar file
    use std::io;
    Err(docker_api::Error::from(io::Error::new(
        io::ErrorKind::Other, 
        "Image export not yet implemented"
    )))
}

fn __images_import(
    _images: &Images,
    _src: &str,
    _repository: Option<&str>,
    _tag: Option<&str>
) -> Result<String, docker_api::Error> {
    // TODO: Implement image import from tar/url
    use std::io;
    Err(docker_api::Error::from(io::Error::new(
        io::ErrorKind::Other, 
        "Image import not yet implemented"
    )))
}
