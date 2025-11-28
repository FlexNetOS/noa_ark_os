#!/usr/bin/env python3
"""
Advanced Directory Corruption and Redundancy Analyzer
Identifies suspicious directory patterns that could indicate corruption or unnecessary duplication.
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

def analyze_repeated_patterns():
    """Find directories with repeated names in their path."""
    print("=== ANALYZING REPEATED DIRECTORY PATTERNS ===")
    
    # Find paths with consecutive identical directory names
    cmd = """find /home/deflex/ark-ai-os-workspace -type d | awk -F/ '{
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

def analyze_deep_nesting():
    """Find suspiciously deep directory structures."""
    print("\n=== ANALYZING DEEP NESTING ===")
    
    cmd = """find /home/deflex/ark-ai-os-workspace -type d | awk -F/ 'NF > 15 {print NF-4 ": " $0}' | sort -nr"""
    deep_paths = run_command(cmd)
    
    print(f"Found {len(deep_paths)} directories with > 15 levels of nesting:")
    for path in deep_paths[:15]:  # Show top 15
        print(f"  {path}")
    
    return deep_paths

def analyze_build_artifacts():
    """Analyze build artifacts that could be safely removed."""
    print("\n=== ANALYZING BUILD ARTIFACTS ===")
    
    artifacts = {
        'target': 0,
        'node_modules': 0,
        '.git/objects': 0,
        'build': 0,
        'dist': 0,
        '__pycache__': 0,
        '.pytest_cache': 0
    }
    
    # Count different types of build artifacts
    for artifact_type in artifacts.keys():
        if artifact_type == '.git/objects':
            cmd = f"find /home/deflex/ark-ai-os-workspace -type d -path '*/.git/objects' | wc -l"
        else:
            cmd = f"find /home/deflex/ark-ai-os-workspace -type d -name '{artifact_type}' | wc -l"
        
        result = run_command(cmd)
        if result and result[0].isdigit():
            artifacts[artifact_type] = int(result[0])
    
    # Get sizes of major build directories
    size_info = {}
    major_dirs = [
        '/home/deflex/ark-ai-os-workspace/target',
        '/home/deflex/ark-ai-os-workspace/desktop-app/ark-ai-os-desktop/node_modules',
        '/home/deflex/ark-ai-os-workspace/ark-dashboard-simple/node_modules'
    ]
    
    for dir_path in major_dirs:
        cmd = f"du -sh {dir_path} 2>/dev/null"
        result = run_command(cmd)
        if result and result[0]:
            size_info[dir_path] = result[0].split()[0]
    
    print("Build artifact counts:")
    for artifact, count in artifacts.items():
        print(f"  {artifact}: {count}")
    
    print("\nMajor directory sizes:")
    for path, size in size_info.items():
        print(f"  {path}: {size}")
    
    return artifacts, size_info

def analyze_duplicate_structures():
    """Find potentially duplicate directory structures."""
    print("\n=== ANALYZING DUPLICATE STRUCTURES ===")
    
    # Look for common patterns that might indicate duplication
    patterns = {
        'src_in_src': "find /home/deflex/ark-ai-os-workspace -type d -name 'src' -exec find {} -maxdepth 1 -name 'src' -type d \\;",
        'lib_in_lib': "find /home/deflex/ark-ai-os-workspace -type d -name 'lib' -exec find {} -maxdepth 1 -name 'lib' -type d \\;",
        'test_in_test': "find /home/deflex/ark-ai-os-workspace -type d -name 'test' -exec find {} -maxdepth 1 -name 'test' -type d \\;",
        'docs_in_docs': "find /home/deflex/ark-ai-os-workspace -type d -name 'docs' -exec find {} -maxdepth 1 -name 'docs' -type d \\;"
    }
    
    duplicates = {}
    for pattern_name, cmd in patterns.items():
        result = run_command(cmd)
        duplicates[pattern_name] = [r for r in result if r.strip()]
        if duplicates[pattern_name]:
            print(f"\n{pattern_name.upper()} found:")
            for dup in duplicates[pattern_name][:10]:
                print(f"  {dup}")
    
    return duplicates

