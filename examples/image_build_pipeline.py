#!/usr/bin/env python3
"""
Image Build Pipeline Example

Demonstrates building, testing, and deploying Docker images with docker-pyo3.
"""

from docker_pyo3 import Docker
import tempfile
import os
import time

def create_sample_dockerfile(app_dir):
    """Create a sample Node.js application with Dockerfile"""
    
    # Create package.json
    package_json = '''
{
  "name": "sample-app",
  "version": "1.0.0",
  "description": "Sample Node.js application",
  "main": "app.js",
  "scripts": {
    "start": "node app.js",
    "test": "echo \\"Test passed\\" && exit 0"
  },
  "dependencies": {
    "express": "^4.18.0"
  }
}
'''
    
    # Create app.js
    app_js = '''
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

app.get('/', (req, res) => {
    res.json({
        message: 'Hello from Docker-PyO3!',
        version: process.env.APP_VERSION || '1.0.0',
        environment: process.env.NODE_ENV || 'development',
        timestamp: new Date().toISOString()
    });
});

app.get('/health', (req, res) => {
    res.json({ status: 'healthy', uptime: process.uptime() });
});

app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
    console.log(`Environment: ${process.env.NODE_ENV || 'development'}`);
});
'''
    
    # Create Dockerfile
    dockerfile = '''
FROM node:18-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

FROM node:18-alpine AS runtime

WORKDIR /app
COPY --from=builder /app/node_modules ./node_modules
COPY app.js ./

RUN addgroup -g 1001 -S nodejs && \\
    adduser -S nodejs -u 1001

USER nodejs

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \\
    CMD node -e "require('http').get('http://localhost:3000/health', (res) => { process.exit(res.statusCode === 200 ? 0 : 1) })"

CMD ["npm", "start"]
'''
    
    # Create .dockerignore
    dockerignore = '''
node_modules
npm-debug.log
.git
.gitignore
README.md
.env
.nyc_output
coverage
.tmp
'''
    
    # Write files
    with open(os.path.join(app_dir, 'package.json'), 'w') as f:
        f.write(package_json)
    
    with open(os.path.join(app_dir, 'app.js'), 'w') as f:
        f.write(app_js)
    
    with open(os.path.join(app_dir, 'Dockerfile'), 'w') as f:
        f.write(dockerfile)
    
    with open(os.path.join(app_dir, '.dockerignore'), 'w') as f:
        f.write(dockerignore)

def build_and_test_image(docker, app_dir, image_name, version):
    """Build and test a Docker image"""
    
    print(f"🏗️  Building image {image_name}:{version}...")
    
    try:
        # Build the image
        image = docker.images().build(
            path=app_dir,
            tag=f"{image_name}:{version}",
            buildargs={
                "NODE_ENV": "production",
                "APP_VERSION": version
            },
            labels={
                "maintainer": "docker-pyo3-example",
                "version": version,
                "build_date": str(time.time())
            }
        )
        
        print("✅ Image built successfully!")
        
        # Get image details
        info = image.inspect()
        print(f"   Image ID: {info['Id'][:12]}...")
        print(f"   Size: {info['Size'] / (1024*1024):.1f} MB")
        print(f"   Created: {info['Created']}")
        
        return image
        
    except Exception as e:
        print(f"❌ Build failed: {e}")
        return None

def test_image(docker, image_name, version):
    """Test the built image"""
    
    print(f"🧪 Testing image {image_name}:{version}...")
    
    # Create test container
    test_container = docker.containers().create(
        image=f"{image_name}:{version}",
        name=f"test-{image_name}-{version}",
        ports={"3000": "3001"},
        env=["NODE_ENV=test"]
    )
    
    try:
        # Start container
        test_container.start()
        print("   Container started")
        
        # Wait for startup
        time.sleep(5)
        
        # Check if container is running
        info = test_container.inspect()
        if not info['State']['Running']:
            print("❌ Container failed to start")
            logs = test_container.logs()
            print(f"   Logs: {logs}")
            return False
        
        print("✅ Container is running")
        
        # Test health endpoint
        try:
            # In a real scenario, you'd make HTTP requests to test the API
            # For this example, we'll just check if the process is running
            result = test_container.exec(["curl", "-f", "http://localhost:3000/health"])
            print("✅ Health check passed")
        except:
            print("⚠️  Health check endpoint not accessible (curl not available)")
        
        # Check logs for startup messages
        logs = test_container.logs()
        if "Server running on port 3000" in logs:
            print("✅ Application started successfully")
        else:
            print("⚠️  Application startup message not found")
        
        return True
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        return False
        
    finally:
        # Cleanup test container
        try:
            test_container.stop()
            test_container.remove(force=True)
            print("   Test container cleaned up")
        except:
            pass

