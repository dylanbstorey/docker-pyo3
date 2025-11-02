import angreal
from angreal.integrations.venv import VirtualEnv
import subprocess
import os
import glob

@angreal.command(name="run-tests", about="run our test suite")
def run_tests():
    venv_path = os.path.join(angreal.get_root(), "..", ".venv")

    with VirtualEnv(venv_path, now=True) as venv:
        # Install required packages
        venv.install("maturin")
        venv.install("pytest")

        # Build the wheel with maturin using venv's python
        subprocess.run([venv.python_executable, "-m", "maturin", "build"], check=True)

        # Find the built wheel - sort by modification time to get the latest
        wheels = glob.glob("target/wheels/docker_pyo3-*.whl")
        if not wheels:
            raise RuntimeError("No wheel file found after build")

        # Sort by modification time, newest first
        latest_wheel = sorted(wheels, key=os.path.getmtime, reverse=True)[0]

        # Install the built wheel directly (force reinstall to get latest build)
        subprocess.run([venv.python_executable, "-m", "pip", "install", "--force-reinstall", latest_wheel], check=True)

        # Run pytest using venv's python
        pytest_rv = subprocess.run([venv.python_executable, "-m", "pytest", "-svv"])

        if pytest_rv.returncode:
            raise RuntimeError(
                f"Tests failed with status code: {pytest_rv.returncode} (pytest)"
            )

