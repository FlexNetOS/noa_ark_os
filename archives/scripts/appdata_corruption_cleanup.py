#!/usr/bin/env python3
"""
AppData Corruption Cleanup Script
Handles broken symbolic links and access-denied files in Windows AppData
"""

import subprocess
import sys
import json
from datetime import datetime

def log_message(message):
    """Log with timestamp"""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{timestamp}] {message}")

def run_windows_command(command, description):
    """Run Windows command with error handling"""
    log_message(f"Executing: {description}")
    log_message(f"Command: {command}")
    
    try:
        # Use cmd.exe to run Windows commands from WSL
        result = subprocess.run(
            ["cmd.exe", "/c", command],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.stdout:
            log_message(f"Success output: {result.stdout[:500]}")
        if result.stderr:
            log_message(f"Error output: {result.stderr[:500]}")
            
        return result.returncode == 0
        
    except subprocess.TimeoutExpired:
        log_message(f"Command timed out after 30 seconds")
        return False
    except Exception as e:
        log_message(f"Command failed: {str(e)}")
        return False

def cleanup_broken_symlinks():
    """Remove broken symbolic links from WindowsApps"""
    log_message("=== CLEANING UP BROKEN SYMBOLIC LINKS ===")
    
    # List of broken symlinks we identified
    broken_symlinks = [
        # Docker symlinks
        r"C:\Users\De-Flex.Net\AppData\Local\Docker\run\dockerInference",
        r"C:\Users\De-Flex.Net\AppData\Local\Docker\run\userAnalyticsOtlpHttp.sock",
        
        # WindowsApps symlinks (sample - there are many more)
        r"C:\Users\De-Flex.Net\AppData\Local\Microsoft\WindowsApps\python.exe",
        r"C:\Users\De-Flex.Net\AppData\Local\Microsoft\WindowsApps\python3.exe",
        r"C:\Users\De-Flex.Net\AppData\Local\Microsoft\WindowsApps\bash.exe",
        r"C:\Users\De-Flex.Net\AppData\Local\Microsoft\WindowsApps\wsl.exe",
        r"C:\Users\De-Flex.Net\AppData\Local\Microsoft\WindowsApps\winget.exe",
    ]
    
    successful_deletions = 0
    
    for symlink in broken_symlinks:
        log_message(f"Attempting to delete broken symlink: {symlink}")
        
        # Try multiple deletion methods
        commands = [
            f'del /f /q "{symlink}"',
            f'rd /s /q "{symlink}"',
            f'attrib -h -s -r "{symlink}" && del /f /q "{symlink}"'
        ]
        
        for cmd in commands:
            if run_windows_command(cmd, f"Delete {symlink}"):
                successful_deletions += 1
                log_message(f"Successfully deleted: {symlink}")
                break
        else:
            log_message(f"Failed to delete: {symlink}")
    
    log_message(f"Deleted {successful_deletions} broken symlinks")

def cleanup_windowsapps_directory():
    """Clean up the entire WindowsApps directory with nuclear approach"""
    log_message("=== NUCLEAR CLEANUP OF WINDOWSAPPS DIRECTORY ===")
    
    windowsapps_path = r"C:\Users\De-Flex.Net\AppData\Local\Microsoft\WindowsApps"
    
    # Nuclear deletion sequence
    commands = [
        f'attrib -h -s -r "{windowsapps_path}\\*.*" /s /d',
        f'takeown /f "{windowsapps_path}" /r /d y',
        f'icacls "{windowsapps_path}" /grant administrators:f /t',
        f'rd /s /q "{windowsapps_path}"',
        f'mkdir "{windowsapps_path}"'
    ]
    
    for cmd in commands:
        run_windows_command(cmd, f"WindowsApps cleanup step")

def cleanup_docker_corruption():
    """Clean up Docker-related corruption"""
    log_message("=== CLEANING UP DOCKER CORRUPTION ===")
    
    docker_path = r"C:\Users\De-Flex.Net\AppData\Local\Docker"
    
    # Stop Docker first
    run_windows_command("taskkill /f /im docker.exe", "Stop Docker")
    run_windows_command("taskkill /f /im com.docker.backend.exe", "Stop Docker Backend")
    
    # Clean up Docker directories
    commands = [
        f'attrib -h -s -r "{docker_path}\\run\\*.*" /s /d',
        f'rd /s /q "{docker_path}\\run"',
        f'mkdir "{docker_path}\\run"'
    ]
    
    for cmd in commands:
        run_windows_command(cmd, "Docker cleanup step")

def cleanup_sfap_corruption():
    """Clean up SFAP cache corruption"""
    log_message("=== CLEANING UP SFAP CORRUPTION ===")
    
    sfap_path = r"C:\Users\De-Flex.Net\AppData\Local\Microsoft\Windows\SFAP"
    
    commands = [
        f'attrib -h -s -r "{sfap_path}\\*.*" /s /d',
        f'takeown /f "{sfap_path}" /r /d y',
        f'icacls "{sfap_path}" /grant administrators:f /t',
        f'rd /s /q "{sfap_path}"'
    ]
    
    for cmd in commands:
        run_windows_command(cmd, "SFAP cleanup step")

def generate_cleanup_report():
    """Generate final cleanup report"""
    log_message("=== GENERATING CLEANUP REPORT ===")
    
    # Check remaining corruption
    check_commands = [
        ('dir /s "C:\\Users\\De-Flex.Net\\AppData\\Local\\Microsoft\\WindowsApps" | find "File(s)"', "WindowsApps file count"),
        ('dir "C:\\Users\\De-Flex.Net\\AppData\\Local\\Docker\\run" 2>nul | find "File(s)"', "Docker run directory"),
        ('dir "C:\\Users\\De-Flex.Net\\AppData\\Local\\Microsoft\\Windows\\SFAP" 2>nul | find "File(s)"', "SFAP directory")
    ]
    
    for cmd, desc in check_commands:
        run_windows_command(cmd, desc)

def main():
    """Main cleanup execution"""
    log_message("Starting AppData corruption cleanup")
    log_message("This will aggressively clean broken symlinks and corrupted directories")
    
    try:
        # Step 1: Clean broken symlinks
        cleanup_broken_symlinks()
        
        # Step 2: Nuclear WindowsApps cleanup
        cleanup_windowsapps_directory()
        
        # Step 3: Docker corruption cleanup
        cleanup_docker_corruption()
        
        # Step 4: SFAP corruption cleanup
        cleanup_sfap_corruption()
        
        # Step 5: Generate report
        generate_cleanup_report()
        
        log_message("AppData corruption cleanup completed!")
        log_message("Recommendation: Restart Windows to complete cleanup")
        
    except Exception as e:
        log_message(f"Cleanup failed with error: {str(e)}")
        sys.exit(1)

if __name__ == "__main__":
    main()
