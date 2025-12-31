#!/usr/bin/env python3
"""
Windows C: Drive Corruption and Redundancy Analyzer
Analyzes Windows filesystem for suspicious directory patterns and cleanup opportunities.
"""

import os
import subprocess
import json
from pathlib import Path
from collections import defaultdict, Counter

def run_command(cmd):
    """Run shell command and return output."""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        return result.stdout.strip().split('\n') if result.stdout.strip() else []
    except Exception as e:
        return []

def analyze_windows_space_usage():
    """Analyze major space consumers on Windows C: drive."""
    print("=== WINDOWS C: DRIVE SPACE ANALYSIS ===")
    
    # Get total size of C: drive
    cmd = "du -sh /mnt/c 2>/dev/null"
    total_size = run_command(cmd)
    if total_size:
        print(f"Total C: drive usage: {total_size[0].split()[0]}")
    
    # Check major directories
    major_dirs = [
        '/mnt/c/Users',
        '/mnt/c/Program Files',
        '/mnt/c/Program Files (x86)',
        '/mnt/c/ProgramData',
        '/mnt/c/Windows',
        '/mnt/c/temp',
        '/mnt/c/AI-Dev',
        '/mnt/c/DeFlex-AI-OS',
        '/mnt/c/ArkOS-Test'
    ]
    
    print("\nMajor directory sizes:")
    for dir_path in major_dirs:
        if os.path.exists(dir_path):
            cmd = f"du -sh '{dir_path}' 2>/dev/null"
            result = run_command(cmd)
            if result and result[0]:
                size = result[0].split()[0]
                print(f"  {dir_path}: {size}")
    
    return total_size

def analyze_windows_repeated_patterns():
    """Find repeated directory patterns on Windows."""
    print("\n=== WINDOWS REPEATED DIRECTORY PATTERNS ===")
    
    # Look for consecutive repeated directory names
    cmd = """find /mnt/c -type d 2>/dev/null | head -10000 | awk -F/ '{
        for(i=1; i<=NF; i++) {
            if($i==$(i+1) && $i!="") {
                print $0 " -> REPEATED: " $i
            }
        }
    }'"""
    
    repeated_paths = run_command(cmd)
    
    print(f"Found {len(repeated_paths)} paths with consecutive repeated directory names:")
    for path in repeated_paths[:20]:  # Show first 20
        print(f"  {path}")
    
    return repeated_paths

def analyze_windows_build_artifacts():
    """Analyze Windows build artifacts and cache directories."""
    print("\n=== WINDOWS BUILD ARTIFACTS AND CACHE ===")
    
    artifacts = {}
    
    # Common Windows build/cache patterns
    patterns = {
        'node_modules': "find /mnt/c -name 'node_modules' -type d 2>/dev/null | head -100",
        'target': "find /mnt/c -name 'target' -type d 2>/dev/null | head -50",
        '__pycache__': "find /mnt/c -name '__pycache__' -type d 2>/dev/null | head -100",
        '.git': "find /mnt/c -name '.git' -type d 2>/dev/null | head -50",
        'bin': "find /mnt/c -name 'bin' -type d 2>/dev/null | head -100",
        'obj': "find /mnt/c -name 'obj' -type d 2>/dev/null | head -100",
        'Debug': "find /mnt/c -name 'Debug' -type d 2>/dev/null | head -50",
        'Release': "find /mnt/c -name 'Release' -type d 2>/dev/null | head -50",
        'packages': "find /mnt/c -name 'packages' -type d 2>/dev/null | head -50"
    }
    
    for pattern_name, cmd in patterns.items():
        result = run_command(cmd)
        artifacts[pattern_name] = [r for r in result if r.strip()]
        count = len(artifacts[pattern_name])
        print(f"\n{pattern_name.upper()}:")
        print(f"  Found: {count} directories")
        
        if count > 0:
            print(f"  Sample locations:")
            for item in artifacts[pattern_name][:5]:
                print(f"    {item}")
            
            # Get size estimate for some patterns
            if pattern_name in ['node_modules', 'target', '__pycache__'] and count > 0:
                size_cmd = f"du -sh '{artifacts[pattern_name][0]}' 2>/dev/null"
                size_result = run_command(size_cmd)
                if size_result and size_result[0]:
                    size = size_result[0].split()[0]
                    print(f"  Sample size: {size}")
    
    return artifacts

def analyze_windows_deep_nesting():
    """Find suspiciously deep directory structures on Windows."""
    print("\n=== WINDOWS DEEP NESTING ANALYSIS ===")
    
    cmd = """find /mnt/c -type d 2>/dev/null | head -5000 | awk -F/ 'NF > 12 {print NF-3 ": " $0}' | sort -nr"""
    deep_paths = run_command(cmd)
    
    print(f"Found {len(deep_paths)} directories with > 12 levels of nesting:")
    for path in deep_paths[:15]:  # Show top 15
        print(f"  {path}")
    
    return deep_paths

