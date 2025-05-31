# Container Management

Container management provides comprehensive Docker container operations with advanced parameter support and lifecycle management.

## Basic Container Operations

### Creating Containers

```python
from docker_pyo3 import Docker

docker = Docker()

# Basic container creation
container = docker.containers().create(
    image="nginx:latest",
    name="my-nginx"
)

# Advanced container creation with parameters
container = docker.containers().create(
    image="postgres:13",
    name="my-database",
    env=["POSTGRES_PASSWORD=secret", "POSTGRES_DB=myapp"],
    ports={"5432": "5432"},
    volumes=["/data:/var/lib/postgresql/data"],
    restart_policy={"name": "unless-stopped"},
    memory=1073741824,  # 1GB in bytes
    labels={"app": "database", "tier": "backend"}
)
```

### Container Lifecycle

```python
# Start container
container.start()

# Stop container
container.stop()

# Restart container
container.restart()

# Pause/unpause container
container.pause()
container.unpause()

# Kill container
container.kill()

# Remove container
container.remove(force=True)
```

## Advanced Container Configuration

### Environment Variables

```python
# Multiple environment variables
container = docker.containers().create(
    image="myapp:latest",
    name="myapp",
    env=[
        "NODE_ENV=production",
        "PORT=3000",
        "DATABASE_URL=postgresql://db:5432/myapp",
        "REDIS_URL=redis://cache:6379",
        "API_KEY=your-secret-key"
    ]
)
```

### Volume Mounting

```python
# Named volume
docker.volumes().create(name="app-data")

container = docker.containers().create(
    image="myapp:latest",
    name="myapp",
    volumes=[
        "app-data:/app/data",           # Named volume
        "/host/path:/container/path",   # Bind mount
        "/host/config:/app/config:ro"   # Read-only bind mount
    ]
)
```

### Port Mapping

```python
container = docker.containers().create(
    image="nginx:latest",
    name="web-server",
    ports={
        "80": "8080",    # Host port 8080 -> Container port 80
        "443": "8443"    # Host port 8443 -> Container port 443
    }
)
```

### Resource Limits

```python
container = docker.containers().create(
    image="myapp:latest",
    name="resource-limited",
    memory=536870912,     # 512MB
    cpu_shares=512,       # CPU shares
    cpus=1.5             # CPU limit
)
```

### Restart Policies

```python
# Always restart
container = docker.containers().create(
    image="myapp:latest",
    name="always-restart",
    restart_policy={"name": "always"}
)

# Restart on failure with max retries
container = docker.containers().create(
    image="myapp:latest",
    name="restart-on-failure",
    restart_policy={
        "name": "on-failure",
        "maximum_retry_count": 3
    }
)

# Unless stopped
container = docker.containers().create(
    image="myapp:latest", 
    name="unless-stopped",
    restart_policy={"name": "unless-stopped"}
)
```

### Network Configuration

```python
# Custom network
network = docker.networks().create("myapp-network")

container = docker.containers().create(
    image="myapp:latest",
    name="networked-app",
    network_mode="myapp-network"
)

# Extra hosts
container = docker.containers().create(
    image="myapp:latest",
    name="with-hosts",
    extra_hosts=[
        "database:192.168.1.100",
        "cache:192.168.1.101"
    ]
)
```

### Working Directory and User

```python
container = docker.containers().create(
    image="myapp:latest",
    name="configured-app",
    working_dir="/app",
    user="1000:1000",  # UID:GID
    entrypoint=["python3"],
    command=["app.py", "--config", "/app/config.yml"]
)
```

## Container Inspection and Monitoring

### Container Information

```python
# Get container details
info = container.inspect()
print(f"Container ID: {info['Id']}")
print(f"State: {info['State']['Status']}")
print(f"Image: {info['Config']['Image']}")

# Get container ID
container_id = container.id()
print(f"Container ID: {container_id}")
```

### Container Logs

```python
# Get all logs
logs = container.logs()
print(logs)

# Get logs with timestamps
logs = container.logs(
    stdout=True,
    stderr=True,
    timestamps=True,
    tail=100  # Last 100 lines
)

# Follow logs (streaming)
logs = container.logs(
    stdout=True,
    stderr=True,
    follow=True
)
```

### Process Management

```python
# List running processes
processes = container.top()
print(f"Running processes: {processes}")

# Execute command in running container
result = container.exec(["ls", "-la", "/app"])
print(f"Directory listing: {result}")

# Interactive execution
result = container.exec(
    ["bash", "-c", "echo 'Hello from container'"],
    stdout=True,
    stderr=True
)
```

## File Operations

### File Transfer

```python
# Copy file into container
with open("local-file.txt", "rb") as f:
    container.copy_file_into(f.read(), "/app/config.txt")

# Copy file from container
file_data = container.copy_from("/app/output.txt")
with open("local-output.txt", "wb") as f:
    f.write(file_data)

# Get file statistics
stat_info = container.stat_file("/app/data.txt")
print(f"File size: {stat_info['size']}")
print(f"Modified: {stat_info['mtime']}")
```

