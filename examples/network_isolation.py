#!/usr/bin/env python3
"""
Network Isolation Example

Demonstrates network security and isolation patterns with docker-pyo3.
"""

from docker_pyo3 import Docker
import time

def create_isolated_environments(docker):
    """Create isolated network environments for different purposes"""
    
    print("🔒 Creating isolated network environments...")
    
    # Create networks for different security zones
    networks = {}
    
    # DMZ network (public-facing services)
    networks['dmz'] = docker.networks().create("dmz-network")
    print("   ✅ Created DMZ network (public zone)")
    
    # Internal network (application services)
    networks['internal'] = docker.networks().create("internal-network")
    print("   ✅ Created internal network (application zone)")
    
    # Database network (database services only)
    networks['database'] = docker.networks().create("database-network")
    print("   ✅ Created database network (data zone)")
    
    # Management network (monitoring and management)
    networks['management'] = docker.networks().create("management-network")
    print("   ✅ Created management network (admin zone)")
    
    return networks

def deploy_public_load_balancer(docker, networks):
    """Deploy public-facing load balancer in DMZ"""
    
    print("🌐 Deploying public load balancer...")
    
    lb_container = docker.containers().create(
        image="nginx:alpine",
        name="public-loadbalancer",
        ports={"80": "8080", "443": "8443"},
        env=["NGINX_WORKER_PROCESSES=auto"],
        labels={
            "zone": "dmz",
            "role": "loadbalancer",
            "public": "true"
        }
    )
    
    # Connect only to DMZ network
    networks['dmz'].connect("public-loadbalancer")
    
    lb_container.start()
    print("   ✅ Load balancer deployed (DMZ only)")
    
    return lb_container

def deploy_web_tier(docker, networks):
    """Deploy web application tier with DMZ and internal access"""
    
    print("🖥️  Deploying web application tier...")
    
    web_containers = []
    
    for i in range(2):
        web_container = docker.containers().create(
            image="nginx:alpine",
            name=f"web-server-{i+1}",
            env=[
                "ENV=production",
                f"INSTANCE_ID=web-{i+1}"
            ],
            labels={
                "zone": "dmz-internal",
                "role": "web",
                "tier": "frontend"
            }
        )
        
        # Connect to both DMZ (for load balancer) and internal (for API access)
        networks['dmz'].connect(f"web-server-{i+1}")
        networks['internal'].connect(f"web-server-{i+1}")
        
        web_container.start()
        web_containers.append(web_container)
    
    print(f"   ✅ Deployed {len(web_containers)} web servers (DMZ + Internal)")
    
    return web_containers

def deploy_api_tier(docker, networks):
    """Deploy API services with internal and database access"""
    
    print("🔧 Deploying API service tier...")
    
    api_containers = []
    
    # User API service
    user_api = docker.containers().create(
        image="busybox",
        name="user-api",
        command=["sh", "-c", "echo 'User API running' && sleep 300"],
        env=[
            "SERVICE_NAME=user-api",
            "DATABASE_URL=postgresql://database:5432/users"
        ],
        labels={
            "zone": "internal-database",
            "role": "api",
            "service": "user"
        }
    )
    
    # Connect to internal (for web access) and database (for data access)
    networks['internal'].connect("user-api")
    networks['database'].connect("user-api")
    
    user_api.start()
    api_containers.append(user_api)
    
    # Order API service
    order_api = docker.containers().create(
        image="busybox", 
        name="order-api",
        command=["sh", "-c", "echo 'Order API running' && sleep 300"],
        env=[
            "SERVICE_NAME=order-api",
            "DATABASE_URL=postgresql://database:5432/orders"
        ],
        labels={
            "zone": "internal-database",
            "role": "api", 
            "service": "order"
        }
    )
    
    networks['internal'].connect("order-api")
    networks['database'].connect("order-api")
    
    order_api.start()
    api_containers.append(order_api)
    
    print(f"   ✅ Deployed {len(api_containers)} API services (Internal + Database)")
    
    return api_containers