def analyze_windows_suspicious_patterns():
    """Look for suspicious patterns on Windows that might indicate corruption."""
    print("\n=== WINDOWS SUSPICIOUS PATTERNS ===")
    
    suspicious = {}
    
    # Look for very long path names
    cmd = "find /mnt/c -type d 2>/dev/null | head -5000 | awk 'length($0) > 200 {print length($0) \": \" $0}'"
    long_paths = run_command(cmd)
    suspicious['long_paths'] = long_paths
    
    # Look for temp directories with suspicious patterns
    cmd = "find /mnt/c -type d -name '*tmp*' -o -name '*temp*' -o -name '*cache*' 2>/dev/null | head -50"
    temp_dirs = run_command(cmd)
    suspicious['temp_dirs'] = temp_dirs
    
    # Look for duplicate directory structures
    cmd = "find /mnt/c -type d -name 'src' -exec find {} -maxdepth 1 -name 'src' -type d \\; 2>/dev/null"
    src_in_src = run_command(cmd)
    suspicious['src_in_src'] = src_in_src
    
    print("Suspicious patterns found:")
    for pattern_type, items in suspicious.items():
        if items and items[0]:  # Check if we have actual results
            print(f"\n{pattern_type.upper()}: {len(items)} items")
            for item in items[:5]:  # Show first 5
                print(f"    {item}")
    
    return suspicious

def analyze_windows_project_directories():
    """Analyze the specific project directories we saw in the C: drive listing."""
    print("\n=== WINDOWS PROJECT DIRECTORIES ANALYSIS ===")
    
    project_dirs = [
        '/mnt/c/AI-Dev',
        '/mnt/c/DeFlex-AI-OS', 
        '/mnt/c/ArkOS-Test',
        '/mnt/c/.github',
        '/mnt/c/scripts'
    ]
    
    for proj_dir in project_dirs:
        if os.path.exists(proj_dir):
            print(f"\n{proj_dir}:")
            
            # Get size
            cmd = f"du -sh '{proj_dir}' 2>/dev/null"
            size_result = run_command(cmd)
            if size_result and size_result[0]:
                size = size_result[0].split()[0]
                print(f"  Size: {size}")
            
            # Count subdirectories
            cmd = f"find '{proj_dir}' -maxdepth 1 -type d | wc -l"
            subdir_result = run_command(cmd)
            if subdir_result and subdir_result[0].isdigit():
                subdirs = int(subdir_result[0]) - 1  # Subtract 1 for the directory itself
                print(f"  Subdirectories: {subdirs}")
            
            # Look for build artifacts
            cmd = f"find '{proj_dir}' -name 'node_modules' -o -name 'target' -o -name '__pycache__' -o -name 'bin' -o -name 'obj' 2>/dev/null | wc -l"
            artifacts_result = run_command(cmd)
            if artifacts_result and artifacts_result[0].isdigit():
                artifacts = int(artifacts_result[0])
                print(f"  Build artifacts: {artifacts}")

def generate_windows_cleanup_recommendations():
    """Generate recommendations for Windows cleanup."""
    print("\n=== WINDOWS CLEANUP RECOMMENDATIONS ===")
    
    recommendations = []
    
    # Windows-specific cleanup targets
    recommendations.append({
        'category': 'Windows Temp Files',
        'description': 'Windows temporary files and cache:',
        'paths': [
            '/mnt/c/temp/*',
            '/mnt/c/Windows/Temp/*',
            '/mnt/c/Users/*/AppData/Local/Temp/*'
        ],
        'safety': 'SAFE',
        'space_saved': 'Variable'
    })
    
    recommendations.append({
        'category': 'Development Build Artifacts',
        'description': 'Development build artifacts on Windows:',
        'patterns': ['node_modules', 'target', '__pycache__', 'bin', 'obj', 'Debug', 'Release'],
        'safety': 'MOSTLY SAFE',
        'space_saved': 'High',
        'note': 'Check if projects are actively being developed'
    })
    
    recommendations.append({
        'category': 'Duplicate Projects',
        'description': 'Check for duplicate project directories:',
        'paths': [
            '/mnt/c/AI-Dev vs /mnt/c/DeFlex-AI-OS',
            '/mnt/c/ArkOS-Test vs other Ark directories'
        ],
        'safety': 'REVIEW NEEDED',
        'space_saved': 'High'
    })
    
    for rec in recommendations:
        print(f"\n{rec['category']} ({rec['safety']}):")
        print(f"  {rec['description']}")
        if 'paths' in rec:
            for path in rec['paths']:
                print(f"    {path}")
        if 'patterns' in rec:
            print(f"    Patterns: {', '.join(rec['patterns'])}")
        if 'note' in rec:
            print(f"  Note: {rec['note']}")
        print(f"  Estimated space saved: {rec['space_saved']}")
    
    return recommendations

def main():
    """Main Windows analysis function."""
    print("WINDOWS C: DRIVE CORRUPTION AND REDUNDANCY ANALYSIS")
    print("=" * 55)
    
    results = {}
    
    # Run all analyses
    results['space_usage'] = analyze_windows_space_usage()
    results['repeated_patterns'] = analyze_windows_repeated_patterns()
    results['build_artifacts'] = analyze_windows_build_artifacts()
    results['deep_nesting'] = analyze_windows_deep_nesting()
    results['suspicious_patterns'] = analyze_windows_suspicious_patterns()
    results['project_analysis'] = analyze_windows_project_directories()
    results['recommendations'] = generate_windows_cleanup_recommendations()
    
    # Save results to file
    with open('/home/deflex/ark-ai-os-workspace/windows_corruption_analysis_report.json', 'w') as f:
        json.dump(results, f, indent=2, default=str)
    
    print(f"\n=== WINDOWS ANALYSIS COMPLETE ===")
    print("Full report saved to: windows_corruption_analysis_report.json")

if __name__ == "__main__":
    main()
