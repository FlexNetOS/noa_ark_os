#!/usr/bin/env python3

"""
NOA Drift Detection and Alerting System
Monitors for deviations in automation behavior and system drift
"""

import json
import hashlib
import time
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Set
import os
import sys
import subprocess
from pathlib import Path

class DriftDetector:
    def __init__(self, project_root: str, data_dir: str = "data/drift_detection"):
        self.project_root = Path(project_root)
        self.data_dir = self.project_root / data_dir
        self.data_dir.mkdir(parents=True, exist_ok=True)

        self.baseline_hashes: Dict[str, str] = {}
        self.drift_alerts: List[Dict] = []
        self.monitored_files: Set[str] = set()

        # Files and directories to monitor for drift
        self.monitor_targets = [
            "services/*/src/**/*.rs",
            "apps/*/src/**/*.rs",
            "frameworks/*/src/**/*.rs",
            "platforms/*/src/**/*.rs",
            "tools/**/*.sh",
            "tools/**/*.py",
            "Cargo.toml",
            "Cargo.lock",
            "*.md",
            "config/**/*.yaml",
            "config/**/*.json",
        ]

    def establish_baseline(self):
        """Establish baseline hashes for all monitored files"""
        print("Establishing drift detection baseline...")

        for pattern in self.monitor_targets:
            try:
                # Use find to locate files matching pattern
                cmd = f"find {self.project_root} -type f -name '{pattern}' 2>/dev/null"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, cwd=self.project_root)

                if result.returncode == 0:
                    files = result.stdout.strip().split('\n')
                    for file_path in files:
                        if file_path and os.path.exists(file_path):
                            file_hash = self.calculate_file_hash(file_path)
                            rel_path = os.path.relpath(file_path, self.project_root)
                            self.baseline_hashes[rel_path] = file_hash
                            self.monitored_files.add(rel_path)

            except Exception as e:
                print(f"Error processing pattern {pattern}: {e}")

        self.save_baseline()
        print(f"Baseline established for {len(self.baseline_hashes)} files")

    def calculate_file_hash(self, file_path: str) -> str:
        """Calculate SHA-256 hash of a file"""
        hash_sha256 = hashlib.sha256()
        try:
            with open(file_path, "rb") as f:
                for chunk in iter(lambda: f.read(4096), b""):
                    hash_sha256.update(chunk)
            return hash_sha256.hexdigest()
        except Exception as e:
            return f"error:{e}"

    def detect_drift(self) -> List[Dict]:
        """Detect drift by comparing current file hashes with baseline"""
        drift_events = []

        for rel_path in self.monitored_files:
            abs_path = self.project_root / rel_path

            if not abs_path.exists():
                # File was deleted
                drift_events.append({
                    'type': 'file_deleted',
                    'file': rel_path,
                    'timestamp': datetime.now().isoformat(),
                    'severity': 'warning',
                    'description': f'Monitored file {rel_path} has been deleted'
                })
                continue

            current_hash = self.calculate_file_hash(str(abs_path))
            baseline_hash = self.baseline_hashes.get(rel_path)

            if baseline_hash != current_hash:
                # File has changed
                severity = self.assess_change_severity(rel_path, baseline_hash, current_hash)

                drift_events.append({
                    'type': 'file_modified',
                    'file': rel_path,
                    'old_hash': baseline_hash,
                    'new_hash': current_hash,
                    'timestamp': datetime.now().isoformat(),
                    'severity': severity,
                    'description': f'File {rel_path} has been modified'
                })

        # Check for new files
        for pattern in self.monitor_targets:
            try:
                cmd = f"find {self.project_root} -type f -name '{pattern}' 2>/dev/null"
                result = subprocess.run(cmd, shell=True, capture_output=True, text=True, cwd=self.project_root)

                if result.returncode == 0:
                    files = result.stdout.strip().split('\n')
                    for file_path in files:
                        if file_path:
                            rel_path = os.path.relpath(file_path, self.project_root)
                            if rel_path not in self.monitored_files:
                                # New file detected
                                drift_events.append({
                                    'type': 'file_added',
                                    'file': rel_path,
                                    'timestamp': datetime.now().isoformat(),
                                    'severity': 'info',
                                    'description': f'New monitored file {rel_path} has been added'
                                })
                                # Add to monitored files
                                self.monitored_files.add(rel_path)

            except Exception as e:
                print(f"Error checking for new files with pattern {pattern}: {e}")

        return drift_events

    def assess_change_severity(self, file_path: str, old_hash: Optional[str], new_hash: str) -> str:
        """Assess the severity of a file change"""
        if old_hash is None:
            return 'info'  # New file

        # Check file extension for severity assessment
        if file_path.endswith(('.rs', '.py', '.sh', '.toml', '.lock')):
            # Critical files - high severity for changes
            return 'high'
        elif file_path.endswith(('.md', '.txt', '.json', '.yaml')):
            # Documentation/config - medium severity
            return 'medium'
        else:
            # Other files - low severity
            return 'low'

    def check_service_drift(self) -> List[Dict]:
        """Check for service behavior drift"""
        drift_events = []

        # Check if services are running as expected
        services = [
            ('conversation', 3001),
            ('personal-assistant', 3002),
            ('development-assistant', 3003),
            ('agent', 3004)
        ]

        for service_name, port in services:
            try:
                import requests
                response = requests.get(f"http://localhost:{port}/health", timeout=5)

                if response.status_code != 200:
                    drift_events.append({
                        'type': 'service_unhealthy',
                        'service': service_name,
                        'port': port,
                        'timestamp': datetime.now().isoformat(),
                        'severity': 'high',
                        'description': f'Service {service_name} is not responding properly (status: {response.status_code})'
                    })

            except requests.exceptions.RequestException:
                drift_events.append({
                    'type': 'service_down',
                    'service': service_name,
                    'port': port,
                    'timestamp': datetime.now().isoformat(),
                    'severity': 'critical',
                    'description': f'Service {service_name} is not accessible'
                })

        return drift_events

    def check_build_drift(self) -> List[Dict]:
        """Check for build system drift"""
        drift_events = []

        try:
            # Check if project still compiles
            result = subprocess.run(
                ["cargo", "check", "--workspace", "--quiet"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )

            if result.returncode != 0:
                drift_events.append({
                    'type': 'build_failure',
                    'timestamp': datetime.now().isoformat(),
                    'severity': 'critical',
                    'description': 'Project build is failing',
                    'details': result.stderr[:500]  # First 500 chars of error
                })

        except subprocess.TimeoutExpired:
            drift_events.append({
                'type': 'build_timeout',
                'timestamp': datetime.now().isoformat(),
                'severity': 'high',
                'description': 'Project build timed out'
            })

        except Exception as e:
            drift_events.append({
                'type': 'build_error',
                'timestamp': datetime.now().isoformat(),
                'severity': 'high',
                'description': f'Build check failed: {e}'
            })

        return drift_events

    def generate_alerts(self, drift_events: List[Dict]):
        """Generate alerts from drift events"""
        alerts = []

        for event in drift_events:
            alert = {
                'id': f"drift_{int(time.time())}_{hash(str(event)) % 10000}",
                'event': event,
                'timestamp': datetime.now().isoformat(),
                'acknowledged': False,
                'escalation_level': self.calculate_escalation_level(event)
            }
            alerts.append(alert)

        # Filter and prioritize alerts
        critical_alerts = [a for a in alerts if a['escalation_level'] >= 3]
        high_alerts = [a for a in alerts if a['escalation_level'] == 2]
        other_alerts = [a for a in alerts if a['escalation_level'] < 2]

        # Keep only most recent alerts per type to avoid spam
        self.drift_alerts.extend(critical_alerts + high_alerts + other_alerts)

        # Keep only last 100 alerts
        if len(self.drift_alerts) > 100:
            self.drift_alerts = self.drift_alerts[-100:]

        return alerts

    def calculate_escalation_level(self, event: Dict) -> int:
        """Calculate escalation level for an event (0-3)"""
        severity = event.get('severity', 'low')

        if event['type'] in ['service_down', 'build_failure']:
            return 3  # Critical
        elif severity == 'critical' or event['type'] == 'build_timeout':
            return 3
        elif severity == 'high' or event['type'] in ['service_unhealthy', 'file_modified']:
            return 2
        elif severity == 'medium':
            return 1
        else:
            return 0

    def get_active_alerts(self) -> List[Dict]:
        """Get currently active (unacknowledged) alerts"""
        return [alert for alert in self.drift_alerts if not alert['acknowledged']]

    def acknowledge_alert(self, alert_id: str):
        """Acknowledge an alert"""
        for alert in self.drift_alerts:
            if alert['id'] == alert_id:
                alert['acknowledged'] = True
                break

    def save_baseline(self):
        """Save baseline hashes to disk"""
        baseline_data = {
            'baseline_hashes': self.baseline_hashes,
            'monitored_files': list(self.monitored_files),
            'timestamp': datetime.now().isoformat()
        }

        with open(self.data_dir / 'drift_baseline.json', 'w') as f:
            json.dump(baseline_data, f, indent=2)

    def load_baseline(self):
        """Load baseline hashes from disk"""
        baseline_file = self.data_dir / 'drift_baseline.json'
        if baseline_file.exists():
            try:
                with open(baseline_file, 'r') as f:
                    data = json.load(f)

                self.baseline_hashes = data.get('baseline_hashes', {})
                self.monitored_files = set(data.get('monitored_files', []))

                print(f"Loaded drift detection baseline from {data['timestamp']}")
                return True
            except Exception as e:
                print(f"Error loading baseline: {e}")

        return False

    def save_alerts(self):
        """Save alerts to disk"""
        with open(self.data_dir / 'drift_alerts.json', 'w') as f:
            json.dump(self.drift_alerts, f, indent=2)

    def load_alerts(self):
        """Load alerts from disk"""
        alerts_file = self.data_dir / 'drift_alerts.json'
        if alerts_file.exists():
            try:
                with open(alerts_file, 'r') as f:
                    self.drift_alerts = json.load(f)
            except Exception as e:
                print(f"Error loading alerts: {e}")

def main():
    if len(sys.argv) < 2:
        print("Usage: python drift_detector.py <command>")
        print("Commands: baseline, monitor, check, alerts")
        sys.exit(1)

    command = sys.argv[1]

    # Get project root (assuming script is in tools/monitoring/drift_detection/)
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent.parent

    detector = DriftDetector(project_root)

    if command == "baseline":
        detector.establish_baseline()
        detector.save_baseline()

    elif command == "monitor":
        print("Starting drift monitoring (press Ctrl+C to stop)...")

        if not detector.load_baseline():
            print("No baseline found. Run 'baseline' command first.")
            sys.exit(1)

        detector.load_alerts()

        try:
            while True:
                # Check for file drift
                file_drift = detector.detect_drift()

                # Check for service drift
                service_drift = detector.check_service_drift()

                # Check for build drift
                build_drift = detector.check_build_drift()

                # Combine all drift events
                all_drift = file_drift + service_drift + build_drift

                if all_drift:
                    alerts = detector.generate_alerts(all_drift)
                    print(f"Generated {len(alerts)} alerts")

                    # Print critical alerts
                    critical_alerts = [a for a in alerts if a['escalation_level'] >= 3]
                    for alert in critical_alerts:
                        print(f"🚨 CRITICAL: {alert['event']['description']}")

                detector.save_alerts()
                time.sleep(300)  # Check every 5 minutes

        except KeyboardInterrupt:
            print("\nStopping drift monitoring...")
            detector.save_alerts()

    elif command == "check":
        if not detector.load_baseline():
            print("No baseline found. Run 'baseline' command first.")
            sys.exit(1)

        drift_events = detector.detect_drift()
        service_drift = detector.check_service_drift()
        build_drift = detector.check_build_drift()

        all_drift = drift_events + service_drift + build_drift

        if all_drift:
            print(f"Detected {len(all_drift)} drift events:")
            for event in all_drift:
                severity_icon = {'critical': '🚨', 'high': '⚠️', 'medium': 'ℹ️', 'low': '📝', 'info': '📝', 'warning': '⚠️'}
                icon = severity_icon.get(event.get('severity', 'info'), '📝')
                print(f"{icon} {event['description']}")
        else:
            print("No drift detected.")

    elif command == "alerts":
        detector.load_alerts()
        active_alerts = detector.get_active_alerts()

        if active_alerts:
            print(f"Active alerts: {len(active_alerts)}")
            for alert in active_alerts:
                event = alert['event']
                severity_icon = {'critical': '🚨', 'high': '⚠️', 'medium': 'ℹ️', 'low': '📝', 'info': '📝', 'warning': '⚠️'}
                icon = severity_icon.get(event.get('severity', 'info'), '📝')
                print(f"{icon} [{alert['id']}] {event['description']}")
        else:
            print("No active alerts.")

    else:
        print(f"Unknown command: {command}")

if __name__ == "__main__":
    main()
