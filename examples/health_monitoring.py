#!/usr/bin/env python3
"""
Health Monitoring Example

Demonstrates container health checks, monitoring, and alerting patterns with docker-pyo3.
"""

from docker_pyo3 import Docker
import time
import threading
import json
from datetime import datetime

class HealthMonitor:
    """Centralized health monitoring system"""
    
    def __init__(self, docker):
        self.docker = docker
        self.monitoring = False
        self.health_data = {}
        self.alerts = []
        
    def start_monitoring(self, containers, interval=10):
        """Start monitoring specified containers"""
        self.monitoring = True
        self.containers = containers
        
        def monitor_loop():
            while self.monitoring:
                for container_name in self.containers:
                    try:
                        health_status = self.check_container_health(container_name)
                        self.health_data[container_name] = health_status
                        
                        # Check for alerts
                        self.check_alerts(container_name, health_status)
                        
                    except Exception as e:
                        self.health_data[container_name] = {
                            'status': 'error',
                            'error': str(e),
                            'timestamp': datetime.now().isoformat()
                        }
                
                time.sleep(interval)
        
        self.monitor_thread = threading.Thread(target=monitor_loop, daemon=True)
        self.monitor_thread.start()
        print(f"🔍 Health monitoring started for {len(containers)} containers")
    
    def stop_monitoring(self):
        """Stop health monitoring"""
        self.monitoring = False
        if hasattr(self, 'monitor_thread'):
            self.monitor_thread.join(timeout=5)
        print("⏹️  Health monitoring stopped")
    
    def check_container_health(self, container_name):
        """Check health of a specific container"""
        try:
            container = self.docker.containers().get(container_name)
            info = container.inspect()
            
            state = info['State']
            config = info['Config']
            
            # Basic health info
            health_status = {
                'container_name': container_name,
                'status': state['Status'],
                'running': state['Running'],
                'pid': state.get('Pid', 0),
                'started_at': state.get('StartedAt'),
                'timestamp': datetime.now().isoformat()
            }
            
            # Health check results (if available)
            if 'Health' in state:
                health = state['Health']
                health_status['health_status'] = health['Status']
                health_status['failing_streak'] = health['FailingStreak']
                if health.get('Log'):
                    latest_check = health['Log'][-1]
                    health_status['last_check'] = {
                        'exit_code': latest_check['ExitCode'],
                        'output': latest_check['Output'],
                        'start': latest_check['Start'],
                        'end': latest_check['End']
                    }
            
            # Resource usage (approximate)
            try:
                processes = container.top()
                health_status['process_count'] = len(processes.get('Processes', []))
            except:
                health_status['process_count'] = 'unknown'
            
            # Port availability
            if config.get('ExposedPorts'):
                health_status['exposed_ports'] = list(config['ExposedPorts'].keys())
            
            # Memory and restart info
            if 'RestartCount' in state:
                health_status['restart_count'] = state['RestartCount']
            
            return health_status
            
        except Exception as e:
            return {
                'container_name': container_name,
                'status': 'not_found',
                'error': str(e),
                'timestamp': datetime.now().isoformat()
            }
    
    def check_alerts(self, container_name, health_status):
        """Check for alert conditions"""
        alerts = []
        
        # Container not running
        if not health_status.get('running', False):
            alerts.append({
                'severity': 'critical',
                'message': f"Container {container_name} is not running",
                'container': container_name,
                'timestamp': datetime.now().isoformat()
            })
        
        # Health check failing
        if health_status.get('health_status') == 'unhealthy':
            alerts.append({
                'severity': 'high',
                'message': f"Container {container_name} is unhealthy",
                'container': container_name,
                'timestamp': datetime.now().isoformat()
            })
        
        # High restart count
        restart_count = health_status.get('restart_count', 0)
        if restart_count > 3:
            alerts.append({
                'severity': 'medium',
                'message': f"Container {container_name} has restarted {restart_count} times",
                'container': container_name,
                'timestamp': datetime.now().isoformat()
            })
        
        # Add new alerts
        for alert in alerts:
            # Avoid duplicate alerts
            if not any(a['message'] == alert['message'] and 
                      a['container'] == alert['container'] for a in self.alerts[-10:]):
                self.alerts.append(alert)
                print(f"🚨 ALERT [{alert['severity'].upper()}]: {alert['message']}")
    
    def get_health_summary(self):
        """Get current health summary"""
        summary = {
            'total_containers': len(self.health_data),
            'healthy': 0,
            'unhealthy': 0,
            'stopped': 0,
            'unknown': 0,
            'total_alerts': len(self.alerts),
            'timestamp': datetime.now().isoformat()
        }
        
        for container_name, health in self.health_data.items():
            if health.get('running') and health.get('status') == 'running':
                if health.get('health_status') == 'healthy' or 'health_status' not in health:
                    summary['healthy'] += 1
                else:
                    summary['unhealthy'] += 1
            elif health.get('status') == 'exited':
                summary['stopped'] += 1
            else:
                summary['unknown'] += 1
        
        return summary

