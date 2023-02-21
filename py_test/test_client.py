from docker_pyo3 import Docker


def test_client_init():
    """ client has expected methods&attrs"""
    d = Docker()
    assert isinstance(d,Docker)

    interface_methods = [
        "containers",
        "images",
        "networks",
        "volumes"
    ]

    for m in interface_methods:
        assert hasattr(d,m) and callable(getattr(d,m))

    gettr_methods = [
        "version",
        "info",
        "ping",
        "data_usage"
    ]

    for gm in gettr_methods:
        assert isinstance(getattr(d,gm)(),dict)

