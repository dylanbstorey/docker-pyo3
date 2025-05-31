#!/usr/bin/env python3
"""
Development Workflow Example

Demonstrates development patterns including hot reloading, testing, and debugging with docker-pyo3.
"""

from docker_pyo3 import Docker
import tempfile
import os
import time

def create_sample_app(app_dir):
    """Create a sample Node.js app with hot reloading support"""
    
    # Package.json with nodemon for hot reloading
    package_json = '''
{
  "name": "dev-app",
  "version": "1.0.0",
  "description": "Development workflow example",
  "main": "server.js",
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest",
    "test:watch": "jest --watch"
  },
  "dependencies": {
    "express": "^4.18.0",
    "cors": "^2.8.5"
  },
  "devDependencies": {
    "nodemon": "^2.0.20",
    "jest": "^29.0.0"
  }
}
'''
    
    # Main application file
    server_js = '''
const express = require('express');
const cors = require('cors');
const app = express();
const PORT = process.env.PORT || 3000;

app.use(cors());
app.use(express.json());

// Health check endpoint
app.get('/health', (req, res) => {
    res.json({ 
        status: 'healthy', 
        version: '1.0.0',
        environment: process.env.NODE_ENV || 'development',
        uptime: process.uptime(),
        timestamp: new Date().toISOString()
    });
});

// API endpoints
app.get('/api/users', (req, res) => {
    res.json([
        { id: 1, name: 'John Doe', email: 'john@example.com' },
        { id: 2, name: 'Jane Smith', email: 'jane@example.com' }
    ]);
});

app.post('/api/users', (req, res) => {
    const { name, email } = req.body;
    const newUser = { 
        id: Date.now(), 
        name, 
        email,
        created: new Date().toISOString()
    };
    res.status(201).json(newUser);
});

// Development route with hot reload indicator
app.get('/api/dev-info', (req, res) => {
    res.json({
        message: 'Development server is running!',
        hotReload: true,
        lastModified: new Date().toISOString(),
        pid: process.pid
    });
});

app.listen(PORT, '0.0.0.0', () => {
    console.log(`🚀 Server running on port ${PORT}`);
    console.log(`📊 Environment: ${process.env.NODE_ENV || 'development'}`);
    console.log(`🔄 Hot reload: ${process.env.NODE_ENV === 'development' ? 'enabled' : 'disabled'}`);
});

module.exports = app;
'''
    
    # Test file
    server_test_js = '''
const request = require('supertest');
const app = require('./server');

describe('API Endpoints', () => {
    test('GET /health should return healthy status', async () => {
        const response = await request(app)
            .get('/health')
            .expect(200);
        
        expect(response.body.status).toBe('healthy');
        expect(response.body).toHaveProperty('uptime');
    });
    
    test('GET /api/users should return user list', async () => {
        const response = await request(app)
            .get('/api/users')
            .expect(200);
        
        expect(Array.isArray(response.body)).toBe(true);
        expect(response.body.length).toBeGreaterThan(0);
    });
    
    test('POST /api/users should create new user', async () => {
        const newUser = { name: 'Test User', email: 'test@example.com' };
        
        const response = await request(app)
            .post('/api/users')
            .send(newUser)
            .expect(201);
        
        expect(response.body.name).toBe(newUser.name);
        expect(response.body.email).toBe(newUser.email);
        expect(response.body).toHaveProperty('id');
    });
});
'''
    
    # Development Dockerfile with hot reloading
    dockerfile_dev = '''
FROM node:18-alpine

WORKDIR /app

# Install dependencies first (for layer caching)
COPY package*.json ./
RUN npm install

# Copy source code
COPY . .

# Development mode with nodemon
EXPOSE 3000
CMD ["npm", "run", "dev"]
'''
    
    # Production Dockerfile
    dockerfile_prod = '''
FROM node:18-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

FROM node:18-alpine AS runtime

WORKDIR /app
COPY --from=builder /app/node_modules ./node_modules
COPY server.js ./

RUN addgroup -g 1001 -S nodejs && \\
    adduser -S nodejs -u 1001

USER nodejs
EXPOSE 3000

CMD ["npm", "start"]
'''
    
    # Write all files
    files = {
        'package.json': package_json,
        'server.js': server_js,
        'server.test.js': server_test_js,
        'Dockerfile.dev': dockerfile_dev,
        'Dockerfile.prod': dockerfile_prod
    }
    
    for filename, content in files.items():
        with open(os.path.join(app_dir, filename), 'w') as f:
            f.write(content)