def create_app_with_health_check(docker, name, port, health_endpoint="/health"):
    """Create application container with health check"""
    print(f"🏥 Creating {name} with health check...")
    
    try:
        container = docker.containers().create(
            image="nginx:alpine",
            name=name,
            ports={f"80": port},
            labels={
                "monitoring": "enabled",
                "health_endpoint": health_endpoint
            },
            # Add health check configuration
            healthcheck={
                "test": ["CMD", "wget", "--quiet", "--tries=1", "--spider", f"http://localhost{health_endpoint}"],
                "interval": "30s",
                "timeout": "10s",
                "retries": 3,
                "start_period": "10s"
            }
        )
        
        container.start()
        print(f"   ✅ {name} started with health monitoring")
        return container
        
    except Exception as e:
        print(f"   ❌ Failed to create {name}: {e}")
        return None

def create_unhealthy_app(docker, name, port):
    """Create an intentionally unhealthy application for demonstration"""
    print(f"🤒 Creating intentionally unhealthy app {name}...")
    
    try:
        container = docker.containers().create(
            image="busybox",
            name=name,
            ports={f"8080": port},
            command=["sh", "-c", "echo 'Starting unhealthy app' && sleep 30 && exit 1"],
            labels={
                "monitoring": "enabled",
                "test_case": "unhealthy"
            },
            healthcheck={
                "test": ["CMD", "false"],  # Always fails
                "interval": "10s",
                "timeout": "5s",
                "retries": 2,
                "start_period": "5s"
            }
        )
        
        container.start()
        print(f"   ✅ {name} started (will become unhealthy)")
        return container
        
    except Exception as e:
        print(f"   ❌ Failed to create {name}: {e}")
        return None

def demonstrate_custom_health_checks(docker):
    """Demonstrate custom health check implementations"""
    print("🩺 Demonstrating custom health check patterns...")
    
    containers = []
    
    try:
        # Web application with HTTP health check
        web_app = create_app_with_health_check(docker, "web-app", "8080", "/health")
        if web_app:
            containers.append("web-app")
        
        # API service with different health endpoint
        api_app = create_app_with_health_check(docker, "api-service", "8081", "/api/health")
        if api_app:
            containers.append("api-service")
        
        # Database with connection test
        print("   🗄️  Creating database with connection health check...")
        db_container = docker.containers().create(
            image="redis:7-alpine",
            name="health-db",
            ports={"6379": "6379"},
            labels={"monitoring": "enabled"},
            healthcheck={
                "test": ["CMD", "redis-cli", "ping"],
                "interval": "20s",
                "timeout": "3s",
                "retries": 3
            }
        )
        db_container.start()
        containers.append("health-db")
        print("   ✅ Database started with connection health check")
        
        # Intentionally unhealthy app
        unhealthy_app = create_unhealthy_app(docker, "unhealthy-app", "8082")
        if unhealthy_app:
            containers.append("unhealthy-app")
        
        return containers
        
    except Exception as e:
        print(f"   ❌ Custom health check demo failed: {e}")
        return containers

