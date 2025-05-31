# Service Definition

Services provide a fluent API for defining containerized applications with reusable templates and configuration management.

## Basic Service Creation

### Creating Services

```python
from docker_pyo3 import Docker

docker = Docker()

# Create a basic service
service = docker.create_service("web")
service.image("nginx:latest")
service.ports(["80:8080"])
service.env("ENV", "production")
```

### Service Configuration Methods

```python
# Image configuration
service.image("nginx:alpine")

# Port mapping
service.ports(["80:8080", "443:8443"])

# Environment variables
service.env("DATABASE_URL", "postgresql://localhost/myapp")
service.env("DEBUG", "false")

# Volume mounting
service.volume("/app/data:/var/lib/app")
service.volume("app-logs:/var/log/app")

# Command and entrypoint
service.command(["nginx", "-g", "daemon off;"])
service.entrypoint(["sh", "-c"])

# Working directory
service.working_dir("/app")

# Networking
service.network("app_network")

# Dependencies
service.depends_on_service("database")

# Resource limits
service.memory("512MB")
service.replicas(2)

# Restart policy
service.restart_policy("unless-stopped")

# Labels
service.label("app", "myapp")
service.label("tier", "frontend")

# Hostname
service.hostname("web-server")
```

## Convenience Constructors

### Pre-configured Service Templates

```python
# Web service with common defaults
web_service = docker.create_service("web")
web_service = Service.web_service("web")  # Alternative static method
web_service.image("nginx:latest")
web_service.ports(["80:80"])

# Database service template
db_service = Service.database_service("db")
db_service.image("postgres:13")
db_service.env("POSTGRES_PASSWORD", "secret")
db_service.env("POSTGRES_DB", "myapp")

# Redis service template
cache_service = Service.redis_service("cache")
# Comes with sensible Redis defaults
```

## Service Cloning and Templates

### Creating Reusable Templates

```python
# Create a base template
base_web_template = Service.web_service("template")
base_web_template.image("nginx:alpine")
base_web_template.restart_policy("unless-stopped")
base_web_template.memory("256MB")

# Clone for different environments
dev_web = base_web_template.clone_with_name("web-dev")
dev_web.ports(["80:3000"])
dev_web.env("ENV", "development")
dev_web.env("DEBUG", "true")

prod_web = base_web_template.clone_with_name("web-prod")
prod_web.ports(["80:80"])
prod_web.env("ENV", "production")
prod_web.env("DEBUG", "false")
prod_web.replicas(3)

# Use in different stacks
dev_stack = docker.create_stack("myapp-dev")
prod_stack = docker.create_stack("myapp-prod")

dev_stack.register_service(dev_web)
prod_stack.register_service(prod_web)
```

### Service Library Pattern

```python
class ServiceLibrary:
    @staticmethod
    def nginx_web(name, port=80):
        service = Service.web_service(name)
        service.image("nginx:alpine")
        service.ports([f"{port}:80"])
        service.restart_policy("unless-stopped")
        return service
    
    @staticmethod
    def postgres_db(name, password, database):
        service = Service.database_service(name)
        service.image("postgres:13")
        service.env("POSTGRES_PASSWORD", password)
        service.env("POSTGRES_DB", database)
        service.volume(f"{name}-data:/var/lib/postgresql/data")
        return service
    
    @staticmethod
    def redis_cache(name):
        service = Service.redis_service(name)
        service.image("redis:7-alpine")
        service.command(["redis-server", "--appendonly", "yes"])
        service.volume(f"{name}-data:/data")
        return service

# Usage
web = ServiceLibrary.nginx_web("frontend", port=8080)
db = ServiceLibrary.postgres_db("database", "secret123", "myapp")
cache = ServiceLibrary.redis_cache("session-store")

# Deploy to stack
stack = docker.create_stack("ecommerce")
stack.register_service(web)
stack.register_service(db)
stack.register_service(cache)
stack.up()
```

## Advanced Service Configuration

### Complex Environment Setup

```python
# Environment variables from different sources
service = docker.create_service("app")
service.image("myapp:latest")

# Direct environment variables
service.env("NODE_ENV", "production")
service.env("PORT", "3000")

# Database connection
service.env("DB_HOST", "database")
service.env("DB_PORT", "5432")
service.env("DB_NAME", "myapp")

# External service URLs
service.env("REDIS_URL", "redis://cache:6379")
service.env("API_BASE_URL", "https://api.example.com")

# Feature flags
service.env("FEATURE_NEW_UI", "true")
service.env("ENABLE_METRICS", "true")
```

### Volume Mounting Strategies

```python
# Application code volume (development)
service.volume("./src:/app/src")

# Data persistence
service.volume("app-data:/app/data")

# Configuration files
service.volume("./config/app.conf:/etc/app/app.conf")

# Logs
service.volume("app-logs:/var/log/app")

# Shared volumes between services
service.volume("shared-uploads:/app/uploads")

# Read-only mounts
service.volume("./static:/app/static:ro")
```

