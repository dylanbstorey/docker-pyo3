# Volume Management

Volume management provides Docker volume operations for data persistence, sharing, and backup across containers.

## Basic Volume Operations

### Creating Volumes

```python
from docker_pyo3 import Docker

docker = Docker()

# Create basic volume
volume = docker.volumes().create(name="myapp-data")

# Create volume with driver options
volume = docker.volumes().create(
    name="database-data",
    driver="local",
    driver_opts={
        "type": "none",
        "o": "bind",
        "device": "/host/path/data"
    }
)

# Create volume with labels
volume = docker.volumes().create(
    name="app-logs",
    labels={
        "app": "myapp",
        "tier": "logging",
        "backup": "daily"
    }
)
```

### Volume Listing and Inspection

```python
# List all volumes
volumes = docker.volumes().list()
for vol in volumes:
    print(f"Volume: {vol['Name']}")
    print(f"Driver: {vol['Driver']}")
    print(f"Mountpoint: {vol['Mountpoint']}")

# Get specific volume
volume = docker.volumes().get("myapp-data")

# Inspect volume details
info = volume.inspect()
print(f"Created: {info['CreatedAt']}")
print(f"Labels: {info['Labels']}")
print(f"Options: {info['Options']}")
```

## Volume Usage Patterns

### Data Persistence

```python
# Database with persistent data
db_volume = docker.volumes().create(name="postgres-data")

db_container = docker.containers().create(
    image="postgres:13",
    name="database",
    env=[
        "POSTGRES_PASSWORD=secret",
        "POSTGRES_DB=myapp"
    ],
    volumes=["postgres-data:/var/lib/postgresql/data"]
)

db_container.start()

# Application with persistent uploads
uploads_volume = docker.volumes().create(name="app-uploads")

app_container = docker.containers().create(
    image="myapp:latest",
    name="application",
    volumes=[
        "app-uploads:/app/uploads",
        "postgres-data:/backup"  # Shared access to database data
    ]
)

app_container.start()
```

### Configuration Management

```python
# Configuration volume
config_volume = docker.volumes().create(name="app-config")

# Initialize configuration container
config_init = docker.containers().create(
    image="busybox",
    name="config-init",
    volumes=["app-config:/config"],
    command=[
        "sh", "-c",
        "echo 'database_url=postgres://db:5432/myapp' > /config/app.conf && "
        "echo 'log_level=info' >> /config/app.conf"
    ]
)

config_init.start()
config_init.wait()
config_init.remove()

# Use configuration in application
app_container = docker.containers().create(
    image="myapp:latest",
    name="configured-app",
    volumes=["app-config:/etc/myapp:ro"]  # Read-only config
)
```

### Log Management

```python
# Centralized logging setup
logs_volume = docker.volumes().create(name="app-logs")

# Web server with log volume
web_container = docker.containers().create(
    image="nginx:latest",
    name="web-server",
    volumes=["app-logs:/var/log/nginx"],
    ports={"80": "8080"}
)

# API server sharing log volume
api_container = docker.containers().create(
    image="myapi:latest",
    name="api-server", 
    volumes=["app-logs:/app/logs"]
)

# Log aggregator
log_aggregator = docker.containers().create(
    image="fluentd:latest",
    name="log-collector",
    volumes=["app-logs:/fluentd/log:ro"]  # Read-only access
)

# Start all containers
for container in [web_container, api_container, log_aggregator]:
    container.start()
```

## Advanced Volume Scenarios

### Backup and Restore

