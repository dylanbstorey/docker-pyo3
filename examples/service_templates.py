#!/usr/bin/env python3
"""
Service Templates Example

Demonstrates creating reusable service templates and patterns with docker-pyo3.
"""

from docker_pyo3 import Docker

class ServiceLibrary:
    """Library of reusable service templates"""
    
    def __init__(self, docker):
        self.docker = docker
    
    def web_service(self, name, image, port=80, replicas=1):
        """Create a web service template"""
        service = self.docker.create_service(name)
        service.image(image)
        service.ports([f"{port}:80"])
        service.restart_policy("unless-stopped")
        service.replicas(replicas)
        service.label("tier", "frontend")
        service.label("service_type", "web")
        return service
    
    def api_service(self, name, image, port=3000, database_url=None):
        """Create an API service template"""
        service = self.docker.create_service(name)
        service.image(image)
        service.ports([f"{port}:3000"])
        service.restart_policy("unless-stopped")
        service.env("NODE_ENV", "production")
        service.env("PORT", "3000")
        
        if database_url:
            service.env("DATABASE_URL", database_url)
        
        service.label("tier", "backend")
        service.label("service_type", "api")
        service.memory("512MB")
        return service
    
    def database_service(self, name, db_type="postgres", password="secret"):
        """Create a database service template"""
        service = self.docker.create_service(name)
        
        if db_type == "postgres":
            service.image("postgres:13")
            service.env("POSTGRES_PASSWORD", password)
            service.env("POSTGRES_DB", name)
            service.volume(f"{name}-data:/var/lib/postgresql/data")
        elif db_type == "mysql":
            service.image("mysql:8")
            service.env("MYSQL_ROOT_PASSWORD", password)
            service.env("MYSQL_DATABASE", name)
            service.volume(f"{name}-data:/var/lib/mysql")
        elif db_type == "mongo":
            service.image("mongo:5")
            service.env("MONGO_INITDB_ROOT_USERNAME", "admin")
            service.env("MONGO_INITDB_ROOT_PASSWORD", password)
            service.volume(f"{name}-data:/data/db")
        
        service.restart_policy("unless-stopped")
        service.memory("1GB")
        service.label("tier", "database")
        service.label("service_type", "database")
        service.label("database_type", db_type)
        return service
    
    def cache_service(self, name, cache_type="redis"):
        """Create a cache service template"""
        service = self.docker.create_service(name)
        
        if cache_type == "redis":
            service.image("redis:7-alpine")
            service.command(["redis-server", "--appendonly", "yes"])
            service.volume(f"{name}-data:/data")
        elif cache_type == "memcached":
            service.image("memcached:alpine")
            service.command(["memcached", "-m", "256"])
        
        service.restart_policy("unless-stopped")
        service.memory("256MB")
        service.label("tier", "cache")
        service.label("service_type", "cache")
        service.label("cache_type", cache_type)
        return service
    
    def worker_service(self, name, image, queue_url=None):
        """Create a background worker service template"""
        service = self.docker.create_service(name)
        service.image(image)
        service.restart_policy("unless-stopped")
        service.env("WORKER_MODE", "true")
        
        if queue_url:
            service.env("QUEUE_URL", queue_url)
        
        service.label("tier", "worker")
        service.label("service_type", "worker")
        service.memory("512MB")
        return service

def create_ecommerce_stack(docker, library):
    """Create a complete e-commerce application stack"""
    print("🛒 Creating e-commerce application stack...")
    
    stack = docker.create_stack("ecommerce")
    
    # Database layer
    main_db = library.database_service("main-db", "postgres", "secure_password")
    
    # Cache layer
    session_cache = library.cache_service("session-cache", "redis")
    
    # API layer
    user_api = library.api_service(
        "user-api", 
        "user-service:latest", 
        port=3001,
        database_url="postgresql://main-db:5432/ecommerce"
    )
    user_api.depends_on_service("main-db")
    user_api.depends_on_service("session-cache")
    
    product_api = library.api_service(
        "product-api",
        "product-service:latest",
        port=3002,
        database_url="postgresql://main-db:5432/ecommerce"
    )
    product_api.depends_on_service("main-db")
    
    order_api = library.api_service(
        "order-api",
        "order-service:latest", 
        port=3003,
        database_url="postgresql://main-db:5432/ecommerce"
    )
    order_api.depends_on_service("main-db")
    order_api.depends_on_service("session-cache")
    
    # Web layer
    web_frontend = library.web_service("web", "ecommerce-web:latest", port=8080, replicas=2)
    web_frontend.env("API_USER_URL", "http://user-api:3001")
    web_frontend.env("API_PRODUCT_URL", "http://product-api:3002")
    web_frontend.env("API_ORDER_URL", "http://order-api:3003")
    web_frontend.depends_on_service("user-api")
    web_frontend.depends_on_service("product-api")
    web_frontend.depends_on_service("order-api")
    
    # Worker layer
    email_worker = library.worker_service(
        "email-worker",
        "email-worker:latest",
        queue_url="redis://session-cache:6379"
    )
    email_worker.depends_on_service("session-cache")
    
    # Register all services
    services = [
        main_db, session_cache, user_api, product_api, 
        order_api, web_frontend, email_worker
    ]
    
    for service in services:
        stack.register_service(service)
    
    return stack

