#!/usr/bin/env python3

"""
NOA Anomaly Detection System
Monitors service metrics and detects unusual patterns using statistical analysis
"""

import json
import time
import statistics
from datetime import datetime, timedelta
from typing import Dict, List, Optional
import os
import sys

class AnomalyDetector:
    def __init__(self, data_dir: str = "data/anomaly_detection"):
        self.data_dir = data_dir
        os.makedirs(data_dir, exist_ok=True)
        self.metrics_history: Dict[str, List[float]] = {}
        self.anomalies: List[Dict] = []

    def record_metric(self, metric_name: str, value: float, timestamp: Optional[datetime] = None):
        """Record a metric value for anomaly detection"""
        if timestamp is None:
            timestamp = datetime.now()

        if metric_name not in self.metrics_history:
            self.metrics_history[metric_name] = []

        self.metrics_history[metric_name].append(value)

        # Keep only last 1000 values to prevent memory issues
        if len(self.metrics_history[metric_name]) > 1000:
            self.metrics_history[metric_name] = self.metrics_history[metric_name][-1000:]

        # Check for anomalies
        anomaly = self.detect_anomaly(metric_name, value)
        if anomaly:
            self.anomalies.append({
                'metric': metric_name,
                'value': value,
                'expected_range': anomaly['expected_range'],
                'deviation': anomaly['deviation'],
                'timestamp': timestamp.isoformat(),
                'severity': anomaly['severity']
            })

            # Keep only last 100 anomalies
            if len(self.anomalies) > 100:
                self.anomalies = self.anomalies[-100:]

    def detect_anomaly(self, metric_name: str, value: float) -> Optional[Dict]:
        """Detect if a value is anomalous using statistical methods"""
        history = self.metrics_history[metric_name]

        if len(history) < 10:  # Need minimum data points
            return None

        try:
            mean = statistics.mean(history)
            stdev = statistics.stdev(history)

            # Calculate z-score
            if stdev == 0:
                return None

            z_score = abs(value - mean) / stdev

            # Define anomaly thresholds
            if z_score > 3.0:  # 3 standard deviations
                expected_min = mean - 3 * stdev
                expected_max = mean + 3 * stdev

                severity = 'critical' if z_score > 4.0 else 'warning'

                return {
                    'expected_range': (expected_min, expected_max),
                    'deviation': z_score,
                    'severity': severity
                }

        except statistics.StatisticsError:
            return None

        return None

    def get_recent_anomalies(self, hours: int = 24) -> List[Dict]:
        """Get anomalies from the last N hours"""
        cutoff = datetime.now() - timedelta(hours=hours)

        return [
            anomaly for anomaly in self.anomalies
            if datetime.fromisoformat(anomaly['timestamp']) > cutoff
        ]

    def get_metric_stats(self, metric_name: str) -> Optional[Dict]:
        """Get statistics for a metric"""
        if metric_name not in self.metrics_history:
            return None

        history = self.metrics_history[metric_name]
        if len(history) < 2:
            return None

        return {
            'count': len(history),
            'mean': statistics.mean(history),
            'median': statistics.median(history),
            'std_dev': statistics.stdev(history),
            'min': min(history),
            'max': max(history),
            'latest': history[-1]
        }

    def save_state(self):
        """Save detector state to disk"""
        state = {
            'metrics_history': self.metrics_history,
            'anomalies': self.anomalies,
            'timestamp': datetime.now().isoformat()
        }

        with open(f"{self.data_dir}/anomaly_detector_state.json", 'w') as f:
            json.dump(state, f, indent=2)

    def load_state(self):
        """Load detector state from disk"""
        state_file = f"{self.data_dir}/anomaly_detector_state.json"
        if os.path.exists(state_file):
            try:
                with open(state_file, 'r') as f:
                    state = json.load(f)

                self.metrics_history = state.get('metrics_history', {})
                self.anomalies = state.get('anomalies', [])

                print(f"Loaded anomaly detector state from {state['timestamp']}")
            except Exception as e:
                print(f"Error loading anomaly detector state: {e}")

def monitor_services(detector: AnomalyDetector):
    """Monitor NOA services and record metrics"""
    services = [
        ('conversation', 3001),
        ('personal-assistant', 3002),
        ('development-assistant', 3003),
        ('agent', 3004)
    ]

    while True:
        for service_name, port in services:
            try:
                import requests
                start_time = time.time()
                response = requests.get(f"http://localhost:{port}/health", timeout=5)
                response_time = (time.time() - start_time) * 1000  # Convert to milliseconds

                # Record response time
                detector.record_metric(f"{service_name}_response_time", response_time)

                # Record status (1 for healthy, 0 for unhealthy)
                status = 1 if response.status_code == 200 else 0
                detector.record_metric(f"{service_name}_status", status)

            except Exception as e:
                # Service is down
                detector.record_metric(f"{service_name}_status", 0)
                detector.record_metric(f"{service_name}_response_time", 0)

        # Save state periodically
        detector.save_state()

        time.sleep(30)  # Check every 30 seconds

def main():
    print("Starting NOA Anomaly Detection System...")

    detector = AnomalyDetector()
    detector.load_state()

    try:
        monitor_services(detector)
    except KeyboardInterrupt:
        print("\nShutting down anomaly detection...")
        detector.save_state()

        # Print recent anomalies
        recent_anomalies = detector.get_recent_anomalies(1)
        if recent_anomalies:
            print(f"\nRecent anomalies in the last hour: {len(recent_anomalies)}")
            for anomaly in recent_anomalies[-5:]:  # Show last 5
                print(f"- {anomaly['metric']}: {anomaly['value']} (severity: {anomaly['severity']})")
        else:
            print("\nNo anomalies detected in the last hour.")

if __name__ == "__main__":
    main()