```python
def backup_volume(docker, volume_name, backup_path):
    """Backup volume data to host filesystem"""
    
    # Create backup container
    backup_container = docker.containers().create(
        image="busybox",
        name=f"backup-{volume_name}",
        volumes=[
            f"{volume_name}:/data:ro",  # Source volume (read-only)
            f"{backup_path}:/backup"    # Backup destination
        ],
        command=[
            "tar", "czf", f"/backup/{volume_name}-backup.tar.gz", "/data"
        ]
    )
    
    try:
        backup_container.start()
        backup_container.wait()
        
        # Check if backup was successful
        info = backup_container.inspect()
        exit_code = info['State']['ExitCode']
        
        if exit_code == 0:
            print(f"✅ Backup of {volume_name} completed successfully")
            return True
        else:
            print(f"❌ Backup of {volume_name} failed")
            return False
            
    finally:
        backup_container.remove(force=True)

def restore_volume(docker, volume_name, backup_path):
    """Restore volume data from backup"""
    
    # Ensure volume exists
    try:
        docker.volumes().get(volume_name)
    except:
        docker.volumes().create(name=volume_name)
    
    # Create restore container
    restore_container = docker.containers().create(
        image="busybox",
        name=f"restore-{volume_name}",
        volumes=[
            f"{volume_name}:/data",      # Target volume
            f"{backup_path}:/backup:ro"  # Backup source (read-only)
        ],
        command=[
            "sh", "-c",
            f"cd /data && tar xzf /backup/{volume_name}-backup.tar.gz --strip-components=1"
        ]
    )
    
    try:
        restore_container.start()
        restore_container.wait()
        
        info = restore_container.inspect()
        exit_code = info['State']['ExitCode']
        
        if exit_code == 0:
            print(f"✅ Restore of {volume_name} completed successfully")
            return True
        else:
            print(f"❌ Restore of {volume_name} failed")
            return False
            
    finally:
        restore_container.remove(force=True)

# Usage
backup_volume(docker, "postgres-data", "/host/backups")
restore_volume(docker, "postgres-data", "/host/backups")
```

### Volume Migration

```python
def migrate_volume_data(docker, source_volume, target_volume):
    """Migrate data from one volume to another"""
    
    # Ensure target volume exists
    try:
        docker.volumes().get(target_volume)
    except:
        docker.volumes().create(name=target_volume)
    
    # Create migration container
    migration_container = docker.containers().create(
        image="busybox",
        name=f"migrate-{source_volume}-to-{target_volume}",
        volumes=[
            f"{source_volume}:/source:ro",  # Source (read-only)
            f"{target_volume}:/target"      # Target (read-write)
        ],
        command=["cp", "-a", "/source/.", "/target/"]
    )
    
    try:
        migration_container.start()
        migration_container.wait()
        
        info = migration_container.inspect()
        exit_code = info['State']['ExitCode']
        
        if exit_code == 0:
            print(f"✅ Migration from {source_volume} to {target_volume} completed")
            return True
        else:
            print(f"❌ Migration failed")
            return False
            
    finally:
        migration_container.remove(force=True)

# Example: Migrate from old to new volume
migrate_volume_data(docker, "old-postgres-data", "new-postgres-data")
```

### Development Volume Workflows

```python
def setup_development_volumes(docker, project_path):
    """Setup development environment with bind mounts and volumes"""
    
    # Create volumes for persistent data
    db_volume = docker.volumes().create(name="dev-postgres")
    cache_volume = docker.volumes().create(name="dev-redis")
    
    # Database with persistent data
    db_container = docker.containers().create(
        image="postgres:13",
        name="dev-postgres",
        env=[
            "POSTGRES_PASSWORD=dev",
            "POSTGRES_DB=myapp_dev"
        ],
        volumes=["dev-postgres:/var/lib/postgresql/data"],
        ports={"5432": "5432"}
    )
    
    # Redis cache
    cache_container = docker.containers().create(
        image="redis:7",
        name="dev-redis",
        volumes=["dev-redis:/data"],
        ports={"6379": "6379"}
    )
    
    # Application with source code bind mount
    app_container = docker.containers().create(
        image="node:18",
        name="dev-app",
        volumes=[
            f"{project_path}/src:/app/src",      # Source code (live reload)
            f"{project_path}/package.json:/app/package.json:ro",  # Dependencies
            "dev-node-modules:/app/node_modules"  # Node modules volume
        ],
        working_dir="/app",
        command=["npm", "run", "dev"],
        ports={"3000": "3000"},
        env=[
            "NODE_ENV=development",
            "DATABASE_URL=postgresql://postgres:dev@dev-postgres:5432/myapp_dev",
            "REDIS_URL=redis://dev-redis:6379"
        ]
    )
    
    # Start development environment
    db_container.start()
    cache_container.start()
    
    # Wait for database to be ready
    import time
    time.sleep(5)
    
    app_container.start()
    
    print("Development environment started:")
    print("- Database: localhost:5432")
    print("- Redis: localhost:6379") 
    print("- App: localhost:3000")
    print("- Source code is bind-mounted for live reload")

# Usage
setup_development_volumes(docker, "/path/to/project")
```

