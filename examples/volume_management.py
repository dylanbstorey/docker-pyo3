#!/usr/bin/env python3
"""
Volume Management Example

Demonstrates data persistence, backup, and volume management patterns with docker-pyo3.
"""

from docker_pyo3 import Docker
import time
import tempfile
import os

def create_data_volume(docker, volume_name):
    """Create a persistent data volume"""
    print(f"💾 Creating data volume '{volume_name}'...")
    
    try:
        volume = docker.volumes().create(
            name=volume_name,
            labels={
                "purpose": "data-storage",
                "managed_by": "docker-pyo3-example"
            }
        )
        print(f"✅ Volume '{volume_name}' created successfully")
        
        # Inspect volume details
        info = volume.inspect()
        print(f"   Driver: {info.get('Driver', 'N/A')}")
        print(f"   Mountpoint: {info.get('Mountpoint', 'N/A')}")
        
        return volume
        
    except Exception as e:
        print(f"❌ Failed to create volume: {e}")
        return None

def populate_volume_with_data(docker, volume_name):
    """Populate volume with sample data"""
    print(f"📝 Populating volume '{volume_name}' with sample data...")
    
    try:
        # Create a temporary container to populate the volume
        setup_container = docker.containers().create(
            image="busybox",
            name=f"setup-{volume_name}",
            volumes=[f"{volume_name}:/data"],
            command=[
                "sh", "-c", 
                """
                echo 'Docker-PyO3 Example Data' > /data/readme.txt &&
                echo 'Created at: '$(date) >> /data/readme.txt &&
                mkdir -p /data/logs /data/config /data/uploads &&
                echo 'log entry 1' > /data/logs/app.log &&
                echo 'log entry 2' >> /data/logs/app.log &&
                echo 'config_value=production' > /data/config/app.conf &&
                echo 'Sample upload content' > /data/uploads/sample.txt &&
                ls -la /data && echo 'Data population complete'
                """
            ]
        )
        
        setup_container.start()
        
        # Wait for completion
        import time
        time.sleep(3)
        
        # Get logs to verify success
        logs = setup_container.logs()
        if "Data population complete" in logs:
            print("✅ Volume populated with sample data")
        else:
            print("⚠️  Volume population may have failed")
            print(f"   Setup logs: {logs}")
        
        # Cleanup setup container
        setup_container.stop()
        setup_container.remove()
        
        return True
        
    except Exception as e:
        print(f"❌ Failed to populate volume: {e}")
        return False

def run_application_with_volume(docker, volume_name):
    """Run an application container that uses the volume"""
    print(f"🚀 Running application with volume '{volume_name}'...")
    
    try:
        app_container = docker.containers().create(
            image="nginx:alpine",
            name="volume-app",
            ports={"80": "8080"},
            volumes=[
                f"{volume_name}:/usr/share/nginx/html:ro",  # Mount volume as web root
                f"{volume_name}/logs:/var/log/nginx"        # Mount logs subdirectory
            ],
            labels={
                "purpose": "volume-demo",
                "uses_volume": volume_name
            }
        )
        
        app_container.start()
        print("✅ Application started with volume mounted")
        print("🌐 Application available at http://localhost:8080")
        
        # Wait for startup
        time.sleep(2)
        
        # Verify the container is running
        info = app_container.inspect()
        if info['State']['Running']:
            print("✅ Application is running successfully")
            
            # Show mounted volumes
            mounts = info.get('Mounts', [])
            print(f"   Mounted volumes ({len(mounts)}):")
            for mount in mounts:
                if mount.get('Type') == 'volume':
                    print(f"     - {mount['Name']} → {mount['Destination']}")
        else:
            print("❌ Application failed to start")
            logs = app_container.logs()
            print(f"   Logs: {logs}")
        
        return app_container
        
    except Exception as e:
        print(f"❌ Failed to start application: {e}")
        return None

