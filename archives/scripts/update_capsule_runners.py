#!/usr/bin/env python3
import os
import re

# Template for the new methods to add
register_method = '''
    async def register_with_api_gateway(self, pid):
        """Register the service with the API Gateway"""
        try:
            import httpx
            
            # Determine service port based on service name
            port = self.get_service_port()
            if not port:
                print(f"⚠️ Could not determine port for service {self.service_name}")
                return
            
            # Service registration payload
            registration = {
                "name": self.service_name,
                "url": f"http://localhost:{port}",
                "weight": 100,
                "priority": 1,
                "health_check": {
                    "path": "/health",
                    "interval_seconds": 30,
                    "timeout_seconds": 5,
                    "retries": 3
                },
                "metadata": {
                    "pid": pid,
                    "capsule_layer": True,
                    "launched_by": "NOA CEO"
                }
            }
            
            # Register with API Gateway
            async with httpx.AsyncClient() as client:
                response = await client.post(
                    "http://localhost:8084/services/register",
                    json=registration,
                    timeout=10.0
                )
                
                if response.status_code == 200:
                    print(f"📝 Service {self.service_name} registered with API Gateway")
                else:
                    print(f"⚠️ Failed to register {self.service_name} with API Gateway: {response.status_code}")
                    
        except ImportError:
            print(f"⚠️ httpx not available for {self.service_name} registration")
        except Exception as e:
            print(f"⚠️ Error registering {self.service_name} with API Gateway: {e}")
    
    def get_service_port(self):
        """Get the port for this service"""
        # Port mapping based on service name
        port_map = {
            "noa-core": 8001,
            "digest-agent": 8002,
            "board-agents": 8003,
            "model-selector": 8004,
            "agent-registry": 8005,
            "security-scanner": 8007,
            "microagent-stacks": 8008,
            "capsule-orchestrator": 8009,
            "trifecta-court": 8000,
            "agent-registry-enhanced": 8009,
            "model-selector-enhanced": 8010,
            "canary-testing": 8011,
            "unity-harmonica": 8012,
            "growth-partnerships-agent": 8013,
            "coo-agent": 8014,
            "cfo-finops-agent": 8015,
            "legal-compliance-agent": 8016,
            "digest-rnd-agent": 8017,
            "security-agent": 8018,
            "strategy-cto-agent": 8019,
            "noa-commander": 8020,
            "noa-executive": 8021,
            "knowledge-graph": 8022,
            "noa-autonomous-optimizer": 8023,
            "quality-pal": 8024,
            "learn-pal": 8025,
            "env-friend": 8026,
            "host-profiler": 8027,
            "model-serving": 8028,
            "model-registry": 8029,
            "api-gateway-complete": 8030,
            "noa-core-complete": 8031,
            "capsule-orchestrator-complete": 8032,
            "board-agents-complete": 8033,
            "model-selector-complete": 8034,
            "microagent-stacks-complete": 8035,
            "security-scanner-complete": 8036,
            "agent-registry-complete": 8037,
            "digest-agent-complete": 8038,
            "board-agents-deploy": 8039,
            "noa-ultimate-autonomous-system": 8040,
            "coordinator-cluster": 8041,
            "sandbox-cluster": 8042,
            "deployed-app-cluster": 8043,
            "noa-core-backup": 8044,
            "unified-api-gateway": 8045
        }
        
        return port_map.get(self.service_name)
'''

# Find all capsule runners
capsule_runners = []
for root, dirs, files in os.walk('capsules/services'):
    for file in files:
        if file == 'capsule_runner.py':
            capsule_runners.append(os.path.join(root, file))

print(f'Found {len(capsule_runners)} capsule runners to update')

for runner_path in capsule_runners:
    with open(runner_path, 'r') as f:
        content = f.read()
    
    # Check if methods already exist
    if 'register_with_api_gateway' in content:
        print(f'Skipping {runner_path} - already has registration methods')
        continue
    
    # Add the registration methods before execute_service
    if 'async def execute_service(self, entry_point):' in content:
        # Replace the execute_service method call
        content = re.sub(
            r'            # For long-running services, run in background\n            print\(f"✅ Service \{self\.service_name\} started as background process \(PID: \{process\.pid\}\)"\)\n            \n            # For long-running services, don\'t wait for completion\n            # Just return success immediately\n            return True',
            r'            # For long-running services, run in background\n            print(f"✅ Service {self.service_name} started as background process (PID: {process.pid})")\n            \n            # Register service with API Gateway\n            await self.register_with_api_gateway(process.pid)\n            \n            # For long-running services, don\'t wait for completion\n            # Just return success immediately\n            return True',
            content
        )
        
        # Add the new methods before execute_service
        content = content.replace(
            '    async def execute_service(self, entry_point):',
            register_method + '\n    async def execute_service(self, entry_point):'
        )
        
        with open(runner_path, 'w') as f:
            f.write(content)
        
        print(f'Updated: {runner_path}')
    else:
        print(f'No execute_service method found in {runner_path}')

print('All capsule runners updated')