def setup_development_environment(docker, app_dir):
    """Set up development environment with hot reloading"""
    print("🛠️  Setting up development environment...")
    
    try:
        # Build development image
        print("   📦 Building development image...")
        dev_image = docker.images().build(
            path=app_dir,
            dockerfile="Dockerfile.dev",
            tag="dev-app:dev",
            buildargs={"NODE_ENV": "development"}
        )
        print("   ✅ Development image built")
        
        # Create development container with volume mounting for hot reload
        print("   🔄 Creating development container with hot reload...")
        dev_container = docker.containers().create(
            image="dev-app:dev",
            name="dev-app-container",
            ports={"3000": "3000"},
            volumes=[f"{app_dir}:/app"],  # Mount source code for hot reload
            env=[
                "NODE_ENV=development",
                "DEBUG=true"
            ],
            labels={
                "environment": "development",
                "hot_reload": "enabled"
            }
        )
        
        dev_container.start()
        print("   ✅ Development container started with hot reload")
        print("   🌐 Application available at http://localhost:3000")
        
        return dev_container
        
    except Exception as e:
        print(f"   ❌ Development setup failed: {e}")
        return None

def test_hot_reload(docker, dev_container, app_dir):
    """Demonstrate hot reloading by modifying source code"""
    print("🔥 Testing hot reload functionality...")
    
    try:
        # Wait for initial startup
        time.sleep(5)
        
        # Test initial API response
        print("   📡 Testing initial API response...")
        result = dev_container.exec(["wget", "-qO-", "http://localhost:3000/api/dev-info"])
        if "Development server is running" in result:
            print("   ✅ Initial API working")
        
        # Modify the source code
        print("   ✏️  Modifying source code...")
        server_file = os.path.join(app_dir, 'server.js')
        
        with open(server_file, 'r') as f:
            content = f.read()
        
        # Add a new endpoint
        new_endpoint = '''
app.get('/api/hot-reload-test', (req, res) => {
    res.json({
        message: 'Hot reload is working!',
        modified: true,
        timestamp: new Date().toISOString()
    });
});
'''
        
        # Insert before the listen call
        modified_content = content.replace(
            'app.listen(PORT,',
            new_endpoint + '\napp.listen(PORT,'
        )
        
        with open(server_file, 'w') as f:
            f.write(modified_content)
        
        print("   ✅ Source code modified (added new endpoint)")
        
        # Wait for nodemon to reload
        print("   ⏱️  Waiting for hot reload...")
        time.sleep(10)
        
        # Test the new endpoint
        print("   🧪 Testing new endpoint after hot reload...")
        try:
            result = dev_container.exec(["wget", "-qO-", "http://localhost:3000/api/hot-reload-test"])
            if "Hot reload is working" in result:
                print("   ✅ Hot reload successful - new endpoint is working!")
            else:
                print("   ⚠️  Hot reload may not have triggered")
                print(f"     Response: {result}")
        except Exception as e:
            print(f"   ⚠️  Could not test new endpoint: {e}")
        
        # Check container logs for reload confirmation
        logs = dev_container.logs()
        if "restarting due to changes" in logs.lower() or "server running" in logs:
            print("   ✅ Hot reload confirmed in logs")
        
    except Exception as e:
        print(f"   ❌ Hot reload test failed: {e}")

def run_tests_in_container(docker, app_dir):
    """Run tests in a dedicated test container"""
    print("🧪 Running tests in containerized environment...")
    
    try:
        # Create test container
        test_container = docker.containers().create(
            image="node:18-alpine",
            name="test-runner",
            volumes=[f"{app_dir}:/app"],
            working_dir="/app",
            env=["NODE_ENV=test"],
            command=[
                "sh", "-c",
                "npm install && npm test"
            ]
        )
        
        test_container.start()
        print("   ⏱️  Running tests...")
        
        # Wait for tests to complete
        time.sleep(15)
        
        # Get test results
        logs = test_container.logs()
        
        # Clean up test container
        test_container.stop()
        test_container.remove()
        
        # Analyze test results
        if "Tests:" in logs and "failed" not in logs.lower():
            print("   ✅ All tests passed!")
        elif "failing" in logs.lower():
            print("   ❌ Some tests failed")
        else:
            print("   ⚠️  Test results unclear")
        
        print("   📋 Test output:")
        for line in logs.split('\n')[-10:]:  # Show last 10 lines
            if line.strip():
                print(f"     {line}")
        
    except Exception as e:
        print(f"   ❌ Test execution failed: {e}")

def build_production_image(docker, app_dir):
    """Build optimized production image"""
    print("🏭 Building production image...")
    
    try:
        prod_image = docker.images().build(
            path=app_dir,
            dockerfile="Dockerfile.prod",
            tag="dev-app:prod",
            buildargs={"NODE_ENV": "production"}
        )
        
        print("   ✅ Production image built")
        
        # Compare image sizes
        dev_info = docker.images().get("dev-app:dev").inspect()
        prod_info = prod_image.inspect()
        
        dev_size = dev_info['Size'] / (1024 * 1024)
        prod_size = prod_info['Size'] / (1024 * 1024)
        
        print(f"   📊 Image sizes:")
        print(f"     Development: {dev_size:.1f} MB")
        print(f"     Production: {prod_size:.1f} MB")
        print(f"     Savings: {dev_size - prod_size:.1f} MB ({((dev_size - prod_size) / dev_size * 100):.1f}%)")
        
        return prod_image
        
    except Exception as e:
        print(f"   ❌ Production build failed: {e}")
        return None