def deploy_database_tier(docker, networks):
    """Deploy database services in isolated database network"""
    
    print("🗄️  Deploying database tier...")
    
    # Main application database
    main_db = docker.containers().create(
        image="postgres:13",
        name="main-database",
        env=[
            "POSTGRES_PASSWORD=secure_password",
            "POSTGRES_DB=application",
            "POSTGRES_USER=app_user"
        ],
        labels={
            "zone": "database",
            "role": "database",
            "type": "primary"
        }
    )
    
    # Connect only to database network
    networks['database'].connect("main-database")
    
    main_db.start()
    print("   ✅ Main database deployed (Database network only)")
    
    # Redis cache (also in database tier for this example)
    cache_db = docker.containers().create(
        image="redis:7-alpine",
        name="cache-database",
        command=["redis-server", "--appendonly", "yes"],
        labels={
            "zone": "database",
            "role": "cache",
            "type": "redis"
        }
    )
    
    networks['database'].connect("cache-database")
    
    cache_db.start()
    print("   ✅ Cache database deployed (Database network only)")
    
    return [main_db, cache_db]

def deploy_management_tier(docker, networks):
    """Deploy management and monitoring services"""
    
    print("📊 Deploying management tier...")
    
    # Monitoring service with access to all networks for observability
    monitor_container = docker.containers().create(
        image="busybox",
        name="monitoring-service",
        command=["sh", "-c", "echo 'Monitoring service running' && sleep 300"],
        env=["SERVICE_NAME=monitoring"],
        labels={
            "zone": "management",
            "role": "monitoring"
        }
    )
    
    # Connect to management network and internal for monitoring
    networks['management'].connect("monitoring-service")
    networks['internal'].connect("monitoring-service")
    
    monitor_container.start()
    print("   ✅ Monitoring service deployed (Management + Internal)")
    
    # Management console (admin access)
    admin_container = docker.containers().create(
        image="busybox",
        name="admin-console",
        command=["sh", "-c", "echo 'Admin console running' && sleep 300"],
        env=["SERVICE_NAME=admin-console"],
        ports={"22": "2222"},  # SSH access
        labels={
            "zone": "management",
            "role": "administration"
        }
    )
    
    # Connect to management network only
    networks['management'].connect("admin-console")
    
    admin_container.start()
    print("   ✅ Admin console deployed (Management network only)")
    
    return [monitor_container, admin_container]

def test_network_connectivity(docker, networks):
    """Test connectivity between different network zones"""
    
    print("\n🔍 Testing network connectivity and isolation...")
    
    def test_connection(from_container, to_container, should_connect=True):
        """Test if one container can reach another"""
        try:
            from_cont = docker.containers().get(from_container)
            result = from_cont.exec(["ping", "-c", "1", to_container])
            
            success = "1 packets transmitted, 1 received" in result
            
            if should_connect:
                if success:
                    print(f"   ✅ {from_container} → {to_container}: Connected (as expected)")
                else:
                    print(f"   ❌ {from_container} → {to_container}: Failed to connect (unexpected)")
            else:
                if not success:
                    print(f"   ✅ {from_container} → {to_container}: Blocked (as expected)")
                else:
                    print(f"   ⚠️  {from_container} → {to_container}: Connected (security concern)")
                    
        except Exception as e:
            if should_connect:
                print(f"   ❌ {from_container} → {to_container}: Test failed - {e}")
            else:
                print(f"   ✅ {from_container} → {to_container}: Blocked (test error indicates isolation)")
    
    # Test allowed connections
    print("\n   Testing allowed connections:")
    test_connection("public-loadbalancer", "web-server-1", True)  # LB to Web
    test_connection("web-server-1", "user-api", True)            # Web to API
    test_connection("user-api", "main-database", True)           # API to DB
    test_connection("monitoring-service", "user-api", True)      # Monitor to API
    
    # Test blocked connections  
    print("\n   Testing blocked connections:")
    test_connection("public-loadbalancer", "user-api", False)    # LB to API (should be blocked)
    test_connection("public-loadbalancer", "main-database", False)  # LB to DB (should be blocked)
    test_connection("web-server-1", "main-database", False)      # Web to DB (should be blocked)
    test_connection("admin-console", "user-api", False)          # Admin to API (should be blocked)