## Volume Monitoring and Maintenance

### Volume Usage Analysis

```python
def analyze_volume_usage(docker):
    """Analyze volume usage and identify large volumes"""
    
    volumes = docker.volumes().list()
    
    print("Volume Usage Analysis:")
    print("=" * 50)
    
    total_volumes = len(volumes)
    print(f"Total volumes: {total_volumes}")
    
    # Analyze each volume
    volume_info = []
    
    for vol in volumes:
        name = vol['Name']
        driver = vol['Driver']
        mountpoint = vol['Mountpoint']
        
        # Get volume size (requires host filesystem access)
        try:
            # Create container to check volume size
            size_check = docker.containers().create(
                image="busybox",
                name=f"size-check-{name[:20]}",
                volumes=[f"{name}:/volume"],
                command=["du", "-sh", "/volume"]
            )
            
            size_check.start()
            size_check.wait()
            
            logs = size_check.logs()
            size = logs.split()[0] if logs.strip() else "Unknown"
            
            size_check.remove(force=True)
            
        except Exception:
            size = "Unknown"
        
        volume_info.append({
            'name': name,
            'driver': driver,
            'size': size,
            'mountpoint': mountpoint
        })
    
    # Sort by name
    volume_info.sort(key=lambda x: x['name'])
    
    # Display results
    for vol in volume_info:
        print(f"\nVolume: {vol['name']}")
        print(f"  Driver: {vol['driver']}")
        print(f"  Size: {vol['size']}")
        print(f"  Mountpoint: {vol['mountpoint']}")

# Usage
analyze_volume_usage(docker)
```

### Volume Cleanup

```python
def cleanup_unused_volumes(docker, dry_run=True):
    """Clean up unused volumes"""
    
    volumes = docker.volumes().list()
    
    # Get list of volumes in use
    containers = docker.containers().list(all=True)
    volumes_in_use = set()
    
    for container in containers:
        mounts = container.get('Mounts', [])
        for mount in mounts:
            if mount['Type'] == 'volume':
                volumes_in_use.add(mount['Name'])
    
    # Find unused volumes
    unused_volumes = []
    for vol in volumes:
        if vol['Name'] not in volumes_in_use:
            unused_volumes.append(vol)
    
    print(f"Found {len(unused_volumes)} unused volumes")
    
    if not unused_volumes:
        print("No unused volumes to clean up")
        return
    
    for vol in unused_volumes:
        name = vol['Name']
        if dry_run:
            print(f"Would remove: {name}")
        else:
            try:
                volume_obj = docker.volumes().get(name)
                volume_obj.delete()
                print(f"Removed: {name}")
            except Exception as e:
                print(f"Failed to remove {name}: {e}")
    
    if dry_run:
        print("\nThis was a dry run. Use dry_run=False to actually remove volumes.")

# Usage
cleanup_unused_volumes(docker, dry_run=True)   # Preview
cleanup_unused_volumes(docker, dry_run=False)  # Actually remove
```

### Volume Backup Automation