### Networking Configuration

```python
# Custom network
service.network("app_backend")

# Multiple networks
service.network("frontend")
service.network("backend")

# Service discovery through networks
web_service = docker.create_service("web")
web_service.network("frontend")

api_service = docker.create_service("api")
api_service.network("frontend")
api_service.network("backend")

db_service = docker.create_service("db")
db_service.network("backend")

# Now web can reach api, api can reach db, but web cannot directly reach db
```

## Service Dependencies and Ordering

### Dependency Management

```python
# Define service dependencies
web_service = docker.create_service("web")
web_service.depends_on_service("api")

api_service = docker.create_service("api")
api_service.depends_on_service("database")
api_service.depends_on_service("cache")

database_service = docker.create_service("database")
cache_service = docker.create_service("cache")

# Stack will start services in dependency order:
# 1. database, cache (in parallel)
# 2. api (after database and cache are running)
# 3. web (after api is running)
```

### Health Check Configuration

```python
# Basic health check
service = docker.create_service("api")
service.image("myapi:latest")
service.healthcheck(
    test=["CMD", "curl", "-f", "http://localhost:3000/health"],
    interval="30s",
    timeout="10s",
    retries=3,
    start_period="40s"
)

# HTTP health check
web_service = docker.create_service("web")
web_service.healthcheck(
    test=["CMD-SHELL", "curl -f http://localhost || exit 1"],
    interval="30s",
    timeout="5s",
    retries=3
)

# Database health check
db_service = docker.create_service("db")
db_service.healthcheck(
    test=["CMD-SHELL", "pg_isready -U postgres"],
    interval="10s",
    timeout="5s",
    retries=5
)
```

## Integration with Stacks

### Multi-Service Application

```python
def create_web_app_stack(docker, stack_name):
    stack = docker.create_stack(stack_name)
    
    # Load balancer
    nginx = docker.create_service("nginx")
    nginx.image("nginx:alpine")
    nginx.ports(["80:80"])
    nginx.volume("./nginx.conf:/etc/nginx/nginx.conf:ro")
    nginx.depends_on_service("app")
    
    # Application servers
    app = docker.create_service("app")
    app.image("myapp:latest")
    app.env("DATABASE_URL", "postgresql://db:5432/myapp")
    app.env("REDIS_URL", "redis://cache:6379")
    app.depends_on_service("db")
    app.depends_on_service("cache")
    app.replicas(3)  # Multiple instances for load balancing
    
    # Database
    db = docker.create_service("db")
    db.image("postgres:13")
    db.env("POSTGRES_PASSWORD", "secret")
    db.env("POSTGRES_DB", "myapp")
    db.volume("db-data:/var/lib/postgresql/data")
    
    # Cache
    cache = docker.create_service("cache")
    cache.image("redis:7")
    cache.volume("cache-data:/data")
    
    # Register all services
    stack.register_service(nginx)
    stack.register_service(app)
    stack.register_service(db)
    stack.register_service(cache)
    
    return stack

# Deploy the application
web_app = create_web_app_stack(docker, "webapp")
web_app.up()

# Scale the application layer
web_app.scale("app", 5)

# Monitor the deployment
status = web_app.status()
print(f"Total containers: {status['total_containers']}")
```

### Microservices Architecture

```python
def create_microservices_stack(docker):
    stack = docker.create_stack("microservices")
    
    # API Gateway
    gateway = docker.create_service("gateway")
    gateway.image("traefik:v2.8")
    gateway.ports(["80:80", "8080:8080"])
    gateway.volume("/var/run/docker.sock:/var/run/docker.sock:ro")
    
    # User Service
    user_service = docker.create_service("user-service")
    user_service.image("user-api:latest")
    user_service.env("DB_HOST", "user-db")
    user_service.depends_on_service("user-db")
    
    user_db = docker.create_service("user-db")
    user_db.image("postgres:13")
    user_db.env("POSTGRES_DB", "users")
    user_db.env("POSTGRES_PASSWORD", "secret")
    
    # Order Service
    order_service = docker.create_service("order-service")
    order_service.image("order-api:latest")
    order_service.env("DB_HOST", "order-db")
    order_service.depends_on_service("order-db")
    
    order_db = docker.create_service("order-db")
    order_db.image("postgres:13")
    order_db.env("POSTGRES_DB", "orders")
    order_db.env("POSTGRES_PASSWORD", "secret")
    
    # Register services
    for service in [gateway, user_service, user_db, order_service, order_db]:
        stack.register_service(service)
    
    return stack

# Deploy microservices
microservices = create_microservices_stack(docker)
microservices.up()
```