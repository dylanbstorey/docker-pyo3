import angreal
from angreal.integrations.venv import venv_required, VirtualEnv
import subprocess
import os

@venv_required(os.path.join(angreal.get_root(),"..",".venv"))
@angreal.command(name="run-tests", about="run our test suite")
def run_tests():
    # venv = VirtualEnv(os.path.join(angreal.get_root(),"..",".venv"))
    # venv._activate()
    # cargo_rv = subprocess.run(
    #     [
    #         "cargo",
    #         "test",
    #         "-v",
    #         "--",
    #         "--nocapture",
    #         "--test-threads=1",
    #     ]
    # )
    subprocess.run(["python", "-m", "pip", "install", "maturin","pytest"])
    subprocess.run(["maturin","build"])
    subprocess.run(["python", "-m", "pip", "install", "."])
    pytest_rv = subprocess.run(["python", "-m", "pytest", "-svv"])

    if pytest_rv.returncode:
        raise RuntimeError(
            f"Tests failed with status codes : {cargo_rv} (cargo) and {pytest_rv}(pytest)"
        )

