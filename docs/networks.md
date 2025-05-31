# Network Management

Network management provides Docker networking operations including network creation, container connectivity, and network isolation.

## Basic Network Operations

### Creating Networks

```python
from docker_pyo3 import Docker

docker = Docker()

# Create basic network
network = docker.networks().create("myapp-network")

# Create network with custom driver
network = docker.networks().create(
    name="custom-network",
    driver="bridge"
)

# Create network with options
network = docker.networks().create(
    name="app-backend",
    driver="bridge",
    options={"com.docker.network.bridge.name": "docker1"}
)
```

### Network Listing and Inspection

```python
# List all networks
networks = docker.networks().list()
for net in networks:
    print(f"Network: {net['Name']}")
    print(f"Driver: {net['Driver']}")
    print(f"Scope: {net['Scope']}")

# Get specific network
network = docker.networks().get("myapp-network")

# Inspect network details
info = network.inspect()
print(f"Network ID: {info['Id']}")
print(f"Created: {info['Created']}")
print(f"Containers: {list(info['Containers'].keys())}")
```

## Container Network Connectivity

### Connecting Containers to Networks

```python
# Create containers
web_container = docker.containers().create(
    image="nginx:latest",
    name="web-server"
)

api_container = docker.containers().create(
    image="myapi:latest", 
    name="api-server"
)

# Create custom network
app_network = docker.networks().create("app-network")

# Connect containers to network
app_network.connect("web-server")
app_network.connect("api-server")

# Start containers
web_container.start()
api_container.start()

# Now containers can communicate using container names as hostnames
```

### Advanced Connection Options

```python
# Connect with aliases
app_network.connect(
    container="web-server",
    aliases=["web", "frontend"]
)

# Connect with specific IP (if subnet allows)
app_network.connect(
    container="api-server",
    ipv4_address="172.20.0.10"
)
```

### Disconnecting Containers

```python
# Disconnect container from network
app_network.disconnect("web-server")

# Force disconnect (even if container is running)
app_network.disconnect("api-server", force=True)
```

## Network Architectures

### Multi-Tier Application

```python
def setup_multi_tier_network(docker):
    """Setup network architecture for multi-tier application"""
    
    # Create separate networks for different tiers
    frontend_net = docker.networks().create("frontend")
    backend_net = docker.networks().create("backend")
    database_net = docker.networks().create("database")
    
    # Load balancer (public-facing)
    lb_container = docker.containers().create(
        image="nginx:latest",
        name="load-balancer",
        ports={"80": "80", "443": "443"}
    )
    
    # Web servers (frontend tier)
    web_containers = []
    for i in range(3):
        web_container = docker.containers().create(
            image="myapp:latest",
            name=f"web-{i}"
        )
        web_containers.append(web_container)
    
    # API servers (backend tier)
    api_containers = []
    for i in range(2):
        api_container = docker.containers().create(
            image="myapi:latest",
            name=f"api-{i}"
        )
        api_containers.append(api_container)
    
    # Database (data tier)
    db_container = docker.containers().create(
        image="postgres:13",
        name="database",
        env=["POSTGRES_PASSWORD=secret"]
    )
    
    # Connect load balancer to frontend network
    frontend_net.connect("load-balancer")
    
    # Connect web servers to frontend and backend networks
    for container in web_containers:
        frontend_net.connect(container.name)
        backend_net.connect(container.name)
    
    # Connect API servers to backend and database networks  
    for container in api_containers:
        backend_net.connect(container.name)
        database_net.connect(container.name)
    
    # Connect database only to database network
    database_net.connect("database")
    
    # Start all containers
    lb_container.start()
    for container in web_containers + api_containers:
        container.start()
    db_container.start()
    
    print("Multi-tier network architecture deployed!")
    print("- Load balancer: public access")
    print("- Web servers: frontend + backend networks")
    print("- API servers: backend + database networks") 
    print("- Database: database network only")

# Deploy the architecture
setup_multi_tier_network(docker)
```

### Microservices Network

