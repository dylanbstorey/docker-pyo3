use std::collections::HashMap;
use std::fs::{File, OpenOptions};

use crate::Pyo3Docker;
use docker_api::models::{
    BuildPrune200Response, ImageDeleteResponseItem, ImageHistory200Response, ImageInspect,
    ImagePrune200Response, ImageSearch200Response,
};
use docker_api::opts::{
    ClearCacheOpts, ImageBuildOpts, ImageFilter, ImageListOpts, ImageName, ImagePushOpts, PullOpts,
    RegistryAuth, TagOpts,
};

use docker_api::{Image, Images};
use futures_util::StreamExt;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::pythonize;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Compatible ImageSummary that handles Docker API v1.44+ which removed VirtualSize.
/// This struct mirrors docker_api::models::ImageSummary but makes virtual_size optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImageSummaryCompat {
    pub id: String,
    pub parent_id: String,
    pub repo_tags: Option<Vec<String>>,
    pub repo_digests: Option<Vec<String>>,
    pub created: i64,
    pub size: i64,
    pub shared_size: i64,
    #[serde(default)]
    pub virtual_size: Option<i64>,
    pub labels: Option<HashMap<String, String>>,
    pub containers: i64,
}

#[pymodule]
pub fn image(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pyo3Images>()?;
    m.add_class::<Pyo3Image>()?;
    Ok(())
}

/// Interface for managing Docker images collection.
#[derive(Debug)]
#[pyclass(name = "Images")]
pub struct Pyo3Images(pub Images);

/// Represents an individual Docker image.
#[derive(Debug)]
#[pyclass(name = "Image")]
pub struct Pyo3Image(pub Image);

#[pymethods]
impl Pyo3Images {
    #[new]
    pub fn new(docker: Pyo3Docker) -> Self {
        Pyo3Images(Images::new(docker.0))
    }

    /// Get a specific image by name, ID, or tag.
    ///
    /// Args:
    ///     name: Image name, ID, or tag (e.g., "busybox", "busybox:latest")
    ///
    /// Returns:
    ///     Image: Image instance
    fn get(&self, name: &str) -> Pyo3Image {
        Pyo3Image(self.0.get(name))
    }

