macro_rules! bo_setter {
    ($a:ident, $o:ident) => {
        if $a.is_some() {
            $o = $o.$a($a.unwrap());
        }
    };
}

macro_rules! pythonize_this {
    ($o:ident) => {{
        Python::attach(|py| -> Py<PyAny> { pythonize(py, &$o).unwrap().into() })
    }};
}

macro_rules! py_sys_exception {
    ($o:ident) => {
        exceptions::PySystemError::new_err(format!("{}", $o))
    };
}
