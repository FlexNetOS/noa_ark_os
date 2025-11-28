#!/usr/bin/env python3
"""
ARK-AI-OS Corruption Detection and Repair System
Handles src/src pattern corruption and other file integrity issues
"""

import os
import re
import hashlib
import json
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from datetime import datetime
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class CorruptionDetector:
    """Detects and repairs file corruption patterns"""
    
    def __init__(self, workspace_path: str):
        self.workspace_path = Path(workspace_path)
        self.corruption_patterns = {
            'src_repetition': re.compile(r'(src/){3,}'),
            'path_duplication': re.compile(r'(/[^/]+){2,}'),
            'circular_import': re.compile(r'import\s+\w+\s*;\s*import\s+\w+'),
            'broken_unicode': re.compile(r'[^\x00-\x7F]{3,}'),
        }
        self.backup_dir = self.workspace_path / 'corruption_backups'
        self.backup_dir.mkdir(exist_ok=True)
        
    def scan_workspace(self) -> Dict[str, List[str]]:
        """Scan entire workspace for corruption patterns"""
        corrupted_files = {}
        
        for pattern_name, pattern in self.corruption_patterns.items():
            corrupted_files[pattern_name] = []
            
            for file_path in self.workspace_path.rglob('*'):
                if file_path.is_file() and file_path.suffix in ['.py', '.rs', '.yaml', '.yml', '.json']:
                    try:
                        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                            content = f.read()
                            
                        if pattern.search(content):
                            corrupted_files[pattern_name].append(str(file_path))
                            logger.warning(f"Corruption detected in {file_path}: {pattern_name}")
                            
                    except Exception as e:
                        logger.error(f"Error reading {file_path}: {e}")
                        
        return corrupted_files
    
    def repair_file(self, file_path: str, pattern_name: str) -> bool:
        """Repair a corrupted file"""
        file_path = Path(file_path)
        
        try:
            # Create backup
            backup_path = self.backup_dir / f"{file_path.name}.backup_{int(datetime.now().timestamp())}"
            with open(file_path, 'rb') as src, open(backup_path, 'wb') as dst:
                dst.write(src.read())
            
            # Read and repair content
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
            
            original_content = content
            
            if pattern_name == 'src_repetition':
                # Fix src/src pattern
                content = re.sub(r'(src/){2,}', 'src/', content)
            elif pattern_name == 'path_duplication':
                # Fix duplicate path segments
                content = re.sub(r'(/[^/]+)\1+', r'\1', content)
            elif pattern_name == 'circular_import':
                # Remove duplicate imports
                lines = content.split('\n')
                seen_imports = set()
                cleaned_lines = []
                
                for line in lines:
                    if line.strip().startswith('import') or line.strip().startswith('from'):
                        if line not in seen_imports:
                            seen_imports.add(line)
                            cleaned_lines.append(line)
                    else:
                        cleaned_lines.append(line)
                
                content = '\n'.join(cleaned_lines)
            
            # Write repaired content
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            
            # Verify repair
            with open(file_path, 'r', encoding='utf-8') as f:
                repaired_content = f.read()
            
            if pattern_name == 'src_repetition':
                if not re.search(r'(src/){2,}', repaired_content):
                    logger.info(f"Successfully repaired {file_path}")
                    return True
            
            logger.error(f"Failed to repair {file_path}")
            return False
            
        except Exception as e:
            logger.error(f"Error repairing {file_path}: {e}")
            return False
    
    def batch_repair(self, corrupted_files: Dict[str, List[str]]) -> Dict[str, int]:
        """Repair all corrupted files"""
        repair_stats = {}
        
        for pattern_name, files in corrupted_files.items():
            repair_stats[pattern_name] = 0
            
            for file_path in files:
                if self.repair_file(file_path, pattern_name):
                    repair_stats[pattern_name] += 1
        
        return repair_stats
    
    def generate_report(self, corrupted_files: Dict[str, List[str]], repair_stats: Dict[str, int]) -> str:
        """Generate corruption analysis report"""
        report = {
            'timestamp': datetime.now().isoformat(),
            'workspace_path': str(self.workspace_path),
            'corruption_analysis': {
                'total_corrupted_files': sum(len(files) for files in corrupted_files.values()),
                'corruption_types': corrupted_files,
                'repair_statistics': repair_stats
            },
            'recommendations': [
                'Regular corruption scans recommended',
                'Backup strategy implemented',
                'Automated repair system active'
            ]
        }
        
        report_path = self.workspace_path / 'corruption_analysis_report.json'
        with open(report_path, 'w') as f:
            json.dump(report, f, indent=2)
        
        return str(report_path)

def main():
    """Main corruption detection and repair workflow"""
    workspace_path = '/home/deflex/ark-ai-os-workspace'
    
    detector = CorruptionDetector(workspace_path)
    
    print("🔍 Scanning workspace for corruption...")
    corrupted_files = detector.scan_workspace()
    
    total_corrupted = sum(len(files) for files in corrupted_files.values())
    print(f"📊 Found {total_corrupted} corrupted files")
    
    if total_corrupted > 0:
        print("🔧 Starting repair process...")
        repair_stats = detector.batch_repair(corrupted_files)
        
        print("📋 Generating analysis report...")
        report_path = detector.generate_report(corrupted_files, repair_stats)
        
        print(f"✅ Corruption analysis complete. Report saved to: {report_path}")
    else:
        print("✅ No corruption detected in workspace")

if __name__ == '__main__':
    main()