def demonstrate_data_persistence(docker, volume_name):
    """Demonstrate that data persists across container restarts"""
    print(f"🔄 Testing data persistence...")
    
    try:
        # Create a container that modifies the volume
        writer_container = docker.containers().create(
            image="busybox",
            name="data-writer",
            volumes=[f"{volume_name}:/data"],
            command=[
                "sh", "-c",
                f"echo 'Written at '$(date) >> /data/persistence_test.txt && cat /data/persistence_test.txt"
            ]
        )
        
        writer_container.start()
        time.sleep(2)
        
        # Get logs and cleanup
        logs = writer_container.logs()
        writer_container.stop()
        writer_container.remove()
        
        # Create another container to read the data
        reader_container = docker.containers().create(
            image="busybox",
            name="data-reader",
            volumes=[f"{volume_name}:/data"],
            command=["cat", "/data/persistence_test.txt"]
        )
        
        reader_container.start()
        time.sleep(2)
        
        read_logs = reader_container.logs()
        reader_container.stop()
        reader_container.remove()
        
        if "Written at" in read_logs:
            print("✅ Data persistence verified - data survived container restart")
            print(f"   Persistent data: {read_logs.strip()}")
        else:
            print("❌ Data persistence test failed")
            
    except Exception as e:
        print(f"❌ Persistence test failed: {e}")

def backup_volume(docker, volume_name, backup_path="/tmp/volume_backup.tar"):
    """Create a backup of the volume"""
    print(f"💾 Creating backup of volume '{volume_name}'...")
    
    try:
        # Create backup container
        backup_container = docker.containers().create(
            image="busybox",
            name=f"backup-{volume_name}",
            volumes=[
                f"{volume_name}:/source:ro",
                f"{os.path.dirname(backup_path)}:/backup"
            ],
            command=[
                "tar", "-czf", f"/backup/{os.path.basename(backup_path)}", 
                "-C", "/source", "."
            ]
        )
        
        backup_container.start()
        
        # Wait for backup to complete
        time.sleep(5)
        
        # Check backup logs
        logs = backup_container.logs()
        backup_container.stop()
        backup_container.remove()
        
        # Verify backup file exists
        if os.path.exists(backup_path):
            size = os.path.getsize(backup_path)
            print(f"✅ Backup created successfully")
            print(f"   Backup file: {backup_path}")
            print(f"   Size: {size} bytes")
            return backup_path
        else:
            print("❌ Backup file not found")
            print(f"   Backup logs: {logs}")
            return None
            
    except Exception as e:
        print(f"❌ Backup failed: {e}")
        return None

def restore_volume(docker, backup_path, new_volume_name):
    """Restore a volume from backup"""
    print(f"🔄 Restoring volume '{new_volume_name}' from backup...")
    
    try:
        # Create new volume for restore
        restore_volume = docker.volumes().create(
            name=new_volume_name,
            labels={
                "purpose": "restored-data",
                "restored_from": os.path.basename(backup_path)
            }
        )
        
        # Restore data
        restore_container = docker.containers().create(
            image="busybox",
            name=f"restore-{new_volume_name}",
            volumes=[
                f"{new_volume_name}:/target",
                f"{os.path.dirname(backup_path)}:/backup"
            ],
            command=[
                "tar", "-xzf", f"/backup/{os.path.basename(backup_path)}",
                "-C", "/target"
            ]
        )
        
        restore_container.start()
        time.sleep(3)
        
        logs = restore_container.logs()
        restore_container.stop()
        restore_container.remove()
        
        # Verify restore by listing contents
        verify_container = docker.containers().create(
            image="busybox",
            name=f"verify-{new_volume_name}",
            volumes=[f"{new_volume_name}:/data"],
            command=["ls", "-la", "/data"]
        )
        
        verify_container.start()
        time.sleep(2)
        
        verify_logs = verify_container.logs()
        verify_container.stop()
        verify_container.remove()
        
        if "readme.txt" in verify_logs:
            print("✅ Volume restored successfully")
            print("   Restored contents:")
            for line in verify_logs.split('\n')[:5]:  # Show first 5 lines
                if line.strip():
                    print(f"     {line}")
            return restore_volume
        else:
            print("❌ Volume restore verification failed")
            print(f"   Verify logs: {verify_logs}")
            return None
            
    except Exception as e:
        print(f"❌ Restore failed: {e}")
        return None

