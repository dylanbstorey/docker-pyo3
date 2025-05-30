import angreal
import os
import subprocess

venv_path = os.path.join(angreal.get_root(),'..','.venv')

cwd = os.path.join(angreal.get_root(),'..')

@angreal.command(name="test", about="run all tests for the docker-pyo3 library")
@angreal.argument(name="coverage", short="c", long="coverage", help="run tests with coverage report", takes_value=False)
@angreal.argument(name="html", short="H", long="html", help="generate HTML coverage report", takes_value=False)
@angreal.argument(name="parallel", short="p", long="parallel", help="run tests in parallel (number of workers or 'auto')", takes_value=True)
@angreal.argument(name="filter", short="k", long="filter", help="filter tests by pattern (passed to pytest -k)", takes_value=True)
def run_tests(coverage=False, html=False, parallel=None, filter=None):
    """
    Run all tests for the docker-pyo3 library
    """
    result = subprocess.run(
        ["pip install ."], cwd=cwd, shell=True
    )
    if result.returncode != 0:
        exit(result.returncode)

    # Build pytest command
    pytest_cmd = "pytest -svv py_test/"
    
    # Add parallel execution
    if parallel:
        if parallel.lower() == "auto":
            pytest_cmd += " -n auto"
        else:
            pytest_cmd += f" -n {parallel}"
    
    # Add test filtering
    if filter:
        pytest_cmd += f" -k {filter}"
    
    if coverage:
        # Add coverage options
        pytest_cmd += " --cov=docker_pyo3 --cov-report=term-missing"
        if html:
            pytest_cmd += " --cov-report=html"
    
    subprocess.run([pytest_cmd], cwd=cwd, shell=True)