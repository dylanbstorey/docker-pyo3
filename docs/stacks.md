# Stack Orchestration

Stack orchestration provides multi-container application management with service scaling, health monitoring, and centralized operations.

## Basic Stack Operations

### Creating and Deploying a Stack

```python
from docker_pyo3 import Docker

docker = Docker()

# Create a new stack
stack = docker.create_stack("myapp")

# Define services
web_service = docker.create_service("web")
web_service.image("nginx:latest")
web_service.ports(["80:8080"])
web_service.env("ENV", "production")

db_service = docker.create_service("db")
db_service.image("postgres:13")
db_service.env("POSTGRES_PASSWORD", "secret")
db_service.env("POSTGRES_DB", "myapp")

# Register services to stack
stack.register_service(web_service)
stack.register_service(db_service)

# Deploy the stack
stack.up()
```

### Stack Status and Health Monitoring

```python
# Get detailed stack status
status = stack.status()
print(f"Stack status: {status['status']}")
print(f"Total containers: {status['total_containers']}")

# Check individual service health
for service_name, service_info in status['services'].items():
    print(f"Service {service_name}:")
    print(f"  Replicas: {service_info['replicas']}")
    print(f"  Running: {service_info['running']}")
    print(f"  Healthy: {service_info['healthy']}")
    print(f"  Unhealthy: {service_info['unhealthy']}")
    
    # Individual container details
    for container in service_info['containers']:
        print(f"    Container {container['id'][:12]}:")
        print(f"      Running: {container['running']}")
        print(f"      Health: {container['health']}")
        print(f"      Status: {container['status']}")
```

## Service Scaling

### Dynamic Scaling

```python
# Scale web service to 3 replicas
stack.scale("web", 3)

# Check scaling results
status = stack.status()
web_replicas = status['services']['web']['replicas']
print(f"Web service now has {web_replicas} replicas")

# Scale back down
stack.scale("web", 1)
```

### Scaling Example with Load Testing

```python
import time

# Deploy initial stack
stack.up()
time.sleep(2)

# Simulate load increase - scale up
print("Scaling up for high load...")
stack.scale("web", 5)

# Monitor scaling
status = stack.status()
while status['services']['web']['running'] < 5:
    print(f"Scaling in progress... {status['services']['web']['running']}/5 running")
    time.sleep(1)
    status = stack.status()

print("Scale up complete!")

# Later, scale down during low traffic
print("Scaling down for low traffic...")
stack.scale("web", 2)
```

## Logging and Monitoring

### Centralized Logging

```python
# Get logs from all services
all_logs = stack.logs()
print(all_logs)

# Get logs from specific services
web_logs = stack.logs(["web"])
print("Web service logs:")
print(web_logs)

# Application-specific log parsing
if "error" in all_logs.lower():
    print("⚠️  Errors detected in application logs")
    
if "database connection" in all_logs.lower():
    print("📊 Database connectivity confirmed")
```

### Health Check Integration

```python
# Wait for services to be healthy
import time

stack.up()
max_wait = 60  # seconds
wait_time = 0

while wait_time < max_wait:
    status = stack.status()
    
    all_healthy = True
    for service_name, service_info in status['services'].items():
        if service_info['unhealthy'] > 0:
            all_healthy = False
            print(f"Service {service_name} has unhealthy containers")
    
    if all_healthy:
        print("✅ All services are healthy!")
        break
        
    time.sleep(5)
    wait_time += 5
```

## Advanced Stack Management

### Service Restart

```python
# Restart individual services
stack.restart_service("web")

# Check if restart was successful
status = stack.status()
if status['services']['web']['running'] > 0:
    print("✅ Web service restart successful")
```

### Stack Cleanup

```python
# Stop and remove all stack resources
stack.down()

# Verify cleanup
status = stack.status()
if status['status'] == 'not_deployed':
    print("✅ Stack successfully cleaned up")
```

### Multi-Stack Management

```python
# Create multiple stacks for different environments
dev_stack = docker.create_stack("myapp-dev")
prod_stack = docker.create_stack("myapp-prod")

# Configure services differently per environment
dev_web = docker.create_service("web")
dev_web.image("nginx:latest")
dev_web.ports(["80:3000"])  # Development port
dev_web.env("ENV", "development")

prod_web = docker.create_service("web")
prod_web.image("nginx:latest")
prod_web.ports(["80:80"])   # Production port
prod_web.env("ENV", "production")

# Deploy to different environments
dev_stack.register_service(dev_web)
prod_stack.register_service(prod_web)

dev_stack.up()
prod_stack.up()

# Scale production differently
prod_stack.scale("web", 3)  # High availability
dev_stack.scale("web", 1)   # Single instance for dev
```

## Error Handling

```python
try:
    # Attempt stack operations
    stack.up()
    stack.scale("web", 3)
    
except Exception as e:
    print(f"Stack operation failed: {e}")
    
    # Get current status for debugging
    try:
        status = stack.status()
        print("Current stack status:", status)
    except:
        print("Unable to get stack status")
    
    # Attempt cleanup
    try:
        stack.down()
        print("Stack cleanup completed")
    except:
        print("Manual cleanup may be required")
```

## Performance Considerations

### Resource Monitoring

```python
# Monitor resource usage
status = stack.status()

total_containers = status['total_containers']
print(f"Managing {total_containers} containers")

if total_containers > 10:
    print("⚠️  High container count - consider resource limits")

# Check for failed containers
for service_name, service_info in status['services'].items():
    failed_containers = len([c for c in service_info['containers'] 
                           if not c['running']])
    if failed_containers > 0:
        print(f"⚠️  {failed_containers} failed containers in {service_name}")
```

### Efficient Scaling Patterns

```python
# Gradual scaling instead of immediate jumps
def gradual_scale(stack, service_name, target_replicas, step_size=1):
    status = stack.status()
    current_replicas = status['services'][service_name]['replicas']
    
    while current_replicas != target_replicas:
        if current_replicas < target_replicas:
            next_replicas = min(current_replicas + step_size, target_replicas)
        else:
            next_replicas = max(current_replicas - step_size, target_replicas)
        
        print(f"Scaling {service_name}: {current_replicas} -> {next_replicas}")
        stack.scale(service_name, next_replicas)
        
        # Wait for scaling to complete
        time.sleep(2)
        status = stack.status()
        current_replicas = status['services'][service_name]['replicas']

# Usage
gradual_scale(stack, "web", 5, step_size=2)
```