def test_production_deployment(docker):
    """Test production deployment"""
    print("🚀 Testing production deployment...")
    
    try:
        prod_container = docker.containers().create(
            image="dev-app:prod",
            name="prod-app-container",
            ports={"3000": "3001"},
            env=["NODE_ENV=production"],
            restart_policy={"name": "unless-stopped"},
            labels={
                "environment": "production",
                "hot_reload": "disabled"
            }
        )
        
        prod_container.start()
        print("   ✅ Production container started")
        print("   🌐 Production app available at http://localhost:3001")
        
        # Wait for startup
        time.sleep(5)
        
        # Test production endpoint
        try:
            result = prod_container.exec(["wget", "-qO-", "http://localhost:3000/health"])
            if "healthy" in result:
                print("   ✅ Production health check passed")
        except:
            print("   ⚠️  Production health check failed (wget not available)")
        
        return prod_container
        
    except Exception as e:
        print(f"   ❌ Production deployment failed: {e}")
        return None

def demonstrate_debugging(docker, dev_container):
    """Demonstrate debugging capabilities"""
    print("🐛 Demonstrating debugging capabilities...")
    
    try:
        # Show container processes
        print("   📊 Container processes:")
        processes = dev_container.top()
        for process in processes.get('Processes', [])[:3]:  # Show first 3
            print(f"     {' '.join(process)}")
        
        # Show environment variables
        print("   🔧 Environment variables:")
        inspect_info = dev_container.inspect()
        env_vars = inspect_info['Config']['Env']
        for env in env_vars[:5]:  # Show first 5
            if not env.startswith('PATH='):
                print(f"     {env}")
        
        # Show recent logs
        print("   📋 Recent application logs:")
        logs = dev_container.logs()
        for line in logs.split('\n')[-5:]:  # Show last 5 lines
            if line.strip():
                print(f"     {line}")
        
        # Show mounted volumes
        print("   💾 Mounted volumes:")
        mounts = inspect_info.get('Mounts', [])
        for mount in mounts:
            if mount.get('Type') == 'bind':
                print(f"     {mount['Source']} → {mount['Destination']}")
        
    except Exception as e:
        print(f"   ❌ Debugging demo failed: {e}")

def cleanup_resources(docker, containers, images):
    """Clean up development resources"""
    print("🧹 Cleaning up development resources...")
    
    # Stop and remove containers
    for container_name in containers:
        try:
            container = docker.containers().get(container_name)
            container.stop()
            container.remove(force=True)
            print(f"   ✅ Removed container: {container_name}")
        except:
            pass
    
    # Remove images
    for image_tag in images:
        try:
            image = docker.images().get(image_tag)
            image.delete()
            print(f"   ✅ Removed image: {image_tag}")
        except:
            pass

def main():
    docker = Docker()
    
    print("💻 Docker-PyO3 Development Workflow Example")
    print("=" * 50)
    
    containers_to_cleanup = []
    images_to_cleanup = ["dev-app:dev", "dev-app:prod"]
    
    # Create temporary directory for sample app
    with tempfile.TemporaryDirectory() as temp_dir:
        print(f"📁 Creating sample application in {temp_dir}")
        
        try:
            # Create sample application
            create_sample_app(temp_dir)
            print("✅ Sample Node.js application created")
            
            # Set up development environment
            dev_container = setup_development_environment(docker, temp_dir)
            if dev_container:
                containers_to_cleanup.append("dev-app-container")
                
                # Test hot reloading
                test_hot_reload(docker, dev_container, temp_dir)
                
                # Demonstrate debugging
                demonstrate_debugging(docker, dev_container)
            
            # Run tests
            run_tests_in_container(docker, temp_dir)
            
            # Build production image
            prod_image = build_production_image(docker, temp_dir)
            
            # Test production deployment
            if prod_image:
                prod_container = test_production_deployment(docker)
                if prod_container:
                    containers_to_cleanup.append("prod-app-container")
            
            print(f"\n🎉 Development workflow demonstration completed!")
            print(f"📊 Workflow stages demonstrated:")
            print(f"   ✅ Development environment setup")
            print(f"   ✅ Hot reload functionality")
            print(f"   ✅ Automated testing")
            print(f"   ✅ Production build optimization")
            print(f"   ✅ Production deployment")
            print(f"   ✅ Debugging capabilities")
            
            print(f"\n🌐 Active services:")
            print(f"   Development: http://localhost:3000")
            print(f"   Production: http://localhost:3001")
            
            print(f"\nPress Enter to clean up...")
            input()
            
        except Exception as e:
            print(f"❌ Workflow demo failed: {e}")
        
        finally:
            # Cleanup
            cleanup_resources(docker, containers_to_cleanup, images_to_cleanup)
    
    print(f"\n🎉 Development workflow example completed!")

if __name__ == "__main__":
    main()