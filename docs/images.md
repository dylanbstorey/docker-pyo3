# Image Management

Image management provides comprehensive Docker image operations including pulling, building, pushing, and registry authentication.

## Basic Image Operations

### Pulling Images

```python
from docker_pyo3 import Docker

docker = Docker()

# Pull latest tag
image = docker.images().pull("nginx")

# Pull specific tag
image = docker.images().pull("nginx:alpine")

# Pull from specific registry
image = docker.images().pull("registry.example.com/myapp:v1.0")
```

### Image Listing and Inspection

```python
# List all images
images = docker.images().list()
for img in images:
    print(f"Image: {img['RepoTags']}")
    print(f"ID: {img['Id']}")
    print(f"Size: {img['Size']} bytes")

# Get specific image
image = docker.images().get("nginx:latest")

# Inspect image details
info = image.inspect()
print(f"Created: {info['Created']}")
print(f"Architecture: {info['Architecture']}")
print(f"OS: {info['Os']}")
```

## Building Images

### Build from Dockerfile

```python
# Build image from current directory
image = docker.images().build(
    path=".",
    tag="myapp:latest"
)

# Build with custom Dockerfile
image = docker.images().build(
    path="./docker",
    dockerfile="Dockerfile.prod",
    tag="myapp:production"
)

# Build with build arguments
image = docker.images().build(
    path=".",
    tag="myapp:latest",
    buildargs={
        "NODE_VERSION": "18",
        "ENV": "production"
    }
)

# Build with labels
image = docker.images().build(
    path=".",
    tag="myapp:latest",
    labels={
        "maintainer": "team@example.com",
        "version": "1.0.0",
        "description": "My application"
    }
)
```

### Advanced Build Options

```python
# Build with specific target stage
image = docker.images().build(
    path=".",
    tag="myapp:dev",
    target="development"
)

# Build without cache
image = docker.images().build(
    path=".",
    tag="myapp:latest", 
    nocache=True
)

# Build and remove intermediate containers
image = docker.images().build(
    path=".",
    tag="myapp:latest",
    rm=True,
    forcerm=True
)
```

## Registry Authentication

### Password Authentication

```python
# Pull from private registry with credentials
image = docker.images().pull(
    "private-registry.com/myapp:latest",
    username="myusername",
    password="mypassword"
)

# Build and push with authentication
image = docker.images().build(
    path=".",
    tag="private-registry.com/myapp:latest"
)

# Push to private registry
image.push(
    repository="private-registry.com/myapp",
    tag="latest",
    username="myusername", 
    password="mypassword"
)
```

### Token Authentication

```python
# Use registry token
image = docker.images().pull(
    "registry.example.com/myapp:latest",
    registry_token="your-registry-token"
)

# Push with token
image.push(
    repository="registry.example.com/myapp",
    tag="latest",
    registry_token="your-registry-token"
)
```

## Image Operations

### Tagging Images

```python
# Get existing image
image = docker.images().get("myapp:latest")

# Add additional tags
image.tag("myapp:v1.0")
image.tag("registry.example.com/myapp:latest")
image.tag("registry.example.com/myapp:v1.0")
```

### Image History

```python
# Get image layer history
image = docker.images().get("nginx:latest")
history = image.history()

for layer in history:
    print(f"Layer ID: {layer['Id']}")
    print(f"Created: {layer['Created']}")
    print(f"Size: {layer['Size']} bytes")
    print(f"Command: {layer['CreatedBy']}")
    print("---")
```

### Image Export and Import

```python
# Export image to file
image = docker.images().get("myapp:latest")
image.export("myapp-latest.tar")

# Import image from file (requires Docker CLI for full functionality)
# This is a simplified export - full import requires external tools
```

## Image Management Workflows

### Development Workflow

```python
def build_and_test_image(docker, app_path, tag):
    """Build image and run basic tests"""
    
    print(f"Building image {tag}...")
    
    # Build the image
    image = docker.images().build(
        path=app_path,
        tag=tag,
        buildargs={"ENV": "test"}
    )
    
    print("Image built successfully!")
    
    # Create test container
    test_container = docker.containers().create(
        image=tag,
        name=f"test-{tag.replace(':', '-')}",
        command=["npm", "test"]  # or your test command
    )
    
    try:
        # Run tests
        test_container.start()
        test_container.wait()
        
        # Get test results
        logs = test_container.logs()
        print("Test output:")
        print(logs)
        
        # Check exit code
        info = test_container.inspect()
        exit_code = info['State']['ExitCode']
        
        if exit_code == 0:
            print("✅ Tests passed!")
            return True
        else:
            print("❌ Tests failed!")
            return False
            
    finally:
        # Cleanup test container
        test_container.remove(force=True)

# Usage
success = build_and_test_image(docker, "./myapp", "myapp:test")
if success:
    print("Ready for deployment!")
```

### Multi-Stage Build Example

```dockerfile
# Example Dockerfile for multi-stage build
FROM node:18 AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

FROM node:18-alpine AS runtime
WORKDIR /app
COPY --from=builder /app/node_modules ./node_modules
COPY . .
EXPOSE 3000
CMD ["npm", "start"]
```

```python
# Build multi-stage image
production_image = docker.images().build(
    path=".",
    tag="myapp:production",
    target="runtime"
)

# Build development image (includes dev dependencies)
dev_image = docker.images().build(
    path=".",
    tag="myapp:development", 
    target="builder"
)
```

### CI/CD Pipeline Integration