    /// List images.
    ///
    /// Args:
    ///     all: Show all images (default hides intermediate images)
    ///     digests: Show digests
    ///     filter: Filter images by dict with type and value:
    ///             - {"type": "dangling"}: dangling images
    ///             - {"type": "label", "key": "foo", "value": "bar"}: by label
    ///             - {"type": "before", "value": "image:tag"}: images before specified
    ///             - {"type": "since", "value": "image:tag"}: images since specified
    ///
    /// Returns:
    ///     list[dict]: List of image information dictionaries
    #[pyo3(signature = (all=None, digests=None, filter=None))]
    fn list(
        &self,
        all: Option<bool>,
        digests: Option<bool>,
        filter: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let mut opts = ImageListOpts::builder();
        bo_setter!(all, opts);
        bo_setter!(digests, opts);

        // Handle filter parameter - expects dict like {"type": "dangling", "value": true}
        // or {"type": "label", "key": "foo", "value": "bar"}
        if let Some(filter_dict) = filter {
            if let Some(filter_type) = filter_dict.get_item("type")? {
                let filter_type_str: String = filter_type.extract()?;

                let image_filter = match filter_type_str.as_str() {
                    "dangling" => ImageFilter::Dangling,
                    "label" => {
                        if let Some(value) = filter_dict.get_item("value")? {
                            if let Some(key) = filter_dict.get_item("key")? {
                                ImageFilter::Label(key.extract()?, value.extract()?)
                            } else {
                                ImageFilter::LabelKey(value.extract()?)
                            }
                        } else {
                            return Err(exceptions::PyValueError::new_err(
                                "label filter requires 'value' (and optionally 'key')",
                            ));
                        }
                    }
                    "before" => {
                        if let Some(value) = filter_dict.get_item("value")? {
                            let image_str: String = value.extract()?;
                            ImageFilter::Before(ImageName::tag(image_str, None::<String>))
                        } else {
                            return Err(exceptions::PyValueError::new_err(
                                "before filter requires 'value'",
                            ));
                        }
                    }
                    "since" => {
                        if let Some(value) = filter_dict.get_item("value")? {
                            let image_str: String = value.extract()?;
                            ImageFilter::Since(ImageName::tag(image_str, None::<String>))
                        } else {
                            return Err(exceptions::PyValueError::new_err(
                                "since filter requires 'value'",
                            ));
                        }
                    }
                    _ => {
                        return Err(exceptions::PyValueError::new_err(format!(
                            "unknown filter type: {}",
                            filter_type_str
                        )))
                    }
                };

                opts = opts.filter([image_filter]);
            }
        }

        // Use docker CLI to list images, working around docker-api's VirtualSize issue
        // Docker API v1.44+ removed VirtualSize field, breaking upstream ImageSummary struct
        let rv = __images_list_via_cli(all.unwrap_or(false));

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(exceptions::PySystemError::new_err(rv)),
        }
    }

    /// Remove unused images.
    ///
    /// Returns:
    ///     dict: Prune results including images deleted and space reclaimed
    fn prune(&self) -> PyResult<Py<PyAny>> {
        match __images_prune(&self.0) {
            Ok(info) => Ok(pythonize_this!(info)),
            Err(e) => Err(exceptions::PySystemError::new_err(format!("{e:?}"))),
        }
    }

    /// Build an image from a Dockerfile.
    ///
    /// Args:
    ///     path: Path to build context directory
    ///     dockerfile: Path to Dockerfile relative to build context
    ///     tag: Tag for the built image (e.g., "myimage:latest")
    ///     extra_hosts: Extra hosts to add to /etc/hosts
    ///     remote: Remote repository URL
    ///     quiet: Suppress build output
    ///     nocahe: Do not use cache when building
    ///     pull: Attempt to pull newer version of base image
    ///     rm: Remove intermediate containers after build
    ///     forcerm: Always remove intermediate containers
    ///     memory: Memory limit in bytes
    ///     memswap: Total memory limit (memory + swap)
    ///     cpu_shares: CPU shares (relative weight)
    ///     cpu_set_cpus: CPUs to allow execution (e.g., "0-3", "0,1")
    ///     cpu_period: CPU CFS period in microseconds
    ///     cpu_quota: CPU CFS quota in microseconds
    ///     shm_size: Size of /dev/shm in bytes
    ///     squash: Squash newly built layers into single layer
    ///     network_mode: Network mode (e.g., "bridge", "host", "none")
    ///     platform: Target platform (e.g., "linux/amd64")
    ///     target: Build stage to target
    ///     outputs: Output configuration
    ///     labels: Labels as dict (e.g., {"version": "1.0"})
    ///
    /// Returns:
    ///     dict: Build result information
    #[pyo3(signature = (path, *, dockerfile=None, tag=None, extra_hosts=None, remote=None, quiet=None, nocahe=None, pull=None, rm=None, forcerm=None, memory=None, memswap=None, cpu_shares=None, cpu_set_cpus=None, cpu_period=None, cpu_quota=None, shm_size=None, squash=None, network_mode=None, platform=None, target=None, outputs=None, labels=None))]
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
        labels: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let mut bo = ImageBuildOpts::builder(path);

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

        let labels_map: Option<HashMap<String, String>> = if labels.is_some() {
            Some(labels.unwrap().extract().unwrap())
        } else {
            None
        };
        let labels: Option<HashMap<&str, &str>> = labels_map
            .as_ref()
            .map(|m| m.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect());

        bo_setter!(labels, bo);

        let rv = __images_build(&self.0, &bo.build());

        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Search for images on Docker Hub.
    ///
    /// Args:
    ///     term: Search term (e.g., "nginx", "python")
    ///
    /// Returns:
    ///     dict: Search results including image names, descriptions, and star counts
    ///
    /// Raises:
    ///     SystemError: If search fails
    fn search(&self, term: &str) -> PyResult<Py<PyAny>> {
        let rv = __images_search(&self.0, term);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Pull an image from a registry.
    ///
    /// Args:
    ///     image: Image name to pull (e.g., "busybox", "ubuntu:latest")
    ///     src: Source repository
    ///     repo: Repository to pull from
    ///     tag: Tag to pull
    ///     auth_password: Password authentication dict with username, password, email, server_address
    ///     auth_token: Token authentication dict with identity_token
    ///
    /// Returns:
    ///     dict: Pull result information
    ///
    /// Raises:
    ///     SystemError: If both auth_password and auth_token are provided
    #[pyo3(signature = (image=None, src=None, repo=None, tag=None, auth_password=None, auth_token=None))]
    fn pull(
        &self,
        image: Option<&str>,
        src: Option<&str>,
        repo: Option<&str>,
        tag: Option<&str>,
        auth_password: Option<&Bound<'_, PyDict>>,
        auth_token: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let mut pull_opts = PullOpts::builder();

        if auth_password.is_some() && auth_token.is_some() {
            let msg = "Got both auth_password and auth_token for image.push(). Only one of these options is allowed";
            return Err(py_sys_exception!(msg));
        }

        let auth = if auth_password.is_some() && auth_token.is_none() {
            let auth_dict = auth_password.unwrap();
            let username = auth_dict.get_item("username").unwrap_or(None);
            let password = auth_dict.get_item("password").unwrap_or(None);
            let email = auth_dict.get_item("email").unwrap_or(None);
            let server_address = auth_dict.get_item("server_address").unwrap_or(None);

            let username = username.map(|v| v.extract::<String>().unwrap());
            let password = password.map(|v| v.extract::<String>().unwrap());
            let email = email.map(|v| v.extract::<String>().unwrap());
            let server_address = server_address.map(|v| v.extract::<String>().unwrap());

            let mut ra = RegistryAuth::builder();

            bo_setter!(username, ra);
            bo_setter!(password, ra);
            bo_setter!(email, ra);
            bo_setter!(server_address, ra);

            Some(ra.build())
        } else if auth_token.is_some() && auth_password.is_none() {
            let token = RegistryAuth::token(
                auth_token
                    .unwrap()
                    .get_item("identity_token")
                    .unwrap_or(None)
                    .expect("identity_token is required")
                    .extract::<String>()
                    .unwrap(),
            );
            Some(token)
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
            Err(rv) => Err(exceptions::PySystemError::new_err(format!("{rv}"))),
        }
    }

    /// Import images from a tarball file.
    ///
    /// This loads images that were previously exported using `docker save` or `image.export()`.
    ///
    /// Args:
    ///     path: Path to the tarball file to import
    ///
    /// Returns:
    ///     list[str]: Import result messages
    ///
    /// Raises:
    ///     SystemError: If import fails
    fn import_image(&self, path: &str) -> PyResult<Py<PyAny>> {
        let rv = __images_import(&self.0, path);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Clear the build cache.
    ///
    /// Args:
    ///     all: Remove all unused build cache, not just dangling ones
    ///     keep_storage: Amount of disk space to keep for cache (in bytes)
    ///
    /// Returns:
    ///     dict: Cache prune results including space reclaimed
    ///
    /// Raises:
    ///     SystemError: If cache clear fails
    #[pyo3(signature = (all=None, keep_storage=None))]
    fn clear_cache(&self, all: Option<bool>, keep_storage: Option<i64>) -> PyResult<Py<PyAny>> {
        let mut opts = ClearCacheOpts::builder();
        bo_setter!(all, opts);
        bo_setter!(keep_storage, opts);

        let rv = __images_clear_cache(&self.0, &opts.build());
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }
}

/// List images using docker CLI to handle Docker API v1.44+ compatibility.
/// Docker API v1.44+ removed the VirtualSize field, breaking the upstream ImageSummary struct.
/// This function calls `docker images` and parses JSON output with optional virtual_size.
fn __images_list_via_cli(all: bool) -> Result<Vec<ImageSummaryCompat>, String> {
    use std::process::Command;

    let mut cmd = Command::new("docker");
    cmd.args(["images", "--format", "json", "--no-trunc"]);
    if all {
        cmd.arg("--all");
    }

    let output = cmd.output().map_err(|e| format!("Failed to execute docker: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "docker images failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Docker outputs one JSON object per line (NDJSON format)
    let mut images = Vec::new();
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // Parse the docker CLI JSON format and convert to our struct
        let cli_image: DockerCliImage =
            serde_json::from_str(line).map_err(|e| format!("Failed to parse JSON: {}", e))?;
        images.push(cli_image.into());
    }

    Ok(images)
}

/// Docker CLI image format (from `docker images --format json`)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DockerCliImage {
    #[serde(rename = "ID")]
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub digest: String,
    pub created_since: String,
    pub created_at: String,
    pub size: String,
    #[serde(default)]
    pub virtual_size: Option<String>,
    pub shared_size: String,
    pub unique_size: String,
    pub containers: String,
}

impl From<DockerCliImage> for ImageSummaryCompat {
    fn from(cli: DockerCliImage) -> Self {
        // Parse size strings to bytes (e.g., "1.23GB" -> bytes)
        fn parse_size(s: &str) -> i64 {
            let s = s.trim();
            if s == "N/A" || s.is_empty() {
                return 0;
            }
            let (num, unit) = s.split_at(s.len().saturating_sub(2));
            let num: f64 = num.parse().unwrap_or(0.0);
            match unit.to_uppercase().as_str() {
                "KB" => (num * 1024.0) as i64,
                "MB" => (num * 1024.0 * 1024.0) as i64,
                "GB" => (num * 1024.0 * 1024.0 * 1024.0) as i64,
                "TB" => (num * 1024.0 * 1024.0 * 1024.0 * 1024.0) as i64,
                _ => {
                    // Try parsing as plain bytes
                    let clean: String = s.chars().filter(|c| c.is_numeric()).collect();
                    clean.parse().unwrap_or(0)
                }
            }
        }

        let repo_tag = if cli.repository != "<none>" && cli.tag != "<none>" {
            Some(vec![format!("{}:{}", cli.repository, cli.tag)])
        } else {
            None
        };

        let repo_digest = if cli.digest != "<none>" {
            Some(vec![format!("{}@{}", cli.repository, cli.digest)])
        } else {
            None
        };

        ImageSummaryCompat {
            id: cli.id,
            parent_id: String::new(), // CLI doesn't provide parent ID
            repo_tags: repo_tag,
            repo_digests: repo_digest,
            created: 0, // CLI provides human-readable, not timestamp
            size: parse_size(&cli.size),
            shared_size: parse_size(&cli.shared_size),
            virtual_size: cli.virtual_size.map(|s| parse_size(&s)),
            labels: None, // CLI doesn't provide labels in default format
            containers: cli.containers.parse().unwrap_or(0),
        }
    }
}

#[tokio::main]
async fn __images_prune(images: &Images) -> Result<ImagePrune200Response, docker_api::Error> {
    images.prune(&Default::default()).await
}

#[tokio::main]
async fn __images_build(
    images: &Images,
    opts: &ImageBuildOpts,
) -> Result<Vec<String>, docker_api::Error> {
    use futures_util::StreamExt;
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
}

#[tokio::main]
async fn __images_pull(
    images: &Images,
    pull_opts: &PullOpts,
) -> Result<Vec<String>, docker_api::Error> {
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
}

#[tokio::main]
async fn __images_search(
    images: &Images,
    term: &str,
) -> Result<ImageSearch200Response, docker_api::Error> {
    images.search(term).await
}

#[tokio::main]
async fn __images_import(images: &Images, path: &str) -> Result<Vec<String>, docker_api::Error> {
    let file = File::open(path).map_err(|e| docker_api::Error::Any(Box::new(e)))?;

    let mut stream = images.import(file);
    let mut ok_stream_vec = Vec::new();
    let mut err_message = None;
    while let Some(import_result) = stream.next().await {
        match import_result {
            Ok(output) => ok_stream_vec.push(format!("{output:?}")),
            Err(e) => err_message = Some(e),
        }
    }

    match err_message {
        Some(err_message) => Err(err_message),
        _ => Ok(ok_stream_vec),
    }
}

#[tokio::main]
async fn __images_clear_cache(
    images: &Images,
    opts: &ClearCacheOpts,
) -> Result<BuildPrune200Response, docker_api::Error> {
    images.clear_cache(opts).await
}

#[pymethods]
impl Pyo3Image {
    #[new]
    fn new(docker: Pyo3Docker, name: &str) -> Pyo3Image {
        Pyo3Image(Image::new(docker.0, name))
    }

    fn __repr__(&self) -> String {
        let inspect = __image_inspect(&self.0).unwrap();
        format!(
            "Image(id: {:?}, name: {})",
            inspect.id.unwrap(),
            self.0.name()
        )
    }

    fn __string__(&self) -> String {
        self.__repr__()
    }

    /// Get the image name.
    ///
    /// Returns:
    ///     str: Image name
    fn name(&self) -> Py<PyAny> {
        let rv = self.0.name();
        pythonize_this!(rv)
    }

    /// Inspect the image to get detailed information.
    ///
    /// Returns:
    ///     dict: Detailed image information including config, layers, etc.
    fn inspect(&self) -> PyResult<Py<PyAny>> {
        let rv = __image_inspect(&self.0);
        match rv {
            Ok(rv) => Ok(pythonize_this!(rv)),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Remove the image (not implemented yet).
    fn remove(&self) -> PyResult<()> {
        Err(exceptions::PyNotImplementedError::new_err(
            "This method is not available yet.",
        ))
    }

    /// Delete the image.
    ///
    /// Returns:
    ///     str: Deletion result information
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
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Get the image history.
    ///
    /// Returns:
    ///     str: Image history information
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
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Export the image to a tar file.
    ///
    /// Args:
    ///     path: Path to save the exported tar file
    ///
    /// Returns:
    ///     str: Path to the exported file
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
                Err(e) => Err(py_sys_exception!(e)),
            }
        } else {
            Err(exceptions::PySystemError::new_err("Unknown error occurred in export. (Seriously I don't know how you get here, open a ticket and tell me what happens)"))
        }
    }

    /// Tag the image with a new name and/or tag.
    ///
    /// Args:
    ///     repo: Repository name (e.g., "myrepo/myimage")
    ///     tag: Tag name (e.g., "v1.0", "latest")
    ///
    /// Returns:
    ///     None
    #[pyo3(signature = (repo=None, tag=None))]
    fn tag(&self, repo: Option<&str>, tag: Option<&str>) -> PyResult<()> {
        let mut opts = TagOpts::builder();

        bo_setter!(repo, opts);
        bo_setter!(tag, opts);

        let rv = __image_tag(&self.0, &opts.build());

        match rv {
            Ok(_rv) => Ok(()),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    /// Push the image to a registry.
    ///
    /// Args:
    ///     auth_password: Password authentication dict with username, password, email, server_address
    ///     auth_token: Token authentication dict with identity_token
    ///     tag: Tag to push
    ///
    /// Returns:
    ///     None
    ///
    /// Raises:
    ///     SystemError: If both auth_password and auth_token are provided
    fn push(
        &self,
        auth_password: Option<&Bound<'_, PyDict>>,
        auth_token: Option<&Bound<'_, PyDict>>,
        tag: Option<&str>,
    ) -> PyResult<()> {
        if auth_password.is_some() && auth_token.is_some() {
            let msg = "Got both auth_password and auth_token for image.push(). Only one of these options is allowed";
            return Err(py_sys_exception!(msg));
        }

        let auth = if auth_password.is_some() && auth_token.is_none() {
            let auth_dict = auth_password.unwrap();
            let username = auth_dict.get_item("username").unwrap_or(None);
            let password = auth_dict.get_item("password").unwrap_or(None);
            let email = auth_dict.get_item("email").unwrap_or(None);
            let server_address = auth_dict.get_item("server_address").unwrap_or(None);

            let username = username.map(|v| v.extract::<String>().unwrap());
            let password = password.map(|v| v.extract::<String>().unwrap());
            let email = email.map(|v| v.extract::<String>().unwrap());
            let server_address = server_address.map(|v| v.extract::<String>().unwrap());

            let mut ra = RegistryAuth::builder();

            bo_setter!(username, ra);
            bo_setter!(password, ra);
            bo_setter!(email, ra);
            bo_setter!(server_address, ra);

            Some(ra.build())
        } else if auth_token.is_some() && auth_password.is_none() {
            let token = RegistryAuth::token(
                auth_token
                    .unwrap()
                    .get_item("identity_token")
                    .unwrap_or(None)
                    .expect("identity_token is required")
                    .extract::<String>()
                    .unwrap(),
            );
            Some(token)
        } else {
            Some(RegistryAuth::builder().build())
        };

        let mut opts = ImagePushOpts::builder();
        bo_setter!(tag, opts);
        bo_setter!(auth, opts);

        let rv = __image_push(&self.0, &opts.build());
        match rv {
            Ok(_rv) => Ok(()),
            Err(rv) => Err(py_sys_exception!(rv)),
        }
    }

    fn distribution_inspect(&self) -> PyResult<()> {
        Err(exceptions::PyNotImplementedError::new_err(
            "This method is not available yet.",
        ))
    }
}

#[tokio::main]
async fn __image_inspect(image: &Image) -> Result<ImageInspect, docker_api::Error> {
    image.inspect().await
}

#[tokio::main]
async fn __image_delete(image: &Image) -> Result<Vec<ImageDeleteResponseItem>, docker_api::Error> {
    image.delete().await
}

#[tokio::main]
async fn __image_history(image: &Image) -> Result<ImageHistory200Response, docker_api::Error> {
    image.history().await
}

#[tokio::main]
async fn __image_export(image: &Image, path: String) -> Option<Result<String, docker_api::Error>> {
    let mut export_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .unwrap();

    let rv = image.export().next().await;

    match rv {
        None => None,
        Some(_rv) => match _rv {
            Ok(bytes) => {
                let w_rv = export_file.write(&bytes).unwrap();
                Some(Ok(format!("{w_rv:?}")))
            }
            Err(_rv) => Some(Err(_rv)),
        },
    }
}

#[tokio::main]
async fn __image_tag(image: &Image, opts: &TagOpts) -> Result<(), docker_api::Error> {
    image.tag(opts).await
}

#[tokio::main]
async fn __image_push(image: &Image, opts: &ImagePushOpts) -> Result<(), docker_api::Error> {
    image.push(opts).await
}