def create_development_stack(docker, library):
    """Create a simplified development stack"""
    print("🧪 Creating development stack...")
    
    stack = docker.create_stack("development")
    
    # Simple database
    dev_db = library.database_service("dev-db", "postgres", "dev_password")
    
    # Simple cache
    dev_cache = library.cache_service("dev-cache", "redis")
    
    # Single API service
    dev_api = library.api_service(
        "dev-api",
        "myapp:dev",
        port=3000,
        database_url="postgresql://dev-db:5432/myapp_dev"
    )
    dev_api.env("NODE_ENV", "development")
    dev_api.env("DEBUG", "true")
    dev_api.depends_on_service("dev-db")
    dev_api.depends_on_service("dev-cache")
    
    # Web frontend with hot reload
    dev_web = library.web_service("dev-web", "myapp-web:dev", port=3001)
    dev_web.env("NODE_ENV", "development")
    dev_web.env("API_URL", "http://dev-api:3000")
    dev_web.depends_on_service("dev-api")
    
    # Register services
    stack.register_service(dev_db)
    stack.register_service(dev_cache)
    stack.register_service(dev_api)
    stack.register_service(dev_web)
    
    return stack

def create_monitoring_stack(docker, library):
    """Create a monitoring and observability stack"""
    print("📊 Creating monitoring stack...")
    
    stack = docker.create_stack("monitoring")
    
    # Time series database for metrics
    prometheus = docker.create_service("prometheus")
    prometheus.image("prom/prometheus:latest")
    prometheus.ports(["9090:9090"])
    prometheus.volume("prometheus-data:/prometheus")
    prometheus.restart_policy("unless-stopped")
    prometheus.label("tier", "monitoring")
    
    # Metrics visualization
    grafana = docker.create_service("grafana")
    grafana.image("grafana/grafana:latest")
    grafana.ports(["3000:3000"])
    grafana.volume("grafana-data:/var/lib/grafana")
    grafana.env("GF_SECURITY_ADMIN_PASSWORD", "admin")
    grafana.depends_on_service("prometheus")
    grafana.restart_policy("unless-stopped")
    grafana.label("tier", "monitoring")
    
    # Log aggregation
    elasticsearch = docker.create_service("elasticsearch")
    elasticsearch.image("elasticsearch:7.17.0")
    elasticsearch.env("discovery.type", "single-node")
    elasticsearch.env("ES_JAVA_OPTS", "-Xms512m -Xmx512m")
    elasticsearch.volume("elasticsearch-data:/usr/share/elasticsearch/data")
    elasticsearch.memory("1GB")
    elasticsearch.restart_policy("unless-stopped")
    elasticsearch.label("tier", "logging")
    
    # Log visualization
    kibana = docker.create_service("kibana")
    kibana.image("kibana:7.17.0")
    kibana.ports(["5601:5601"])
    kibana.env("ELASTICSEARCH_HOSTS", "http://elasticsearch:9200")
    kibana.depends_on_service("elasticsearch")
    kibana.restart_policy("unless-stopped")
    kibana.label("tier", "logging")
    
    # Register services
    for service in [prometheus, grafana, elasticsearch, kibana]:
        stack.register_service(service)
    
    return stack

def main():
    docker = Docker()
    library = ServiceLibrary(docker)
    
    print("🏗️  Docker-PyO3 Service Templates Example")
    print("=" * 50)
    
    # Show available stack types
    print("Available stack templates:")
    print("1. E-commerce (microservices architecture)")
    print("2. Development (simple development environment)")
    print("3. Monitoring (observability stack)")
    
    choice = input("\nSelect stack to deploy (1-3): ").strip()
    
    if choice == "1":
        stack = create_ecommerce_stack(docker, library)
        app_url = "http://localhost:8080"
    elif choice == "2":
        stack = create_development_stack(docker, library)
        app_url = "http://localhost:3001"
    elif choice == "3":
        stack = create_monitoring_stack(docker, library)
        app_url = "http://localhost:3000 (Grafana), http://localhost:5601 (Kibana)"
    else:
        print("Invalid choice. Exiting.")
        return
    
    print(f"\n📋 Stack '{stack.name}' created with {stack.service_count()} services:")
    for service_name in stack.get_registered_services():
        print(f"   - {service_name}")
    
    # Deploy the stack
    print(f"\n🚀 Deploying {stack.name} stack...")
    try:
        stack.up()
        print("✅ Stack deployed successfully!")
        
        # Wait for startup
        import time
        print("⏱️  Waiting for services to initialize...")
        time.sleep(15)
        
        # Show status
        status = stack.status()
        print(f"\n📊 Stack Status:")
        print(f"   Overall: {status['status']}")
        print(f"   Containers: {status['total_containers']}")
        
        for service_name, service_info in status['services'].items():
            running = service_info['running']
            total = service_info['replicas']
            print(f"   {service_name}: {running}/{total} running")
        
        print(f"\n🌐 Application available at: {app_url}")
        print("Press Enter to clean up...")
        input()
        
    except Exception as e:
        print(f"❌ Deployment failed: {e}")
    
    # Cleanup
    print("\n🧹 Cleaning up...")
    try:
        stack.down()
        print("✅ Stack cleaned up successfully!")
    except Exception as e:
        print(f"❌ Cleanup failed: {e}")
    
    print("\n🎉 Service templates example completed!")

if __name__ == "__main__":
    main()