```python
def setup_microservices_network(docker):
    """Setup network for microservices architecture"""
    
    # Create service-specific networks
    user_net = docker.networks().create("user-service")
    order_net = docker.networks().create("order-service")
    payment_net = docker.networks().create("payment-service")
    shared_net = docker.networks().create("api-gateway")
    
    # API Gateway
    gateway = docker.containers().create(
        image="traefik:v2.8",
        name="api-gateway",
        ports={"80": "80", "8080": "8080"},
        volumes=["/var/run/docker.sock:/var/run/docker.sock:ro"]
    )
    
    # User service and database
    user_service = docker.containers().create(
        image="user-api:latest",
        name="user-service"
    )
    user_db = docker.containers().create(
        image="postgres:13",
        name="user-db",
        env=["POSTGRES_DB=users", "POSTGRES_PASSWORD=secret"]
    )
    
    # Order service and database
    order_service = docker.containers().create(
        image="order-api:latest", 
        name="order-service"
    )
    order_db = docker.containers().create(
        image="postgres:13",
        name="order-db",
        env=["POSTGRES_DB=orders", "POSTGRES_PASSWORD=secret"]
    )
    
    # Payment service and database
    payment_service = docker.containers().create(
        image="payment-api:latest",
        name="payment-service"
    )
    payment_db = docker.containers().create(
        image="postgres:13",
        name="payment-db", 
        env=["POSTGRES_DB=payments", "POSTGRES_PASSWORD=secret"]
    )
    
    # Network connections
    # Gateway connects to all service networks
    shared_net.connect("api-gateway")
    user_net.connect("api-gateway")
    order_net.connect("api-gateway")
    payment_net.connect("api-gateway")
    
    # Each service connects to its own network and shared network
    user_net.connect("user-service")
    user_net.connect("user-db")
    shared_net.connect("user-service")
    
    order_net.connect("order-service")
    order_net.connect("order-db") 
    shared_net.connect("order-service")
    
    payment_net.connect("payment-service")
    payment_net.connect("payment-db")
    shared_net.connect("payment-service")
    
    # Start all services
    containers = [
        gateway, user_service, user_db,
        order_service, order_db, payment_service, payment_db
    ]
    
    for container in containers:
        container.start()
    
    print("Microservices network deployed!")
    print("Services can communicate through API gateway")
    print("Each service has isolated database access")

# Deploy microservices
setup_microservices_network(docker)
```

## Network Security and Isolation

### Network Isolation

```python
def create_isolated_environments(docker):
    """Create isolated environments for different purposes"""
    
    # Production environment
    prod_net = docker.networks().create(
        name="production",
        driver="bridge",
        options={"com.docker.network.bridge.enable_icc": "false"}
    )
    
    # Development environment  
    dev_net = docker.networks().create(
        name="development", 
        driver="bridge"
    )
    
    # Testing environment
    test_net = docker.networks().create(
        name="testing",
        driver="bridge"
    )
    
    # Deploy same application to different environments
    environments = [
        ("prod", prod_net, "myapp:latest"),
        ("dev", dev_net, "myapp:dev"), 
        ("test", test_net, "myapp:test")
    ]
    
    for env_name, network, image in environments:
        # Web container
        web = docker.containers().create(
            image=image,
            name=f"{env_name}-web",
            env=[f"ENV={env_name}"]
        )
        
        # Database container
        db = docker.containers().create(
            image="postgres:13",
            name=f"{env_name}-db",
            env=[f"POSTGRES_DB={env_name}_db", "POSTGRES_PASSWORD=secret"]
        )
        
        # Connect to environment network
        network.connect(f"{env_name}-web")
        network.connect(f"{env_name}-db")
        
        # Start containers
        web.start()
        db.start()
        
        print(f"Deployed {env_name} environment")

# Create isolated environments
create_isolated_environments(docker)
```

### Network Access Control

```python
def setup_network_access_control(docker):
    """Setup network with access control between services"""
    
    # DMZ network (public-facing services)
    dmz_net = docker.networks().create("dmz")
    
    # Internal network (internal services only)
    internal_net = docker.networks().create("internal")
    
    # Database network (database access only)
    db_net = docker.networks().create("database")
    
    # Public load balancer (DMZ only)
    lb = docker.containers().create(
        image="nginx:latest",
        name="public-lb",
        ports={"80": "80"}
    )
    dmz_net.connect("public-lb")
    
    # Web servers (DMZ + Internal)
    web = docker.containers().create(
        image="webapp:latest", 
        name="web-server"
    )
    dmz_net.connect("web-server")
    internal_net.connect("web-server")
    
    # API servers (Internal + Database)
    api = docker.containers().create(
        image="api:latest",
        name="api-server"
    )
    internal_net.connect("api-server") 
    db_net.connect("api-server")
    
    # Database (Database network only)
    db = docker.containers().create(
        image="postgres:13",
        name="database",
        env=["POSTGRES_PASSWORD=secret"]
    )
    db_net.connect("database")
    
    # Background workers (Internal + Database)
    worker = docker.containers().create(
        image="worker:latest",
        name="background-worker"
    )
    internal_net.connect("background-worker")
    db_net.connect("background-worker")
    
    # Start all containers
    for container in [lb, web, api, db, worker]:
        container.start()
    
    print("Network access control deployed:")
    print("- Public LB: DMZ only (internet access)")
    print("- Web servers: DMZ + Internal (can reach API)")
    print("- API servers: Internal + Database (cannot reach internet)")
    print("- Database: Database network only (maximum isolation)")
    print("- Workers: Internal + Database (no internet, can reach API)")

# Setup access control
setup_network_access_control(docker)
```

