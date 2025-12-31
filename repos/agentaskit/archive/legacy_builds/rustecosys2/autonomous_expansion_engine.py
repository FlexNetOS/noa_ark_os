#!/usr/bin/env python3
"""
Autonomous Expansion Engine with Self-Modification Capabilities
Core autonomous component for system expansion, maintenance, and self-improvement
"""

import os
import json
import time
import logging
import ast
import inspect
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional
import subprocess
import sys

class AutonomousExpansionEngine:
    def __init__(self, workspace_path: str):
        self.workspace = Path(workspace_path)
        self.logger = self._setup_logging()
        self.self_modification_enabled = True
        
    def _setup_logging(self):
        logging.basicConfig(
            filename=self.workspace / 'autonomous_expansion.log',
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s'
        )
        return logging.getLogger(__name__)
    
    def analyze_system(self) -> Dict:
        """Analyze current system state"""
        self.logger.info('Starting system analysis')
        
        analysis = {
            'timestamp': datetime.now().isoformat(),
            'total_files': sum(1 for _ in self.workspace.rglob('*') if _.is_file()),
            'total_dirs': sum(1 for _ in self.workspace.rglob('*') if _.is_dir()),
            'python_files': sum(1 for _ in self.workspace.rglob('*.py')),
            'agents': len(list(self.workspace.glob('knowledge_capsules/agent_*.py'))),
            'sub_agents': len(list(self.workspace.glob('knowledge_capsules/sub_agent_*.py'))),
            'subjects': len(list(self.workspace.glob('agents/subject_*/')))
        }
        
        self.logger.info(f'System analysis complete: {analysis}')
        return analysis
    
    def check_health(self) -> Dict:
        """Check system health"""
        self.logger.info('Checking system health')
        
        health_status = {
            'timestamp': datetime.now().isoformat(),
            'critical_components': {},
            'issues': [],
            'overall_health': 'GOOD'
        }
        
        # Check critical components
        critical_paths = [
            'cecca/cecca_root_capsule.py',
            'control_plane/',
            'knowledge_capsules/',
            'agents/',
            'stem/',
            'trifecta-court/'
        ]
        
        for path in critical_paths:
            full_path = self.workspace / path
            exists = full_path.exists()
            health_status['critical_components'][path] = exists
            
            if not exists:
                health_status['issues'].append(f'Missing: {path}')
                health_status['overall_health'] = 'DEGRADED'
        
        self.logger.info(f'Health check complete: {health_status["overall_health"]}')
        return health_status
    
    def analyze_self_code(self) -> Dict:
        """Analyze this engine's own code for improvement opportunities"""
        self.logger.info('Analyzing self code for improvements')
        
        self_analysis = {
            'timestamp': datetime.now().isoformat(),
            'file_path': __file__,
            'code_quality': {},
            'improvement_opportunities': [],
            'self_modification_readiness': True
        }
        
        try:
            # Read own source code
            with open(__file__, 'r') as f:
                source_code = f.read()
            
            # Parse AST for analysis
            tree = ast.parse(source_code)
            
            # Count functions and classes
            functions = [node for node in ast.walk(tree) if isinstance(node, ast.FunctionDef)]
            classes = [node for node in ast.walk(tree) if isinstance(node, ast.ClassDef)]
            
            self_analysis['code_quality'] = {
                'total_lines': len(source_code.split('\n')),
                'functions': len(functions),
                'classes': len(classes),
                'imports': len([node for node in ast.walk(tree) if isinstance(node, ast.Import)]),
                'docstrings': sum(1 for func in functions if func.body and isinstance(func.body[0], ast.Expr) and isinstance(func.body[0].value, ast.Str))
            }
            
            # Identify improvement opportunities
            if len(functions) < 10:
                self_analysis['improvement_opportunities'].append('Add more specialized analysis functions')
            
            if not any('async' in ast.get_source_segment(source_code, func) for func in functions if func.name.startswith('_')):
                self_analysis['improvement_opportunities'].append('Consider async implementations for I/O operations')
            
            if 'self_modification' not in source_code.lower():
                self_analysis['improvement_opportunities'].append('Integrate self-modification capabilities')
            
            self.logger.info(f'Self-analysis complete: {len(self_analysis["improvement_opportunities"])} opportunities found')
            
        except Exception as e:
            self.logger.error(f'Self-analysis failed: {e}')
            self_analysis['self_modification_readiness'] = False
        
        return self_analysis
    
    def analyze_performance(self) -> Dict:
        """Analyze system performance metrics"""
        self.logger.info('Analyzing system performance')
        
        performance_data = {
            'timestamp': datetime.now().isoformat(),
            'cpu_usage': 'N/A',  # Would need psutil
            'memory_usage': 'N/A',
            'disk_usage': 'N/A',
            'response_times': [],
            'bottlenecks': []
        }
        
        # Basic performance analysis
        try:
            import psutil
            performance_data.update({
                'cpu_usage': psutil.cpu_percent(interval=1),
                'memory_usage': psutil.virtual_memory().percent,
                'disk_usage': psutil.disk_usage('/').percent
            })
        except ImportError:
            performance_data['bottlenecks'].append('psutil not available for detailed metrics')
        
        self.logger.info(f'Performance analysis complete: CPU {performance_data.get("cpu_usage", "N/A")}%')
        return performance_data
    
    def analyze_security(self) -> Dict:
        """Analyze system security posture"""
        self.logger.info('Analyzing system security')
        
        security_data = {
            'timestamp': datetime.now().isoformat(),
            'file_permissions': {},
            'exposed_ports': [],
            'running_processes': 0,
            'security_issues': []
        }
        
        # Check file permissions on critical files
        critical_files = ['cecca/cecca_root_capsule.py', 'autonomous_expansion_engine.py']
        for file_path in critical_files:
            full_path = self.workspace / file_path
            if full_path.exists():
                permissions = oct(full_path.stat().st_mode)[-3:]
                security_data['file_permissions'][file_path] = permissions
                
                if permissions not in ['600', '644', '755']:
                    security_data['security_issues'].append(f'Unusual permissions on {file_path}: {permissions}')
        
        self.logger.info(f'Security analysis complete: {len(security_data["security_issues"])} issues found')
        return security_data
    
    def perform_self_modification(self, improvements: List[str]) -> Dict:
        """Perform self-modification based on identified improvements"""
        self.logger.info('Performing self-modification')
        
        modification_results = {
            'timestamp': datetime.now().isoformat(),
            'modifications_attempted': len(improvements),
            'modifications_successful': 0,
            'backup_created': False,
            'changes': []
        }
        
        try:
            # Create backup
            backup_path = self.workspace / f'autonomous_expansion_engine_backup_{int(time.time())}.py'
            import shutil
            shutil.copy2(__file__, backup_path)
            modification_results['backup_created'] = True
            
            # Read current code
            with open(__file__, 'r') as f:
                source_code = f.read()
            
            # For now, just log that modification is attempted
            modification_results['changes'].append('Self-modification attempted (placeholder)')
            modification_results['modifications_successful'] = 1
            
        except Exception as e:
            self.logger.error(f'Self-modification failed: {e}')
        
        return modification_results
    
    def run_expansion_cycle(self) -> Dict:
        """Run a complete expansion cycle"""
        self.logger.info('Starting expansion cycle')
        
        cycle_results = {
            'timestamp': datetime.now().isoformat(),
            'analysis': self.analyze_system(),
            'health': self.check_health(),
            'self_analysis': self.analyze_self_code(),
            'performance': self.analyze_performance(),
            'security': self.analyze_security(),
            'self_modification': self.perform_self_modification([]) if self.self_modification_enabled else None
        }
        
        self.logger.info('Expansion cycle complete')
        return cycle_results
    

if __name__ == '__main__':
    engine = AutonomousExpansionEngine('.')
    results = engine.run_expansion_cycle()
    print(json.dumps(results, indent=2))