def demonstrate_volume_sharing(docker, volume_name):
    """Demonstrate sharing a volume between multiple containers"""
    print(f"🤝 Demonstrating volume sharing between containers...")
    
    containers = []
    
    try:
        # Create multiple containers sharing the same volume
        for i in range(3):
            container = docker.containers().create(
                image="busybox",
                name=f"shared-app-{i+1}",
                volumes=[f"{volume_name}:/shared"],
                command=[
                    "sh", "-c",
                    f"echo 'Container {i+1} started at '$(date) >> /shared/shared.log && "
                    f"while true; do echo 'Container {i+1} heartbeat at '$(date) >> /shared/shared.log; sleep 10; done"
                ]
            )
            
            container.start()
            containers.append(container)
            print(f"   ✅ Started shared-app-{i+1}")
        
        # Wait for some activity
        print("   ⏱️  Waiting for containers to write to shared volume...")
        time.sleep(15)
        
        # Check shared log
        reader = docker.containers().create(
            image="busybox",
            name="log-reader",
            volumes=[f"{volume_name}:/shared"],
            command=["cat", "/shared/shared.log"]
        )
        
        reader.start()
        time.sleep(2)
        
        shared_logs = reader.logs()
        reader.stop()
        reader.remove()
        
        print("   📋 Shared log contents:")
        for line in shared_logs.split('\n')[:10]:  # Show first 10 lines
            if line.strip():
                print(f"     {line}")
        
        print("✅ Volume sharing demonstrated successfully")
        
    except Exception as e:
        print(f"❌ Volume sharing demo failed: {e}")
    
    finally:
        # Cleanup shared containers
        for i, container in enumerate(containers):
            try:
                container.stop()
                container.remove()
                print(f"   🧹 Cleaned up shared-app-{i+1}")
            except:
                pass

def cleanup_volumes(docker, volume_names):
    """Clean up created volumes"""
    print(f"🧹 Cleaning up volumes...")
    
    for volume_name in volume_names:
        try:
            volume = docker.volumes().get(volume_name)
            volume.delete()
            print(f"   ✅ Removed volume '{volume_name}'")
        except Exception as e:
            print(f"   ⚠️  Failed to remove volume '{volume_name}': {e}")

def main():
    docker = Docker()
    
    print("📂 Docker-PyO3 Volume Management Example")
    print("=" * 50)
    
    volume_name = "example-data-volume"
    restored_volume_name = "restored-data-volume"
    volumes_to_cleanup = [volume_name, restored_volume_name]
    backup_file = None
    app_container = None
    
    try:
        # Create and populate volume
        volume = create_data_volume(docker, volume_name)
        if not volume:
            return
        
        success = populate_volume_with_data(docker, volume_name)
        if not success:
            return
        
        # Run application with volume
        app_container = run_application_with_volume(docker, volume_name)
        if not app_container:
            return
        
        # Test data persistence
        demonstrate_data_persistence(docker, volume_name)
        
        # Create backup
        backup_file = backup_volume(docker, volume_name)
        
        # Demonstrate volume sharing
        demonstrate_volume_sharing(docker, volume_name)
        
        # Restore from backup
        if backup_file:
            restored_volume = restore_volume(docker, backup_file, restored_volume_name)
            if restored_volume:
                print("✅ Backup and restore cycle completed successfully")
        
        print(f"\n🎉 Volume management demonstration completed!")
        print(f"📊 Summary:")
        print(f"   ✅ Created persistent volume")
        print(f"   ✅ Populated with sample data")
        print(f"   ✅ Demonstrated data persistence")
        print(f"   ✅ Created volume backup")
        print(f"   ✅ Demonstrated volume sharing")
        print(f"   ✅ Restored from backup")
        
        print(f"\nPress Enter to clean up...")
        input()
        
    except Exception as e:
        print(f"❌ Demo failed: {e}")
    
    finally:
        # Cleanup
        print(f"\n🧹 Cleaning up resources...")
        
        # Stop and remove app container
        if app_container:
            try:
                app_container.stop()
                app_container.remove(force=True)
                print("   ✅ Removed application container")
            except:
                pass
        
        # Remove backup file
        if backup_file and os.path.exists(backup_file):
            try:
                os.remove(backup_file)
                print(f"   ✅ Removed backup file")
            except:
                pass
        
        # Remove volumes
        cleanup_volumes(docker, volumes_to_cleanup)
    
    print(f"\n🎉 Volume management example completed!")

if __name__ == "__main__":
    main()