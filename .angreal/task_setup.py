import angreal
from angreal.integrations.venv import venv_required, VirtualEnv
import subprocess
import os


@angreal.command(name="setup", about="run our test suite")
def run_tests():
    venv = VirtualEnv(os.path.join(angreal.get_root(),"..",".venv"))
    venv._create()
    venv._activate()
    subprocess.run(["python", "-m", "pip", "install", "maturin","pytest"])
    

