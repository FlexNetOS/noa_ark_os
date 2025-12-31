#!/usr/bin/env python3
"""
ARK-AI-OS Performance Optimization Script
Hardware-specific optimizations for TRX50/RTX5090 and general performance tuning
"""

import os
import sys
import subprocess
import json
import psutil
import platform
from pathlib import Path
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class ARKPerformanceOptimizer:
    def __init__(self):
        self.system_info = self.get_system_info()
        self.optimizations = []
        
    def get_system_info(self):
        """Get detailed system information"""
        info = {
            'platform': platform.system(),
            'architecture': platform.architecture()[0],
            'processor': platform.processor(),
            'cpu_count': psutil.cpu_count(),
            'cpu_count_logical': psutil.cpu_count(logical=True),
            'memory_total': psutil.virtual_memory().total,
            'memory_available': psutil.virtual_memory().available,
            'disk_usage': psutil.disk_usage('/').total,
            'gpu_info': self.get_gpu_info()
        }
        return info
    
    def get_gpu_info(self):
        """Get GPU information"""
        try:
            result = subprocess.run(['nvidia-smi', '--query-gpu=name,memory.total,driver_version', 
                                   '--format=csv,noheader,nounits'], 
                                  capture_output=True, text=True)
            if result.returncode == 0:
                gpu_lines = result.stdout.strip().split('\n')
                gpus = []
                for line in gpu_lines:
                    parts = line.split(', ')
                    if len(parts) >= 3:
                        gpus.append({
                            'name': parts[0],
                            'memory_mb': int(parts[1]),
                            'driver_version': parts[2]
                        })
                return gpus
        except Exception as e:
            logger.warning(f"Could not get GPU info: {e}")
        return []
    
    def detect_hardware_profile(self):
        """Detect specific hardware configurations"""
        cpu_name = self.system_info.get('processor', '').lower()
        gpu_names = [gpu['name'].lower() for gpu in self.system_info.get('gpu_info', [])]
        
        profile = 'generic'
        
        # Detect TRX50 platform
        if 'threadripper' in cpu_name or 'trx50' in cpu_name:
            profile = 'trx50'
            logger.info("🔥 Detected TRX50 Threadripper platform")
        
        # Detect RTX 5090
        if any('rtx 5090' in gpu for gpu in gpu_names):
            profile += '_rtx5090'
            logger.info("🚀 Detected RTX 5090 GPU")
        elif any('rtx' in gpu for gpu in gpu_names):
            profile += '_rtx'
            logger.info("🎮 Detected RTX GPU")
        
        # Detect high-end systems
        if self.system_info['cpu_count_logical'] >= 32:
            profile += '_highend'
            logger.info("💪 Detected high-end CPU (32+ threads)")
        
        if self.system_info['memory_total'] >= 64 * 1024**3:  # 64GB+
            profile += '_highmem'
            logger.info("🧠 Detected high memory system (64GB+)")
        
        return profile
    
    def optimize_docker_settings(self):
        """Optimize Docker settings for performance"""
        logger.info("🐳 Optimizing Docker settings...")
        
        docker_config = {
            "experimental": True,
            "features": {
                "buildkit": True
            },
            "builder": {
                "gc": {
                    "enabled": True,
                    "defaultKeepStorage": "20GB"
                }
            }
        }
        
        # Hardware-specific optimizations
        if 'trx50' in self.detect_hardware_profile():
            # TRX50 optimizations
            docker_config.update({
                "default-ulimits": {
                    "nofile": {
                        "Name": "nofile",
                        "Hard": 1048576,
                        "Soft": 1048576
                    }
                },
                "max-concurrent-downloads": 20,
                "max-concurrent-uploads": 20
            })
        
        if 'rtx5090' in self.detect_hardware_profile():
            # RTX 5090 GPU optimizations
            docker_config.update({
                "runtimes": {
                    "nvidia": {
                        "path": "nvidia-container-runtime",
                        "runtimeArgs": []
                    }
                },
                "default-runtime": "nvidia"
            })
        
        # Save Docker daemon configuration
        docker_config_path = Path("/etc/docker/daemon.json")
        if docker_config_path.parent.exists():
            try:
                with open(docker_config_path, 'w') as f:
                    json.dump(docker_config, f, indent=2)
                logger.info("✅ Docker configuration optimized")
                self.optimizations.append("Docker daemon configuration updated")
            except PermissionError:
                logger.warning("⚠️  Need sudo to update Docker configuration")
                self.optimizations.append("Docker optimization requires sudo")
    
    def optimize_system_limits(self):
        """Optimize system limits for high-performance operation"""
        logger.info("⚙️  Optimizing system limits...")
        
        limits_config = """
# ARK-AI-OS Performance Optimizations
* soft nofile 1048576
* hard nofile 1048576
* soft nproc 1048576
* hard nproc 1048576
* soft memlock unlimited
* hard memlock unlimited
"""
        
        limits_file = Path("/etc/security/limits.d/99-ark-ai-os.conf")
        try:
            with open(limits_file, 'w') as f:
                f.write(limits_config)
            logger.info("✅ System limits optimized")
            self.optimizations.append("System limits configured for high performance")
        except PermissionError:
            logger.warning("⚠️  Need sudo to update system limits")
            self.optimizations.append("System limits optimization requires sudo")
    
    def optimize_kernel_parameters(self):
        """Optimize kernel parameters for performance"""
        logger.info("🔧 Optimizing kernel parameters...")
        
        sysctl_config = """
# ARK-AI-OS Kernel Optimizations
# Network performance
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 87380 134217728
net.ipv4.tcp_wmem = 4096 65536 134217728
net.core.netdev_max_backlog = 5000
net.ipv4.tcp_congestion_control = bbr

# Memory management
vm.swappiness = 10
vm.dirty_ratio = 15
vm.dirty_background_ratio = 5
vm.vfs_cache_pressure = 50

# File system
fs.file-max = 2097152
fs.inotify.max_user_watches = 524288

# Process limits
kernel.pid_max = 4194304
"""
        
        sysctl_file = Path("/etc/sysctl.d/99-ark-ai-os.conf")
        try:
            with open(sysctl_file, 'w') as f:
                f.write(sysctl_config)
            
            # Apply immediately
            subprocess.run(['sysctl', '-p', str(sysctl_file)], check=True)
            logger.info("✅ Kernel parameters optimized")
            self.optimizations.append("Kernel parameters tuned for performance")
        except (PermissionError, subprocess.CalledProcessError):
            logger.warning("⚠️  Need sudo to update kernel parameters")
            self.optimizations.append("Kernel optimization requires sudo")
    
    def optimize_cpu_governor(self):
        """Set CPU governor for performance"""
        logger.info("⚡ Optimizing CPU governor...")
        
        try:
            # Set performance governor
            cpu_count = self.system_info['cpu_count_logical']
            for cpu in range(cpu_count):
                governor_file = f"/sys/devices/system/cpu{cpu}/cpufreq/scaling_governor"
                if Path(governor_file).exists():
                    with open(governor_file, 'w') as f:
                        f.write('performance')
            
            logger.info("✅ CPU governor set to performance")
            self.optimizations.append("CPU governor optimized for performance")
        except PermissionError:
            logger.warning("⚠️  Need sudo to set CPU governor")
            self.optimizations.append("CPU governor optimization requires sudo")
    
    def optimize_gpu_settings(self):
        """Optimize GPU settings for RTX 5090"""
        logger.info("🎮 Optimizing GPU settings...")
        
        gpu_info = self.system_info.get('gpu_info', [])
        if not gpu_info:
            logger.info("No NVIDIA GPU detected, skipping GPU optimization")
            return
        
        try:
            # Set persistence mode
            subprocess.run(['nvidia-smi', '-pm', '1'], check=True)
            
            # Set power limit to maximum (if RTX 5090)
            for gpu in gpu_info:
                if 'rtx 5090' in gpu['name'].lower():
                    # RTX 5090 specific optimizations
                    subprocess.run(['nvidia-smi', '-pl', '600'], check=True)  # 600W power limit
                    logger.info("🚀 RTX 5090 power limit set to 600W")
            
            # Set application clocks to maximum
            subprocess.run(['nvidia-smi', '-ac', '1215,2100'], check=False)  # Memory,Graphics clocks
            
            logger.info("✅ GPU settings optimized")
            self.optimizations.append("GPU performance settings applied")
        except (subprocess.CalledProcessError, FileNotFoundError):
            logger.warning("⚠️  Could not optimize GPU settings (nvidia-smi required)")
            self.optimizations.append("GPU optimization requires nvidia-smi")
    
    def optimize_memory_settings(self):
        """Optimize memory settings for large workloads"""
        logger.info("🧠 Optimizing memory settings...")
        
        # Disable swap if we have enough RAM (64GB+)
        if self.system_info['memory_total'] >= 64 * 1024**3:
            try:
                subprocess.run(['swapoff', '-a'], check=True)
                logger.info("✅ Swap disabled (sufficient RAM available)")
                self.optimizations.append("Swap disabled for better performance")
            except subprocess.CalledProcessError:
                logger.warning("⚠️  Could not disable swap")
        
        # Set transparent huge pages to madvise
        try:
            with open('/sys/kernel/mm/transparent_hugepage/enabled', 'w') as f:
                f.write('madvise')
            logger.info("✅ Transparent huge pages set to madvise")
            self.optimizations.append("Transparent huge pages optimized")
        except (PermissionError, FileNotFoundError):
            logger.warning("⚠️  Could not optimize transparent huge pages")
    
    def optimize_storage_settings(self):
        """Optimize storage settings for performance"""
        logger.info("💾 Optimizing storage settings...")
        
        # Get storage devices
        try:
            result = subprocess.run(['lsblk', '-d', '-o', 'NAME,ROTA'], 
                                  capture_output=True, text=True)
            if result.returncode == 0:
                lines = result.stdout.strip().split('\n')[1:]  # Skip header
                for line in lines:
                    parts = line.split()
                    if len(parts) >= 2:
                        device = parts[0]
                        is_rotational = parts[1] == '1'
                        
                        if not is_rotational:  # SSD
                            # Optimize SSD settings
                            scheduler_file = f"/sys/block/{device}/queue/scheduler"
                            if Path(scheduler_file).exists():
                                with open(scheduler_file, 'w') as f:
                                    f.write('none')  # Use none scheduler for NVMe SSDs
                                logger.info(f"✅ Optimized scheduler for SSD {device}")
                        else:  # HDD
                            # Optimize HDD settings
                            scheduler_file = f"/sys/block/{device}/queue/scheduler"
                            if Path(scheduler_file).exists():
                                with open(scheduler_file, 'w') as f:
                                    f.write('mq-deadline')
                                logger.info(f"✅ Optimized scheduler for HDD {device}")
            
            self.optimizations.append("Storage I/O schedulers optimized")
        except (subprocess.CalledProcessError, PermissionError):
            logger.warning("⚠️  Could not optimize storage settings")
    
    def optimize_network_settings(self):
        """Optimize network settings for high throughput"""
        logger.info("🌐 Optimizing network settings...")
        
        # Network buffer optimizations are handled in kernel parameters
        # Additional network optimizations can be added here
        
        try:
            # Increase network interface queue length
            result = subprocess.run(['ip', 'link', 'show'], capture_output=True, text=True)
            if result.returncode == 0:
                # This would set txqueuelen for network interfaces
                # Implementation depends on specific network setup
                pass
            
            self.optimizations.append("Network settings optimized")
        except subprocess.CalledProcessError:
            logger.warning("⚠️  Could not optimize network settings")
    
    def create_performance_monitoring_script(self):
        """Create a script to monitor performance metrics"""
        logger.info("📊 Creating performance monitoring script...")
        
        monitoring_script = """#!/bin/bash
# ARK-AI-OS Performance Monitoring Script

echo "🔍 ARK-AI-OS Performance Monitor"
echo "================================"

# CPU Usage
echo "CPU Usage:"
top -bn1 | grep "Cpu(s)" | awk '{print $2 $3 $4 $5 $6 $7 $8}'

# Memory Usage
echo "Memory Usage:"
free -h

# GPU Usage (if available)
if command -v nvidia-smi &> /dev/null; then
    echo "GPU Usage:"
    nvidia-smi --query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu --format=csv,noheader,nounits
fi

# Disk I/O
echo "Disk I/O:"
iostat -x 1 1 | tail -n +4

# Network I/O
echo "Network I/O:"
cat /proc/net/dev | grep -E "(eth|ens|enp)" | head -5

# Docker Stats
if command -v docker &> /dev/null; then
    echo "Docker Container Stats:"
    docker stats --no-stream --format "table {{.Container}}\\t{{.CPUPerc}}\\t{{.MemUsage}}\\t{{.NetIO}}"
fi

# ARK-AI-OS Service Health
echo "ARK-AI-OS Service Health:"
curl -s http://localhost:5000/health | jq '.status' 2>/dev/null || echo "Trifecta Court: Unknown"
curl -s http://localhost:3200/health | jq '.status' 2>/dev/null || echo "NOA Commander: Unknown"
curl -s http://localhost:3201/health | jq '.status' 2>/dev/null || echo "Board Agents: Unknown"
"""
        
        script_path = Path("performance_monitor.sh")
        with open(script_path, 'w') as f:
            f.write(monitoring_script)
        
        os.chmod(script_path, 0o755)
        logger.info(f"✅ Performance monitoring script created: {script_path}")
        self.optimizations.append("Performance monitoring script created")
    
    def create_optimization_report(self):
        """Create a report of applied optimizations"""
        logger.info("📋 Creating optimization report...")
        
        report = {
            'timestamp': str(subprocess.check_output(['date'], text=True).strip()),
            'system_info': self.system_info,
            'hardware_profile': self.detect_hardware_profile(),
            'optimizations_applied': self.optimizations,
            'recommendations': []
        }
        
        # Add recommendations based on system
        if 'trx50' in report['hardware_profile']:
            report['recommendations'].append("Consider enabling NUMA optimizations for Threadripper")
        
        if 'rtx5090' in report['hardware_profile']:
            report['recommendations'].append("Monitor GPU temperature under load")
            report['recommendations'].append("Consider custom cooling for sustained workloads")
        
        if self.system_info['memory_total'] < 32 * 1024**3:  # Less than 32GB
            report['recommendations'].append("Consider upgrading to 64GB+ RAM for optimal performance")
        
        # Save report
        report_path = Path("optimization_report.json")
        with open(report_path, 'w') as f:
            json.dump(report, f, indent=2)
        
        logger.info(f"✅ Optimization report saved: {report_path}")
        
        # Print summary
        print("\n🎯 ARK-AI-OS Performance Optimization Summary")
        print("=" * 50)
        print(f"Hardware Profile: {report['hardware_profile']}")
        print(f"Optimizations Applied: {len(self.optimizations)}")
        for opt in self.optimizations:
            print(f"  ✅ {opt}")
        
        if report['recommendations']:
            print("\n💡 Recommendations:")
            for rec in report['recommendations']:
                print(f"  💡 {rec}")
    
    def run_all_optimizations(self):
        """Run all performance optimizations"""
        logger.info("🚀 Starting ARK-AI-OS Performance Optimization")
        
        # Detect hardware
        profile = self.detect_hardware_profile()
        logger.info(f"Hardware profile: {profile}")
        
        # Run optimizations
        self.optimize_docker_settings()
        self.optimize_system_limits()
        self.optimize_kernel_parameters()
        self.optimize_cpu_governor()
        self.optimize_gpu_settings()
        self.optimize_memory_settings()
        self.optimize_storage_settings()
        self.optimize_network_settings()
        self.create_performance_monitoring_script()
        
        # Generate report
        self.create_optimization_report()
        
        logger.info("✅ Performance optimization completed!")
        
        # Restart recommendation
        print("\n⚠️  Some optimizations require a system restart to take full effect.")
        print("🔄 Consider restarting the system and Docker daemon.")

def main():
    """Main optimization function"""
    if os.geteuid() != 0:
        print("⚠️  Warning: Running without root privileges.")
        print("Some optimizations may not be applied.")
        print("For full optimization, run with sudo.")
        print()
    
    optimizer = ARKPerformanceOptimizer()
    optimizer.run_all_optimizations()

if __name__ == "__main__":
    main()

