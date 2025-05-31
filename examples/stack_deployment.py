#!/usr/bin/env python3
"""
Stack Deployment Example

Demonstrates multi-container application deployment using docker-pyo3 stacks.
"""

from docker_pyo3 import Docker
import time

def main():
    docker = Docker()
    
    print("🚀 Docker-PyO3 Stack Deployment Example")
    print("=" * 50)
    
    # Create a new stack
    print("📦 Creating application stack...")
    stack = docker.create_stack("webapp-example")
    
    # Define database service
    print("🗄️  Defining database service...")
    db_service = docker.create_service("database")
    db_service.image("postgres:13")
    db_service.env("POSTGRES_PASSWORD", "example_password")
    db_service.env("POSTGRES_DB", "webapp")
    db_service.env("POSTGRES_USER", "webapp_user")
    db_service.memory("512MB")
    db_service.restart_policy("unless-stopped")
    db_service.label("tier", "database")
    
    # Define web application service
    print("🌐 Defining web application service...")
    web_service = docker.create_service("webapp")
    web_service.image("nginx:alpine")
    web_service.ports(["8080:80"])
    web_service.env("DATABASE_URL", "postgresql://webapp_user:example_password@database:5432/webapp")
    web_service.env("ENVIRONMENT", "production")
    web_service.depends_on_service("database")
    web_service.restart_policy("unless-stopped")
    web_service.label("tier", "frontend")
    
    # Define cache service
    print("💾 Defining cache service...")
    cache_service = docker.create_service("cache")
    cache_service.image("redis:7-alpine")
    cache_service.command(["redis-server", "--appendonly", "yes"])
    cache_service.memory("256MB")
    cache_service.restart_policy("unless-stopped")
    cache_service.label("tier", "cache")
    
    # Register services to stack
    print("📋 Registering services to stack...")
    stack.register_service(db_service)
    stack.register_service(cache_service)
    stack.register_service(web_service)
    
    print(f"   Registered {stack.service_count()} services")
    print(f"   Services: {', '.join(stack.get_registered_services())}")
    
    # Deploy the stack
    print("\n🚀 Deploying stack...")
    try:
        stack.up()
        print("✅ Stack deployed successfully!")
    except Exception as e:
        print(f"❌ Stack deployment failed: {e}")
        return
    
    # Wait for services to start
    print("\n⏱️  Waiting for services to initialize...")
    time.sleep(10)
    
    # Check stack status
    print("\n📊 Checking stack status...")
    try:
        status = stack.status()
        
        print(f"   Overall status: {status['status']}")
        print(f"   Total containers: {status['total_containers']}")
        print(f"   Networks: {status['networks']}")
        
        print("\n   Service details:")
        for service_name, service_info in status['services'].items():
            print(f"     {service_name}:")
            print(f"       Replicas: {service_info['replicas']}")
            print(f"       Running: {service_info['running']}")
            print(f"       Healthy: {service_info['healthy']}")
            print(f"       Unhealthy: {service_info['unhealthy']}")
            
            # Show container details
            for container in service_info['containers']:
                container_id = container['id'][:12]
                print(f"         Container {container_id}: {container['status']} (Health: {container['health']})")
                
    except Exception as e:
        print(f"❌ Failed to get stack status: {e}")
    
    # Test service scaling
    print("\n📈 Testing service scaling...")
    try:
        print("   Scaling webapp to 3 replicas...")
        stack.scale("webapp", 3)
        
        time.sleep(5)  # Wait for scaling
        
        status = stack.status()
        webapp_replicas = status['services']['webapp']['replicas']
        print(f"   ✅ Webapp now has {webapp_replicas} replicas")
        
    except Exception as e:
        print(f"❌ Scaling failed: {e}")
    
    # Get application logs
    print("\n📋 Getting application logs...")
    try:
        logs = stack.logs(["webapp"])
        print("   Recent webapp logs:")
        log_lines = logs.split('\n')[:5]  # Show first 5 lines
        for line in log_lines:
            if line.strip():
                print(f"     {line}")
                
    except Exception as e:
        print(f"❌ Failed to get logs: {e}")
    
    # Demonstrate service restart
    print("\n🔄 Testing service restart...")
    try:
        stack.restart_service("webapp")
        print("   ✅ Webapp service restarted")
        
        time.sleep(3)
        
        status = stack.status()
        webapp_running = status['services']['webapp']['running']
        print(f"   Running webapp containers: {webapp_running}")
        
    except Exception as e:
        print(f"❌ Service restart failed: {e}")
    
    # Scale back down
    print("\n📉 Scaling back down...")
    try:
        stack.scale("webapp", 1)
        print("   ✅ Scaled webapp back to 1 replica")
    except Exception as e:
        print(f"❌ Scale down failed: {e}")
    
    # Show final status
    print("\n📊 Final stack status:")
    try:
        status = stack.status()
        print(f"   Status: {status['status']}")
        print(f"   Total containers: {status['total_containers']}")
        
        all_healthy = True
        for service_name, service_info in status['services'].items():
            if service_info['unhealthy'] > 0:
                all_healthy = False
                
        if all_healthy:
            print("   🎉 All services are healthy!")
        else:
            print("   ⚠️  Some services may have health issues")
            
    except Exception as e:
        print(f"❌ Failed to get final status: {e}")
    
    print(f"\n🌐 Application is available at http://localhost:8080")
    print("Press Enter to clean up and exit...")
    input()
    
    # Clean up
    print("\n🧹 Cleaning up stack...")
    try:
        stack.down()
        print("✅ Stack cleaned up successfully!")
    except Exception as e:
        print(f"❌ Cleanup failed: {e}")
    
    print("\n🎉 Stack deployment example completed!")

if __name__ == "__main__":
    main()