def demonstrate_monitoring_dashboard(monitor):
    """Display a simple monitoring dashboard"""
    print("\n📊 Health Monitoring Dashboard")
    print("=" * 50)
    
    summary = monitor.get_health_summary()
    
    print(f"🔍 Overall Status:")
    print(f"   Total Containers: {summary['total_containers']}")
    print(f"   Healthy: {summary['healthy']} ✅")
    print(f"   Unhealthy: {summary['unhealthy']} ❌")
    print(f"   Stopped: {summary['stopped']} ⏹️")
    print(f"   Unknown: {summary['unknown']} ❓")
    print(f"   Total Alerts: {summary['total_alerts']} 🚨")
    
    print(f"\n📋 Container Details:")
    for container_name, health in monitor.health_data.items():
        status_icon = "✅" if health.get('running') else "❌"
        health_icon = ""
        
        if 'health_status' in health:
            health_icon = "🟢" if health['health_status'] == 'healthy' else "🔴"
        
        print(f"   {status_icon} {container_name} {health_icon}")
        print(f"     Status: {health.get('status', 'unknown')}")
        print(f"     Running: {health.get('running', False)}")
        print(f"     Restarts: {health.get('restart_count', 0)}")
        
        if 'health_status' in health:
            print(f"     Health: {health['health_status']}")
            print(f"     Failing Streak: {health.get('failing_streak', 0)}")
        
        if 'process_count' in health:
            print(f"     Processes: {health['process_count']}")
    
    # Show recent alerts
    if monitor.alerts:
        print(f"\n🚨 Recent Alerts:")
        for alert in monitor.alerts[-5:]:  # Show last 5 alerts
            severity_icon = {"critical": "🔴", "high": "🟠", "medium": "🟡", "low": "🟢"}.get(alert['severity'], "⚪")
            print(f"   {severity_icon} [{alert['severity'].upper()}] {alert['message']}")
            print(f"     Time: {alert['timestamp']}")

def simulate_health_issues(docker, containers):
    """Simulate various health issues for demonstration"""
    print("\n🎭 Simulating health issues for demonstration...")
    
    try:
        # Stop one container to trigger alerts
        if len(containers) > 1:
            victim_container = docker.containers().get(containers[1])
            victim_container.stop()
            print(f"   ⏹️  Stopped {containers[1]} to simulate failure")
        
        # Wait for monitoring to detect
        time.sleep(15)
        
    except Exception as e:
        print(f"   ❌ Simulation failed: {e}")

def cleanup_monitoring_demo(docker, containers):
    """Clean up monitoring demonstration"""
    print("\n🧹 Cleaning up monitoring demo...")
    
    for container_name in containers:
        try:
            container = docker.containers().get(container_name)
            container.stop()
            container.remove(force=True)
            print(f"   ✅ Removed {container_name}")
        except:
            pass

def main():
    docker = Docker()
    monitor = HealthMonitor(docker)
    
    print("🏥 Docker-PyO3 Health Monitoring Example")
    print("=" * 50)
    
    containers = []
    
    try:
        # Create applications with health checks
        containers = demonstrate_custom_health_checks(docker)
        
        if not containers:
            print("❌ No containers created for monitoring")
            return
        
        # Start health monitoring
        monitor.start_monitoring(containers, interval=5)
        
        # Wait for initial health checks
        print("\n⏱️  Waiting for initial health checks...")
        time.sleep(20)
        
        # Show initial dashboard
        demonstrate_monitoring_dashboard(monitor)
        
        # Simulate health issues
        simulate_health_issues(docker, containers)
        
        # Wait for alerts to trigger
        print("\n⏱️  Waiting for health monitoring to detect issues...")
        time.sleep(20)
        
        # Show updated dashboard
        print("\n📊 Updated Health Dashboard After Simulation:")
        print("=" * 50)
        demonstrate_monitoring_dashboard(monitor)
        
        print(f"\n🎉 Health monitoring demonstration completed!")
        print(f"📊 Monitoring Features Demonstrated:")
        print(f"   ✅ Container health checks")
        print(f"   ✅ Automated health monitoring")
        print(f"   ✅ Alert generation")
        print(f"   ✅ Health status dashboard")
        print(f"   ✅ Failure simulation and detection")
        
        print(f"\nPress Enter to clean up...")
        input()
        
    except Exception as e:
        print(f"❌ Health monitoring demo failed: {e}")
    
    finally:
        # Stop monitoring and cleanup
        monitor.stop_monitoring()
        cleanup_monitoring_demo(docker, containers)
    
    print(f"\n🎉 Health monitoring example completed!")

if __name__ == "__main__":
    main()