```python
def ci_cd_pipeline(docker, git_commit, branch):
    """Complete CI/CD pipeline for image management"""
    
    # Determine tag based on branch
    if branch == "main":
        tag = f"myapp:{git_commit[:8]}"
        latest_tag = "myapp:latest"
    else:
        tag = f"myapp:{branch}-{git_commit[:8]}"
        latest_tag = None
    
    print(f"Building image for commit {git_commit} on branch {branch}")
    
    # Build image
    image = docker.images().build(
        path=".",
        tag=tag,
        buildargs={
            "GIT_COMMIT": git_commit,
            "BUILD_DATE": str(datetime.now().isoformat())
        },
        labels={
            "git.commit": git_commit,
            "git.branch": branch,
            "build.date": str(datetime.now().isoformat())
        }
    )
    
    # Tag as latest for main branch
    if latest_tag:
        image.tag(latest_tag)
    
    # Run security scan (example)
    print("Running security scan...")
    scan_container = docker.containers().create(
        image="aquasec/trivy:latest",
        command=["image", tag],
        volumes=["/var/run/docker.sock:/var/run/docker.sock:ro"]
    )
    
    try:
        scan_container.start()
        scan_container.wait()
        scan_logs = scan_container.logs()
        
        # Check for critical vulnerabilities
        if "CRITICAL" in scan_logs:
            print("❌ Critical vulnerabilities found!")
            return False
        else:
            print("✅ Security scan passed!")
            
    finally:
        scan_container.remove(force=True)
    
    # Push to registry
    if branch == "main":
        print("Pushing to production registry...")
        image.push(
            repository="prod-registry.com/myapp",
            tag=git_commit[:8],
            username="ci-user",
            password="ci-password"
        )
        
        if latest_tag:
            image.push(
                repository="prod-registry.com/myapp", 
                tag="latest",
                username="ci-user",
                password="ci-password"
            )
    
    print(f"✅ Pipeline completed for {tag}")
    return True

# Usage
pipeline_success = ci_cd_pipeline(docker, "abc123def", "main")
```

## Image Cleanup and Maintenance

### Pruning Unused Images

```python
# Remove unused images
pruned = docker.images().prune()
print(f"Freed space: {pruned['SpaceReclaimed']} bytes")
print(f"Deleted images: {len(pruned['ImagesDeleted'])}")

# Remove all unused images (including tagged)
pruned = docker.images().prune(all=True)
```

### Selective Image Cleanup

```python
def cleanup_old_images(docker, keep_tags=5):
    """Keep only the most recent N tags of each repository"""
    
    images = docker.images().list()
    
    # Group images by repository
    repos = {}
    for img in images:
        repo_tags = img.get('RepoTags', [])
        for tag in repo_tags:
            if ':' in tag:
                repo, tag_name = tag.split(':', 1)
                if repo not in repos:
                    repos[repo] = []
                repos[repo].append({
                    'tag': tag,
                    'id': img['Id'],
                    'created': img['Created']
                })
    
    # Sort and cleanup each repository
    for repo, tags in repos.items():
        # Sort by creation date (newest first)
        tags.sort(key=lambda x: x['created'], reverse=True)
        
        if len(tags) > keep_tags:
            to_delete = tags[keep_tags:]
            print(f"Cleaning up {len(to_delete)} old images from {repo}")
            
            for img_info in to_delete:
                try:
                    image = docker.images().get(img_info['id'])
                    image.delete()
                    print(f"  Deleted {img_info['tag']}")
                except Exception as e:
                    print(f"  Failed to delete {img_info['tag']}: {e}")

# Usage
cleanup_old_images(docker, keep_tags=3)
```

## Error Handling and Best Practices

### Robust Image Operations

```python
def safe_image_pull(docker, image_name, max_retries=3):
    """Pull image with retry logic"""
    
    for attempt in range(max_retries):
        try:
            print(f"Pulling {image_name} (attempt {attempt + 1}/{max_retries})")
            image = docker.images().pull(image_name)
            print(f"Successfully pulled {image_name}")
            return image
            
        except Exception as e:
            print(f"Failed to pull {image_name}: {e}")
            if attempt == max_retries - 1:
                print(f"Max retries exceeded for {image_name}")
                raise
            else:
                print("Retrying in 5 seconds...")
                time.sleep(5)

# Usage
image = safe_image_pull(docker, "nginx:latest")
```

### Image Size Optimization

```python
def analyze_image_size(docker, image_name):
    """Analyze image layers for size optimization"""
    
    image = docker.images().get(image_name)
    history = image.history()
    
    print(f"Image: {image_name}")
    print("Layer analysis:")
    
    total_size = 0
    for i, layer in enumerate(history):
        size = layer['Size']
        total_size += size
        command = layer['CreatedBy'][:60] + "..." if len(layer['CreatedBy']) > 60 else layer['CreatedBy']
        
        print(f"  Layer {i}: {size:>10} bytes - {command}")
    
    print(f"Total size: {total_size:,} bytes ({total_size / (1024*1024):.1f} MB)")
    
    # Find largest layers
    large_layers = sorted(history, key=lambda x: x['Size'], reverse=True)[:3]
    print("\nLargest layers:")
    for layer in large_layers:
        size_mb = layer['Size'] / (1024*1024)
        print(f"  {size_mb:.1f} MB - {layer['CreatedBy'][:100]}")

# Usage
analyze_image_size(docker, "myapp:latest")
```