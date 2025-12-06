#!/usr/bin/env python3
"""
322k Deployment Validation Tool
Validates deployment success against Truth Gate requirements
"""

import json
import sys
import hashlib
from datetime import datetime

def validate_deployment(manifest_path, deployment_report_path):
    """Validate deployment against success criteria"""
    print("🔍 Validating 322k deployment...")
    
    # Load manifest and report
    with open(manifest_path, 'r') as f:
        manifest = json.load(f)
    
    with open(deployment_report_path, 'r') as f:
        report = json.load(f)
    
    # Validate success criteria
    success_criteria = manifest['success_criteria']
    deployment_summary = report['deployment_summary']
    
    validations = []
    
    # Check agent deployment
    total_agents = int(deployment_summary['total_agents_deployed'])
    target_agents = success_criteria['deployment_metrics']['total_agents_deployed']
    validations.append({
        'check': 'Total Agents Deployed',
        'actual': total_agents,
        'target': target_agents,
        'passed': total_agents >= target_agents
    })
    
    # Check success rate
    success_rate = float(deployment_summary['success_rate'].replace('%', ''))
    target_rate = float(success_criteria['deployment_metrics']['success_rate'].replace('>', '').replace('%', ''))
    validations.append({
        'check': 'Success Rate',
        'actual': f"{success_rate}%",
        'target': f">{target_rate}%",
        'passed': success_rate >= target_rate
    })
    
    # Truth Gate validation
    truth_gate = report.get('truth_gate_compliance', {})
    validations.append({
        'check': 'Truth Gate Compliance',
        'actual': 'PASSED' if all(truth_gate.values()) else 'FAILED',
        'target': 'PASSED',
        'passed': all(truth_gate.values())
    })
    
    # Print results
    print("\n📊 Validation Results:")
    print("-" * 60)
    for validation in validations:
        status = "✅ PASS" if validation['passed'] else "❌ FAIL"
        print(f"{validation['check']:<25} {validation['actual']:<15} {status}")
    
    overall_success = all(v['passed'] for v in validations)
    print("-" * 60)
    print(f"Overall Result: {'✅ DEPLOYMENT SUCCESSFUL' if overall_success else '❌ DEPLOYMENT FAILED'}")
    
    return overall_success

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python3 validate_deployment.py <manifest_path> <report_path>")
        sys.exit(1)
    
    success = validate_deployment(sys.argv[1], sys.argv[2])
    sys.exit(0 if success else 1)