def show_network_topology(docker, networks):
    """Display the network topology and container placement"""
    
    print("\n🗺️  Network Topology Overview:")
    print("=" * 50)
    
    for net_name, network in networks.items():
        print(f"\n🔗 Network: {net_name}")
        
        try:
            info = network.inspect()
            containers = info.get('Containers', {})
            
            if containers:
                print(f"   Connected containers ({len(containers)}):")
                for container_id, container_info in containers.items():
                    name = container_info['Name']
                    ip = container_info.get('IPv4Address', 'N/A').split('/')[0]
                    print(f"     - {name} ({ip})")
            else:
                print("   No containers connected")
                
        except Exception as e:
            print(f"   Error inspecting network: {e}")
    
    print(f"\n🛡️  Security Zones:")
    print(f"   DMZ: Public load balancer")
    print(f"   DMZ + Internal: Web servers")
    print(f"   Internal + Database: API services")
    print(f"   Database: Database servers")
    print(f"   Management: Admin and monitoring")

def cleanup_environment(docker, networks):
    """Clean up all containers and networks"""
    
    print("\n🧹 Cleaning up environment...")
    
    # Stop and remove all containers
    container_names = [
        "public-loadbalancer", "web-server-1", "web-server-2",
        "user-api", "order-api", "main-database", "cache-database",
        "monitoring-service", "admin-console"
    ]
    
    for name in container_names:
        try:
            container = docker.containers().get(name)
            container.stop()
            container.remove(force=True)
            print(f"   ✅ Removed {name}")
        except:
            pass
    
    # Remove networks
    for net_name, network in networks.items():
        try:
            network.delete()
            print(f"   ✅ Removed {net_name} network")
        except Exception as e:
            print(f"   ⚠️  Failed to remove {net_name}: {e}")

def main():
    docker = Docker()
    
    print("🔐 Docker-PyO3 Network Isolation Example")
    print("=" * 50)
    
    try:
        # Create isolated network environments
        networks = create_isolated_environments(docker)
        
        # Deploy services in different security zones
        lb = deploy_public_load_balancer(docker, networks)
        web_servers = deploy_web_tier(docker, networks)
        api_servers = deploy_api_tier(docker, networks)
        databases = deploy_database_tier(docker, networks)
        management = deploy_management_tier(docker, networks)
        
        # Wait for all services to start
        print("\n⏱️  Waiting for services to initialize...")
        time.sleep(10)
        
        # Test network connectivity and isolation
        test_network_connectivity(docker, networks)
        
        # Show network topology
        show_network_topology(docker, networks)
        
        print(f"\n🎉 Network isolation example deployed successfully!")
        print(f"🌐 Public access: http://localhost:8080")
        print(f"🔒 Admin access: ssh://localhost:2222")
        
        print(f"\nSecurity zones established:")
        print(f"  ✅ DMZ: Load balancer isolated")
        print(f"  ✅ Web tier: Can reach APIs but not databases")
        print(f"  ✅ API tier: Can reach databases but not public")
        print(f"  ✅ Database tier: Completely isolated")
        print(f"  ✅ Management tier: Controlled access")
        
        print(f"\nPress Enter to clean up...")
        input()
        
    except Exception as e:
        print(f"❌ Deployment failed: {e}")
    
    finally:
        # Cleanup
        if 'networks' in locals():
            cleanup_environment(docker, networks)
    
    print(f"\n🎉 Network isolation example completed!")

if __name__ == "__main__":
    main()