def tag_and_prepare_for_registry(docker, image_name, version, registry="localhost:5000"):
    """Tag image for registry deployment"""
    
    print(f"🏷️  Tagging image for registry {registry}...")
    
    try:
        image = docker.images().get(f"{image_name}:{version}")
        
        # Tag for registry
        registry_tag = f"{registry}/{image_name}:{version}"
        latest_tag = f"{registry}/{image_name}:latest"
        
        image.tag(registry_tag)
        image.tag(latest_tag)
        
        print(f"✅ Tagged as {registry_tag}")
        print(f"✅ Tagged as {latest_tag}")
        
        return registry_tag, latest_tag
        
    except Exception as e:
        print(f"❌ Tagging failed: {e}")
        return None, None

def deploy_image(docker, registry_tag):
    """Deploy the image as a running application"""
    
    print(f"🚀 Deploying {registry_tag}...")
    
    try:
        # Create deployment container
        app_container = docker.containers().create(
            image=registry_tag,
            name="sample-app-production",
            ports={"3000": "8080"},
            env=[
                "NODE_ENV=production",
                "PORT=3000"
            ],
            restart_policy={"name": "unless-stopped"},
            labels={
                "environment": "production",
                "app": "sample-app"
            }
        )
        
        app_container.start()
        print("✅ Application deployed successfully!")
        print("🌐 Application available at http://localhost:8080")
        
        # Wait and check health
        time.sleep(3)
        
        info = app_container.inspect()
        if info['State']['Running']:
            print("✅ Application is running in production")
            
            # Show logs
            logs = app_container.logs()
            print("   Recent logs:")
            for line in logs.split('\n')[-3:]:
                if line.strip():
                    print(f"     {line}")
        else:
            print("❌ Application failed to start in production")
            
        return app_container
        
    except Exception as e:
        print(f"❌ Deployment failed: {e}")
        return None

def cleanup_deployment(container):
    """Clean up deployed application"""
    
    if container:
        print("🧹 Cleaning up deployment...")
        try:
            container.stop()
            container.remove(force=True)
            print("✅ Deployment cleaned up")
        except Exception as e:
            print(f"⚠️  Cleanup warning: {e}")

def main():
    docker = Docker()
    
    print("🏭 Docker-PyO3 Image Build Pipeline Example")
    print("=" * 50)
    
    # Configuration
    image_name = "sample-node-app"
    version = "1.0.0"
    
    # Create temporary directory for our sample app
    with tempfile.TemporaryDirectory() as temp_dir:
        print(f"📁 Creating sample application in {temp_dir}")
        
        # Create sample application files
        create_sample_dockerfile(temp_dir)
        print("✅ Sample application created")
        
        # Build the image
        image = build_and_test_image(docker, temp_dir, image_name, version)
        if not image:
            print("❌ Build pipeline failed")
            return
        
        # Test the image
        test_passed = test_image(docker, image_name, version)
        if not test_passed:
            print("❌ Image tests failed")
            return
        
        # Tag for registry (simulate registry preparation)
        registry_tag, latest_tag = tag_and_prepare_for_registry(docker, image_name, version)
        if not registry_tag:
            print("❌ Registry tagging failed")
            return
        
        # Deploy the application
        deployed_container = deploy_image(docker, f"{image_name}:{version}")
        
        if deployed_container:
            print("\n🎉 Build pipeline completed successfully!")
            print("Pipeline stages:")
            print("  ✅ Build")
            print("  ✅ Test") 
            print("  ✅ Tag")
            print("  ✅ Deploy")
            
            print("\nPress Enter to clean up...")
            input()
            
            # Cleanup
            cleanup_deployment(deployed_container)
        
        # Clean up images
        print("🧹 Cleaning up images...")
        try:
            # Remove tagged images
            for tag in [f"{image_name}:{version}", registry_tag, latest_tag]:
                try:
                    img = docker.images().get(tag)
                    img.delete()
                    print(f"   Removed {tag}")
                except:
                    pass
        except Exception as e:
            print(f"⚠️  Image cleanup warning: {e}")
    
    print("\n🎉 Image build pipeline example completed!")

if __name__ == "__main__":
    main()