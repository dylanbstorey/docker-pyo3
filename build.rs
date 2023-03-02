/// when using this crate as a dependency in another pyo3 based package on a mac - the linker args don't propagate
/// correctly - so we are explicit about this build arg for inheritence sakes.
fn main() {
    pyo3_build_config::add_extension_module_link_args();
}