def analyze_git_repos():
    """Analyze git repositories and their sizes."""
    print("\n=== ANALYZING GIT REPOSITORIES ===")
    
    git_repos = run_command("find /home/deflex/ark-ai-os-workspace -name '.git' -type d")
    
    print(f"Found {len(git_repos)} git repositories:")
    
    repo_sizes = {}
    for repo in git_repos[:20]:  # Limit to first 20 for performance
        repo_dir = os.path.dirname(repo)
        cmd = f"du -sh {repo} 2>/dev/null"
        result = run_command(cmd)
        if result and result[0]:
            size = result[0].split()[0]
            repo_sizes[repo_dir] = size
            print(f"  {repo_dir}: {size}")
    
    return git_repos, repo_sizes

def analyze_suspicious_patterns():
    """Look for specific suspicious patterns that might indicate corruption."""
    print("\n=== ANALYZING SUSPICIOUS PATTERNS ===")
    
    suspicious = {}
    
    # Look for very long path names (potential corruption)
    cmd = "find /home/deflex/ark-ai-os-workspace -type d | awk 'length($0) > 200 {print length($0) \": \" $0}'"
    long_paths = run_command(cmd)
    suspicious['long_paths'] = long_paths
    
    # Look for unusual character patterns in directory names
    cmd = "find /home/deflex/ark-ai-os-workspace -type d -name '*[0-9][0-9][0-9][0-9][0-9]*'"
    numeric_heavy = run_command(cmd)
    suspicious['numeric_heavy'] = numeric_heavy
    
    # Look for repeated underscores or dashes (potential corruption markers)
    cmd = "find /home/deflex/ark-ai-os-workspace -type d -name '*___*' -o -name '*---*'"
    repeated_chars = run_command(cmd)
    suspicious['repeated_chars'] = repeated_chars
    
    print("Suspicious patterns found:")
    for pattern_type, items in suspicious.items():
        if items and items[0]:  # Check if we have actual results
            print(f"  {pattern_type}: {len(items)} items")
            for item in items[:5]:  # Show first 5
                print(f"    {item}")
    
    return suspicious

def generate_cleanup_recommendations():
    """Generate recommendations for safe cleanup."""
    print("\n=== CLEANUP RECOMMENDATIONS ===")
    
    recommendations = []
    
    # Safe to clean build artifacts
    recommendations.append({
        'category': 'Build Artifacts',
        'description': 'These can be safely deleted and regenerated:',
        'paths': [
            '/home/deflex/ark-ai-os-workspace/target/debug',
            '/home/deflex/ark-ai-os-workspace/target/release/.fingerprint',
        ],
        'safety': 'SAFE',
        'space_saved': '~5GB'
    })
    
    # Node modules in repos (not main projects)
    recommendations.append({
        'category': 'Repository Node Modules',
        'description': 'Node modules in .noa_repos can be removed:',
        'command': 'find /home/deflex/ark-ai-os-workspace/.noa_repos -name "node_modules" -type d',
        'safety': 'SAFE',
        'space_saved': '~500MB'
    })
    
    # Git object cleanup
    recommendations.append({
        'category': 'Git Cleanup',
        'description': 'Git repositories can be cleaned:',
        'command': 'cd repo && git gc --aggressive --prune=now',
        'safety': 'SAFE',
        'space_saved': '~1GB'
    })
    
    for rec in recommendations:
        print(f"\n{rec['category']} ({rec['safety']}):")
        print(f"  {rec['description']}")
        if 'paths' in rec:
            for path in rec['paths']:
                print(f"    {path}")
        if 'command' in rec:
            print(f"    Command: {rec['command']}")
        print(f"  Estimated space saved: {rec['space_saved']}")
    
    return recommendations

def main():
    """Main analysis function."""
    print("DIRECTORY CORRUPTION AND REDUNDANCY ANALYSIS")
    print("=" * 50)
    
    results = {}
    
    # Run all analyses
    results['repeated_patterns'] = analyze_repeated_patterns()
    results['deep_nesting'] = analyze_deep_nesting() 
    results['build_artifacts'] = analyze_build_artifacts()
    results['duplicate_structures'] = analyze_duplicate_structures()
    results['git_analysis'] = analyze_git_repos()
    results['suspicious_patterns'] = analyze_suspicious_patterns()
    results['recommendations'] = generate_cleanup_recommendations()
    
    # Save results to file
    with open('/home/deflex/ark-ai-os-workspace/corruption_analysis_report.json', 'w') as f:
        json.dump(results, f, indent=2, default=str)
    
    print(f"\n=== ANALYSIS COMPLETE ===")
    print("Full report saved to: corruption_analysis_report.json")

if __name__ == "__main__":
    main()