## Container Listing and Filtering

### List Containers

```python
# List all containers (running and stopped)
all_containers = docker.containers().list(all=True)

# List only running containers
running_containers = docker.containers().list()

# Filter containers
filtered_containers = docker.containers().list(
    all=True,
    since="container-name",  # Containers created after this one
    before="other-container" # Containers created before this one
)
```

### Container Search and Selection

```python
# Get specific container
container = docker.containers().get("container-name-or-id")

# Find containers by label
all_containers = docker.containers().list(all=True)
app_containers = [c for c in all_containers 
                 if c.get('Labels', {}).get('app') == 'myapp']

# Find containers by image
nginx_containers = [c for c in all_containers
                   if 'nginx' in c.get('Image', '')]
```

## Error Handling

### Robust Container Operations

```python
def safe_container_operation(docker, image, name, operation):
    try:
        # Try to get existing container
        container = docker.containers().get(name)
        print(f"Container {name} already exists")
        
        # Check if it's running
        info = container.inspect()
        is_running = info['State']['Running']
        
        if operation == "start" and not is_running:
            container.start()
            print(f"Started existing container {name}")
        elif operation == "stop" and is_running:
            container.stop()
            print(f"Stopped container {name}")
            
    except Exception as e:
        if "not found" in str(e).lower() and operation == "start":
            # Container doesn't exist, create it
            print(f"Creating new container {name}")
            container = docker.containers().create(
                image=image,
                name=name
            )
            container.start()
            print(f"Created and started container {name}")
        else:
            print(f"Error in container operation: {e}")
            raise

# Usage
safe_container_operation(docker, "nginx:latest", "my-nginx", "start")
```

### Container Health Monitoring

```python
import time

def wait_for_container_healthy(container, timeout=60):
    """Wait for container to become healthy"""
    start_time = time.time()
    
    while time.time() - start_time < timeout:
        try:
            info = container.inspect()
            state = info['State']
            
            # Check if container is running
            if not state['Running']:
                print(f"Container is not running: {state['Status']}")
                return False
            
            # Check health status if available
            health = state.get('Health')
            if health:
                health_status = health['Status']
                if health_status == 'healthy':
                    print("Container is healthy!")
                    return True
                elif health_status == 'unhealthy':
                    print("Container is unhealthy!")
                    return False
                else:
                    print(f"Health status: {health_status}")
            else:
                # No health check defined, assume healthy if running
                print("No health check defined, container is running")
                return True
                
        except Exception as e:
            print(f"Error checking container health: {e}")
            return False
        
        time.sleep(5)
    
    print(f"Timeout waiting for container to become healthy")
    return False

# Usage
container = docker.containers().create(
    image="nginx:latest",
    name="health-monitored"
)
container.start()

if wait_for_container_healthy(container):
    print("Container is ready!")
else:
    print("Container failed to become healthy")
```

## Performance and Optimization

### Resource Monitoring

```python
def monitor_container_resources(container):
    """Monitor container resource usage"""
    try:
        info = container.inspect()
        
        # Get resource configuration
        host_config = info.get('HostConfig', {})
        memory_limit = host_config.get('Memory', 0)
        cpu_shares = host_config.get('CpuShares', 0)
        
        print(f"Memory limit: {memory_limit / (1024*1024):.0f}MB")
        print(f"CPU shares: {cpu_shares}")
        
        # Get current state
        state = info['State']
        print(f"Status: {state['Status']}")
        print(f"Started at: {state['StartedAt']}")
        
        if state['Running']:
            print("Container is currently running")
        else:
            exit_code = state['ExitCode']
            print(f"Container exited with code: {exit_code}")
            
    except Exception as e:
        print(f"Error monitoring container: {e}")

# Usage
monitor_container_resources(container)
```

### Efficient Container Management

```python
def manage_container_lifecycle(docker, config):
    """Efficiently manage container lifecycle"""
    name = config['name']
    
    try:
        # Try to get existing container
        container = docker.containers().get(name)
        info = container.inspect()
        
        # Check if configuration matches
        current_image = info['Config']['Image']
        desired_image = config['image']
        
        if current_image != desired_image:
            print(f"Image changed: {current_image} -> {desired_image}")
            # Remove old container and create new one
            container.stop()
            container.remove(force=True)
            container = docker.containers().create(**config)
            container.start()
            print(f"Recreated container with new image")
        else:
            # Ensure container is running
            if not info['State']['Running']:
                container.start()
                print(f"Started existing container")
            else:
                print(f"Container is already running")
                
        return container
        
    except Exception as e:
        if "not found" in str(e).lower():
            # Create new container
            container = docker.containers().create(**config)
            container.start()
            print(f"Created new container")
            return container
        else:
            raise

# Usage
config = {
    'image': 'nginx:latest',
    'name': 'my-web-server',
    'ports': {'80': '8080'},
    'restart_policy': {'name': 'unless-stopped'}
}

container = manage_container_lifecycle(docker, config)
```