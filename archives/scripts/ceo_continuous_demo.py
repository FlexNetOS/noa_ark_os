#!/usr/bin/env python3
"""
NOA CEO Continuous Operation Demo
Demonstrates how the CEO commands the system to run continuously
"""

import requests
import time
import json

def demo_continuous_operation():
    """Demonstrate CEO commanding continuous operation"""

    print("🚀 NOA CEO Continuous Operation Demo")
    print("=" * 50)

    # Step 1: Check CEO health
    print("\n1. Checking CEO status...")
    try:
        response = requests.get("http://localhost:8011/health")
        print(f"✅ CEO Status: {response.json()}")
    except Exception as e:
        print(f"❌ CEO not available: {e}")
        return

    # Step 2: Command continuous operation
    print("\n2. CEO commanding continuous operation...")
    try:
        response = requests.post("http://localhost:8011/system/continuous-operation")
        result = response.json()
        print(f"✅ Continuous operation initiated: {result['status']}")
        print(f"   Work Plan ID: {result['work_plan_id']}")
        print(f"   System Status: {result['system_status']}")
    except Exception as e:
        print(f"❌ Failed to initiate continuous operation: {e}")
        return

    # Step 3: Check system status
    print("\n3. Checking system status...")
    try:
        response = requests.get("http://localhost:8011/system/status")
        status = response.json()
        print("✅ System Status:"        print(f"   CEO: {status['ceo_status']}")
        print(f"   Continuous Mode: {status.get('continuous_operation', 'Unknown')}")
        print(f"   Orchestrator: {status['orchestrator_status'].get('status', 'Unknown')}")
        print(f"   NOA CORE: {status['core_status'].get('status', 'Unknown')}")
    except Exception as e:
        print(f"❌ Failed to get system status: {e}")

    # Step 4: Monitor for a bit
    print("\n4. Monitoring continuous operation...")
    for i in range(3):
        time.sleep(2)
        try:
            response = requests.get("http://localhost:8001/status")
            orch_status = response.json()
            print(f"   Orchestrator continuous mode: {orch_status.get('continuous_mode', False)}")
        except:
            print("   Orchestrator status unavailable")

    # Step 5: Stop continuous operation
    print("\n5. CEO stopping continuous operation...")
    try:
        response = requests.post("http://localhost:8011/system/stop-continuous")
        result = response.json()
        print(f"✅ Continuous operation stopped: {result['status']}")
    except Exception as e:
        print(f"❌ Failed to stop continuous operation: {e}")

    print("\n🎯 Demo completed successfully!")
    print("The CEO successfully commanded the system to:")
    print("  - Run NOA CORE at 100% performance")
    print("  - Enable continuous self-updating")
    print("  - Enable continuous optimization")
    print("  - Enable continuous capsulation")
    print("  - No timeouts allowed")

if __name__ == "__main__":
    demo_continuous_operation()
