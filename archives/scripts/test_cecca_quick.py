#!/usr/bin/env python3
"""
Quick smoke test for CECCA capsule - tests finite execution
"""
import sys
import os
import subprocess
import signal
import time

def test_cecca_finite_execution():
    """Test that CECCA runs and exits within reasonable time"""
    print("🧪 Testing CECCA finite execution...")
    
    # Change to CECCA directory
    cecca_dir = "/home/deflex/ark-ai-os-workspace/capsules/core/noa-orchestrator"
    os.chdir(cecca_dir)
    
    # Start CECCA in background with timeout
    start_time = time.time()
    try:
        # Use timeout to prevent infinite execution
        result = subprocess.run([
            "timeout", "30s", 
            "python3", "src/capsule.py"
        ], capture_output=True, text=True)
        
        end_time = time.time()
        execution_time = end_time - start_time
        
        print(f"⏱️ Execution time: {execution_time:.2f} seconds")
        print(f"🔄 Return code: {result.returncode}")
        
        # Check if it finished within reasonable time (not infinite)
        if execution_time < 30:
            print("✅ CECCA executed finitely (no infinite loops)")
            return True
        else:
            print("❌ CECCA may have infinite loops (timeout reached)")
            return False
            
    except Exception as e:
        print(f"❌ Error testing CECCA: {e}")
        return False

if __name__ == "__main__":
    success = test_cecca_finite_execution()
    sys.exit(0 if success else 1)
