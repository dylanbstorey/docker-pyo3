#!/usr/bin/env python3
"""
Basic Container Operations Example

Demonstrates fundamental container lifecycle management with docker-pyo3.
"""

from docker_pyo3 import Docker
import time

def main():
    # Initialize Docker client
    docker = Docker()
    
    print("🐳 Docker-PyO3 Basic Container Operations Example")
    print("=" * 50)
    
    # Pull an image
    print("📦 Pulling nginx image...")
    try:
        docker.images().pull("nginx:alpine")
        print("✅ Image pulled successfully")
    except Exception as e:
        print(f"❌ Failed to pull image: {e}")
        return
    
    # Create a container
    print("\n🏗️  Creating container...")
    try:
        container = docker.containers().create(
            image="nginx:alpine",
            name="example-nginx",
            ports={"80": "8080"},
            env=["ENV=development", "DEBUG=true"],
            labels={"example": "basic-ops", "tier": "web"}
        )
        print("✅ Container created successfully")
    except Exception as e:
        print(f"❌ Failed to create container: {e}")
        return
    
    # Start the container
    print("\n▶️  Starting container...")
    try:
        container.start()
        print("✅ Container started successfully")
        print("🌐 Nginx is available at http://localhost:8080")
    except Exception as e:
        print(f"❌ Failed to start container: {e}")
        return
    
    # Wait a moment for startup
    time.sleep(2)
    
    # Inspect the container
    print("\n🔍 Inspecting container...")
    try:
        info = container.inspect()
        state = info['State']
        config = info['Config']
        
        print(f"   Status: {state['Status']}")
        print(f"   Running: {state['Running']}")
        print(f"   Image: {config['Image']}")
        print(f"   Environment: {config['Env'][:3]}...")  # Show first 3 env vars
    except Exception as e:
        print(f"❌ Failed to inspect container: {e}")
    
    # Get container logs
    print("\n📋 Getting container logs...")
    try:
        logs = container.logs()
        print(f"   Log output: {logs[:100]}...")  # Show first 100 chars
    except Exception as e:
        print(f"❌ Failed to get logs: {e}")
    
    # Execute a command in the container
    print("\n⚡ Executing command in container...")
    try:
        result = container.exec(["nginx", "-v"])
        print(f"   Nginx version: {result.strip()}")
    except Exception as e:
        print(f"❌ Failed to execute command: {e}")
    
    # List running processes
    print("\n📊 Listing container processes...")
    try:
        processes = container.top()
        print(f"   Active processes: {len(processes.get('Processes', []))}")
    except Exception as e:
        print(f"❌ Failed to list processes: {e}")
    
    # Pause and unpause
    print("\n⏸️  Testing pause/unpause...")
    try:
        container.pause()
        print("   Container paused")
        time.sleep(1)
        
        container.unpause()
        print("   Container unpaused")
    except Exception as e:
        print(f"❌ Failed to pause/unpause: {e}")
    
    # Stop the container
    print("\n⏹️  Stopping container...")
    try:
        container.stop()
        print("✅ Container stopped successfully")
    except Exception as e:
        print(f"❌ Failed to stop container: {e}")
    
    # Remove the container
    print("\n🗑️  Removing container...")
    try:
        container.remove()
        print("✅ Container removed successfully")
    except Exception as e:
        print(f"❌ Failed to remove container: {e}")
    
    print("\n🎉 Basic container operations completed!")

if __name__ == "__main__":
    main()