```python
import schedule
import time
from datetime import datetime

def automated_volume_backup(docker, volumes_to_backup, backup_base_path):
    """Automated volume backup system"""
    
    def backup_job():
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        for volume_name in volumes_to_backup:
            backup_path = f"{backup_base_path}/{volume_name}"
            
            # Create backup directory if it doesn't exist
            backup_container = docker.containers().create(
                image="busybox",
                name="mkdir-backup",
                volumes=[f"{backup_base_path}:/backups"],
                command=["mkdir", "-p", f"/backups/{volume_name}"]
            )
            backup_container.start()
            backup_container.wait()
            backup_container.remove(force=True)
            
            # Perform backup
            print(f"Backing up {volume_name}...")
            
            backup_container = docker.containers().create(
                image="busybox",
                name=f"backup-{volume_name}-{timestamp}",
                volumes=[
                    f"{volume_name}:/data:ro",
                    f"{backup_path}:/backup"
                ],
                command=[
                    "tar", "czf", 
                    f"/backup/backup-{timestamp}.tar.gz", 
                    "/data"
                ]
            )
            
            try:
                backup_container.start()
                backup_container.wait()
                
                info = backup_container.inspect()
                if info['State']['ExitCode'] == 0:
                    print(f"✅ {volume_name} backed up successfully")
                else:
                    print(f"❌ {volume_name} backup failed")
                    
            finally:
                backup_container.remove(force=True)
            
            # Clean up old backups (keep last 7 days)
            cleanup_container = docker.containers().create(
                image="busybox",
                name=f"cleanup-{volume_name}",
                volumes=[f"{backup_path}:/backup"],
                command=[
                    "find", "/backup", "-name", "backup-*.tar.gz",
                    "-mtime", "+7", "-delete"
                ]
            )
            cleanup_container.start()
            cleanup_container.wait()
            cleanup_container.remove(force=True)
    
    # Schedule backups
    schedule.every().day.at("02:00").do(backup_job)  # Daily at 2 AM
    
    print("Backup scheduler started. Running daily at 2:00 AM")
    print("Volumes to backup:", volumes_to_backup)
    
    # Run scheduler (this would typically run in a separate process)
    while True:
        schedule.run_pending()
        time.sleep(60)  # Check every minute

# Usage (in production, run this in a separate process/container)
# automated_volume_backup(
#     docker, 
#     ["postgres-data", "app-uploads", "redis-data"],
#     "/host/backups"
# )
```

## Volume Best Practices

### Volume Naming Convention

```python
def create_volume_with_metadata(docker, app_name, volume_type, environment="prod"):
    """Create volume with consistent naming and metadata"""
    
    volume_name = f"{app_name}-{volume_type}-{environment}"
    
    volume = docker.volumes().create(
        name=volume_name,
        labels={
            "app": app_name,
            "type": volume_type,
            "environment": environment,
            "created_at": datetime.now().isoformat(),
            "backup_policy": "daily" if volume_type in ["database", "uploads"] else "weekly"
        }
    )
    
    print(f"Created volume: {volume_name}")
    return volume

# Usage
db_volume = create_volume_with_metadata(docker, "myapp", "database", "prod")
uploads_volume = create_volume_with_metadata(docker, "myapp", "uploads", "prod") 
logs_volume = create_volume_with_metadata(docker, "myapp", "logs", "prod")
```

### Volume Security

```python
def create_secure_volume_setup(docker):
    """Create volume setup with security considerations"""
    
    # Sensitive data volume (database, secrets)
    sensitive_volume = docker.volumes().create(
        name="app-sensitive-data",
        labels={
            "security_level": "high",
            "encryption": "required",
            "backup_retention": "90_days"
        }
    )
    
    # Regular application data
    app_volume = docker.volumes().create(
        name="app-data",
        labels={
            "security_level": "medium", 
            "backup_retention": "30_days"
        }
    )
    
    # Temporary/cache data
    cache_volume = docker.volumes().create(
        name="app-cache",
        labels={
            "security_level": "low",
            "backup_retention": "none",
            "ephemeral": "true"
        }
    )
    
    # Database with restricted access
    db_container = docker.containers().create(
        image="postgres:13",
        name="secure-database",
        volumes=["app-sensitive-data:/var/lib/postgresql/data"],
        env=["POSTGRES_PASSWORD=secure_password"],
        user="999:999",  # Non-root user
        # No port mapping - internal access only
    )
    
    # Application with read-only access to sensitive data
    app_container = docker.containers().create(
        image="myapp:latest",
        name="secure-app",
        volumes=[
            "app-sensitive-data:/app/secrets:ro",  # Read-only secrets
            "app-data:/app/data",                  # Read-write app data
            "app-cache:/app/cache"                 # Cache data
        ],
        user="1000:1000"  # Non-root user
    )
    
    print("Secure volume setup created:")
    print("- Sensitive data: Read-only access for app")
    print("- All containers run as non-root users")
    print("- Database has no external ports")

# Usage
create_secure_volume_setup(docker)
```