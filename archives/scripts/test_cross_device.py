#!/usr/bin/env python3
"""
Cross-Device Experience Framework Test Suite
Tests all 16 cross-device components for functionality
"""

import asyncio
import sys
import os
from pathlib import Path

# Add cross-device-experience to path
sys.path.append(str(Path(__file__).parent / "cross-device-experience"))

async def test_component(component_class, component_name: str):
    """Test a single component"""
    print(f"\n🧪 Testing {component_name}...")

    try:
        # Initialize component
        component = component_class()

        # Test basic operation
        test_operation = {
            "action": "test_operation",
            "devices": [
                {
                    "device_id": "test-device-1",
                    "device_type": "test_device",
                    "capabilities": {"test": True}
                }
            ]
        }

        result = await component.manage_cross_device_operation(test_operation)

        if result.get("status") == "success":
            print(f"✅ {component_name}: PASSED")
            return True
        else:
            print(f"❌ {component_name}: FAILED - {result.get('reason', 'Unknown error')}")
            return False

    except Exception as e:
        print(f"❌ {component_name}: ERROR - {e}")
        return False

async def run_full_test_suite():
    """Run tests on all cross-device components"""
    print("🚀 Starting Cross-Device Experience Framework Test Suite")
    print("=" * 60)

    # Import all components
    components_to_test = [
        ("coherent_experience_manager", "Coherentexperiencemanager"),
        ("unified_ux_system", "Unifieduxsystem"),
        ("cross_device_auth_system", "Crossdeviceauthsystem"),
        ("cross_device_backup_system", "Crossdevicebackupsystem"),
        ("cross_device_performance_optimizer", "Crossdeviceperformanceoptimizer"),
        ("cross_device_security_system", "Crossdevicesecuritysystem"),
        ("cross_device_sync_system", "Crossdevicesyncsystem"),
        ("cross_device_task_continuity", "Crossdevicetaskcontinuity"),
        ("device_capability_detector", "Devicecapabilitydetector"),
        ("device_resource_sharing_system", "Deviceresourcesharingsystem"),
        ("device_ui_adapter", "Deviceuiadapter"),
        ("seamless_handoff_system", "Seamlesshandoffsystem"),
        ("unified_data_sync", "Unifieddatasync"),
        ("unified_notification_system", "Unifiednotificationsystem"),
        ("unified_settings_system", "Unifiedsettingssystem"),
        ("constitutional_cross_device_validator", "Constitutionalcrossdevicevalidator")
    ]

    passed = 0
    total = len(components_to_test)

    for module_name, class_name in components_to_test:
        try:
            # Dynamic import
            module = __import__(module_name, fromlist=[class_name])
            component_class = getattr(module, class_name)

            # Test the component
            if await test_component(component_class, module_name):
                passed += 1

        except ImportError as e:
            print(f"❌ {module_name}: IMPORT FAILED - {e}")
        except Exception as e:
            print(f"❌ {module_name}: UNEXPECTED ERROR - {e}")

    print("\n" + "=" * 60)
    print(f"📊 Test Results: {passed}/{total} components passed")

    if passed == total:
        print("🎉 All cross-device components are working correctly!")
        return True
    else:
        print(f"⚠️  {total - passed} components need attention")
        return False

async def test_launcher_integration():
    """Test the launcher integration"""
    print("\n🔗 Testing Launcher Integration...")

    try:
        from cross_device_launcher import CrossDeviceLauncher

        launcher = CrossDeviceLauncher()
        await launcher.initialize_components()

        print("✅ Launcher initialization: PASSED")

        # Test health checks
        await launcher.run_health_checks()

        print("✅ Health checks: PASSED")
        return True

    except Exception as e:
        print(f"❌ Launcher integration: FAILED - {e}")
        return False

if __name__ == "__main__":
    async def main():
        # Run component tests
        components_ok = await run_full_test_suite()

        # Run launcher integration test
        launcher_ok = await test_launcher_integration()

        # Final result
        if components_ok and launcher_ok:
            print("\n🎉 Cross-Device Experience Framework: FULLY OPERATIONAL")
            sys.exit(0)
        else:
            print("\n❌ Cross-Device Experience Framework: ISSUES DETECTED")
            sys.exit(1)

    asyncio.run(main())
