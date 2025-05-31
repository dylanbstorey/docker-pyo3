import angreal
import os
import subprocess

venv_path = os.path.join(angreal.get_root(),'..','.venv')
cwd = os.path.join(angreal.get_root(),'..')

@angreal.command(name="test-manual", about="run the manual stdout/stderr separation test")
def run_manual_test():
    """
    Run the manual stdout/stderr separation test for docker-pyo3
    
    This test verifies that Docker log stream separation works correctly.
    You need to manually verify the output meets expectations.
    """
    print("🔍 Running manual stdout/stderr separation test...")
    print("📋 What to look for in the output:")
    print("   - STDOUT ONLY should contain 'To stdout' but NOT 'To stderr'")
    print("   - STDERR ONLY should contain 'To stderr' but NOT 'To stdout'")
    print("   - BOTH STREAMS should contain both 'To stdout' AND 'To stderr'")
    print("   - Note: Docker's stream multiplexing behavior may vary\n")
    
    # Install the package first
    print("📦 Installing docker-pyo3...")
    result = subprocess.run(
        ["pip install ."], cwd=cwd, shell=True
    )
    if result.returncode != 0:
        print("❌ Failed to install docker-pyo3")
        exit(result.returncode)
    
    # Run the specific manual test (first uncomment the skip decorator)
    print("🧪 Running the manual test...")
    print("Note: The test is currently marked as SKIPPED for automated runs.")
    print("To run it manually, you need to:")
    print("1. Edit py_test/test_streaming_operations.py")
    print("2. Comment out the @pytest.mark.skip line (line 91)")
    print("3. Run: python -m pytest py_test/test_streaming_operations.py::TestLogStreaming::test_separate_stdout_stderr -xvs\n")
    
    pytest_cmd = "python -m pytest py_test/test_streaming_operations.py::TestLogStreaming::test_separate_stdout_stderr -xvs"
    
    print(f"📝 Command to run manually:\n   {pytest_cmd}\n")
    print("💡 Or uncomment the skip decorator and run:")
    print("   angreal test-manual-run")

@angreal.command(name="test-manual-run", about="run the manual test with skip disabled")
def run_manual_test_directly():
    """
    Run the manual test directly (assumes skip decorator is commented out)
    """
    # Install the package first
    result = subprocess.run(
        ["pip install ."], cwd=cwd, shell=True
    )
    if result.returncode != 0:
        exit(result.returncode)
    
    pytest_cmd = "python -m pytest py_test/test_streaming_operations.py::TestLogStreaming::test_separate_stdout_stderr -xvs -s"
    result = subprocess.run([pytest_cmd], cwd=cwd, shell=True)
    
    if result.returncode == 0:
        print("\n✅ Manual test completed successfully!")
        print("🔍 Please verify the output above matches expected behavior:")
        print("   - STDOUT ONLY contains 'To stdout' but not 'To stderr'")
        print("   - STDERR ONLY contains 'To stderr' but not 'To stdout'")
        print("   - BOTH STREAMS contains both messages")
    else:
        print(f"\n❌ Manual test failed with exit code {result.returncode}")