## Network Monitoring and Troubleshooting

### Network Diagnostics

```python
def diagnose_network_connectivity(docker, network_name):
    """Diagnose network connectivity issues"""
    
    try:
        network = docker.networks().get(network_name)
        info = network.inspect()
        
        print(f"Network: {network_name}")
        print(f"Driver: {info['Driver']}")
        print(f"Scope: {info['Scope']}")
        print(f"Created: {info['Created']}")
        
        # List connected containers
        containers = info.get('Containers', {})
        print(f"\nConnected containers ({len(containers)}):")
        
        for container_id, container_info in containers.items():
            name = container_info['Name']
            ipv4 = container_info.get('IPv4Address', 'N/A')
            print(f"  - {name}: {ipv4}")
            
            # Test container connectivity
            try:
                container = docker.containers().get(name)
                container_inspect = container.inspect()
                
                if container_inspect['State']['Running']:
                    print(f"    Status: Running ✅")
                    
                    # Test network connectivity from this container
                    test_cmd = ["ping", "-c", "1", "8.8.8.8"]
                    try:
                        result = container.exec(test_cmd)
                        if "1 packets transmitted, 1 received" in result:
                            print(f"    Internet: Connected ✅")
                        else:
                            print(f"    Internet: Failed ❌")
                    except:
                        print(f"    Internet: Test failed ❌")
                        
                else:
                    print(f"    Status: Not running ❌")
                    
            except Exception as e:
                print(f"    Error: {e} ❌")
        
        # Check network configuration
        config = info.get('IPAM', {})
        print(f"\nNetwork Configuration:")
        print(f"  Driver: {config.get('Driver', 'N/A')}")
        
        for i, subnet_config in enumerate(config.get('Config', [])):
            subnet = subnet_config.get('Subnet', 'N/A')
            gateway = subnet_config.get('Gateway', 'N/A')
            print(f"  Subnet {i}: {subnet}")
            print(f"  Gateway {i}: {gateway}")
            
    except Exception as e:
        print(f"Error diagnosing network {network_name}: {e}")

# Usage
diagnose_network_connectivity(docker, "myapp-network")
```

### Container Communication Testing

```python
def test_container_communication(docker, network_name):
    """Test communication between containers on a network"""
    
    network = docker.networks().get(network_name)
    info = network.inspect()
    containers = list(info.get('Containers', {}).values())
    
    if len(containers) < 2:
        print("Need at least 2 containers to test communication")
        return
    
    print(f"Testing communication on network {network_name}")
    
    # Test communication between each pair of containers
    for i, container_a in enumerate(containers):
        for j, container_b in enumerate(containers):
            if i >= j:  # Skip self and duplicate tests
                continue
                
            name_a = container_a['Name']
            name_b = container_b['Name']
            
            print(f"\nTesting: {name_a} -> {name_b}")
            
            try:
                container = docker.containers().get(name_a)
                
                # Test ping connectivity
                ping_cmd = ["ping", "-c", "1", name_b]
                result = container.exec(ping_cmd)
                
                if "1 packets transmitted, 1 received" in result:
                    print(f"  Ping: Success ✅")
                else:
                    print(f"  Ping: Failed ❌")
                
                # Test DNS resolution
                nslookup_cmd = ["nslookup", name_b]
                result = container.exec(nslookup_cmd)
                
                if "can't resolve" not in result.lower():
                    print(f"  DNS: Success ✅")
                else:
                    print(f"  DNS: Failed ❌")
                    
            except Exception as e:
                print(f"  Error: {e} ❌")

# Usage
test_container_communication(docker, "myapp-network")
```

## Network Cleanup and Maintenance

### Network Pruning

```python
# Remove unused networks
pruned = docker.networks().prune()
print(f"Removed networks: {pruned['NetworksDeleted']}")

# Remove specific network
try:
    network = docker.networks().get("old-network")
    network.delete()
    print("Network removed successfully")
except Exception as e:
    print(f"Failed to remove network: {e}")
```

### Network Maintenance

```python
def cleanup_networks(docker, keep_defaults=True):
    """Clean up unused networks"""
    
    networks = docker.networks().list()
    default_networks = ["bridge", "host", "none"]
    
    for network in networks:
        name = network['Name']
        network_id = network['Id']
        
        # Skip default networks if requested
        if keep_defaults and name in default_networks:
            continue
        
        try:
            net_obj = docker.networks().get(network_id)
            info = net_obj.inspect()
            
            # Check if network has connected containers
            containers = info.get('Containers', {})
            
            if not containers:
                print(f"Removing unused network: {name}")
                net_obj.delete()
            else:
                print(f"Keeping network {name} (has {len(containers)} containers)")
                
        except Exception as e:
            print(f"Error processing network {name}: {e}")

# Usage
cleanup_networks(docker)
```