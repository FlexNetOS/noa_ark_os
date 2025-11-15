#!/usr/bin/env python3

"""
NOA Predictive Maintenance System
Analyzes service metrics to predict potential failures and recommend maintenance actions
"""

import json
import time
import statistics
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple
import os
import sys
from collections import defaultdict

class PredictiveMaintenance:
    def __init__(self, data_dir: str = "data/predictive_maintenance"):
        self.data_dir = data_dir
        os.makedirs(data_dir, exist_ok=True)
        self.metrics_history: Dict[str, List[Tuple[datetime, float]]] = defaultdict(list)
        self.maintenance_predictions: List[Dict] = []
        self.failure_patterns: Dict[str, Dict] = {}

    def record_metric(self, metric_name: str, value: float, timestamp: Optional[datetime] = None):
        """Record a metric value with timestamp"""
        if timestamp is None:
            timestamp = datetime.now()

        self.metrics_history[metric_name].append((timestamp, value))

        # Keep only last 1000 values
        if len(self.metrics_history[metric_name]) > 1000:
            self.metrics_history[metric_name] = self.metrics_history[metric_name][-1000:]

        # Analyze for maintenance needs
        self.analyze_metric_trends(metric_name)

    def analyze_metric_trends(self, metric_name: str):
        """Analyze metric trends to predict maintenance needs"""
        history = self.metrics_history[metric_name]

        if len(history) < 50:  # Need sufficient data
            return

        # Extract values and timestamps
        timestamps, values = zip(*history)

        try:
            # Calculate trend (simple linear regression slope)
            n = len(values)
            x = list(range(n))
            slope = self.calculate_slope(x, list(values))

            # Calculate recent volatility
            recent_values = list(values)[-20:]  # Last 20 points
            volatility = statistics.stdev(recent_values) if len(recent_values) > 1 else 0

            # Predict maintenance based on metric type
            prediction = self.predict_maintenance(metric_name, slope, volatility, values[-1])

            if prediction:
                self.maintenance_predictions.append({
                    'metric': metric_name,
                    'prediction': prediction,
                    'timestamp': datetime.now().isoformat(),
                    'confidence': self.calculate_confidence(slope, volatility),
                    'time_to_failure': prediction.get('time_to_failure')
                })

                # Keep only recent predictions
                cutoff = datetime.now() - timedelta(hours=24)
                self.maintenance_predictions = [
                    p for p in self.maintenance_predictions
                    if datetime.fromisoformat(p['timestamp']) > cutoff
                ]

        except Exception as e:
            print(f"Error analyzing {metric_name}: {e}")

    def calculate_slope(self, x: List[float], y: List[float]) -> float:
        """Calculate linear regression slope"""
        n = len(x)
        sum_x = sum(x)
        sum_y = sum(y)
        sum_xy = sum(xi * yi for xi, yi in zip(x, y))
        sum_x2 = sum(xi * xi for xi in x)

        denominator = n * sum_x2 - sum_x * sum_x
        if denominator == 0:
            return 0

        return (n * sum_xy - sum_x * sum_y) / denominator

    def predict_maintenance(self, metric_name: str, slope: float, volatility: float, current_value: float) -> Optional[Dict]:
        """Predict maintenance needs based on metric analysis"""

        if 'response_time' in metric_name:
            # Response time predictions
            if slope > 0.1 and current_value > 1000:  # Increasing response time
                return {
                    'type': 'performance_degradation',
                    'description': f'Response time increasing for {metric_name}',
                    'recommended_action': 'Optimize service performance, check resource usage',
                    'urgency': 'high' if current_value > 2000 else 'medium',
                    'time_to_failure': '1-3 days'
                }

        elif 'memory' in metric_name:
            # Memory usage predictions
            if slope > 1000 and current_value > 500000:  # KB, increasing memory usage
                return {
                    'type': 'memory_leak',
                    'description': f'Memory usage increasing for {metric_name}',
                    'recommended_action': 'Check for memory leaks, restart service if necessary',
                    'urgency': 'high' if current_value > 800000 else 'medium',
                    'time_to_failure': 'hours to days'
                }

        elif 'cpu' in metric_name:
            # CPU usage predictions
            if slope > 0.5 and current_value > 80:  # Increasing CPU usage
                return {
                    'type': 'cpu_overload',
                    'description': f'CPU usage trending high for {metric_name}',
                    'recommended_action': 'Monitor CPU-intensive operations, consider scaling',
                    'urgency': 'medium',
                    'time_to_failure': '1-2 days'
                }

        elif 'status' in metric_name:
            # Service status predictions
            recent_statuses = [v for _, v in self.metrics_history[metric_name][-10:]]
            failure_rate = sum(1 for s in recent_statuses if s == 0) / len(recent_statuses)

            if failure_rate > 0.3:  # 30% failure rate
                return {
                    'type': 'service_instability',
                    'description': f'High failure rate detected for {metric_name}',
                    'recommended_action': 'Investigate service logs, check dependencies, consider restart',
                    'urgency': 'critical',
                    'time_to_failure': 'immediate'
                }

        return None

    def calculate_confidence(self, slope: float, volatility: float) -> float:
        """Calculate confidence score for prediction (0-1)"""
        # Higher slope and volatility = higher confidence
        slope_confidence = min(abs(slope) / 10.0, 1.0)
        volatility_confidence = min(volatility / 100.0, 1.0)

        return (slope_confidence + volatility_confidence) / 2.0

    def get_maintenance_recommendations(self, hours: int = 24) -> List[Dict]:
        """Get maintenance recommendations from the last N hours"""
        cutoff = datetime.now() - timedelta(hours=hours)

        return [
            pred for pred in self.maintenance_predictions
            if datetime.fromisoformat(pred['timestamp']) > cutoff
        ]

    def get_service_health_score(self, service_name: str) -> Optional[float]:
        """Calculate overall health score for a service (0-100)"""
        metrics = [k for k in self.metrics_history.keys() if k.startswith(f"{service_name}_")]

        if not metrics:
            return None

        total_score = 0.0
        count = 0

        for metric in metrics:
            history = self.metrics_history[metric]
            if len(history) < 5:
                continue

            recent_values = [v for _, v in history[-5:]]
            avg_value = statistics.mean(recent_values)

            # Calculate metric-specific score
            if 'response_time' in metric:
                score = max(0, 100 - (avg_value / 10))  # Lower response time = higher score
            elif 'status' in metric:
                score = avg_value * 100  # Status 1.0 = 100 points
            elif 'memory' in metric:
                score = max(0, 100 - (avg_value / 10000))  # Lower memory usage = higher score
            elif 'cpu' in metric:
                score = max(0, 100 - avg_value)  # Lower CPU usage = higher score
            else:
                score = 50  # Neutral score for unknown metrics

            total_score += score
            count += 1

        return total_score / count if count > 0 else None

    def save_state(self):
        """Save maintenance system state"""
        # Convert datetime objects to strings for JSON serialization
        serializable_history = {}
        for metric, data in self.metrics_history.items():
            serializable_history[metric] = [(ts.isoformat(), val) for ts, val in data]

        state = {
            'metrics_history': serializable_history,
            'maintenance_predictions': self.maintenance_predictions,
            'failure_patterns': self.failure_patterns,
            'timestamp': datetime.now().isoformat()
        }

        with open(f"{self.data_dir}/predictive_maintenance_state.json", 'w') as f:
            json.dump(state, f, indent=2)

    def load_state(self):
        """Load maintenance system state"""
        state_file = f"{self.data_dir}/predictive_maintenance_state.json"
        if os.path.exists(state_file):
            try:
                with open(state_file, 'r') as f:
                    state = json.load(f)

                # Convert back to datetime objects
                for metric, data in state.get('metrics_history', {}).items():
                    self.metrics_history[metric] = [(datetime.fromisoformat(ts), val) for ts, val in data]

                self.maintenance_predictions = state.get('maintenance_predictions', [])
                self.failure_patterns = state.get('failure_patterns', {})

                print(f"Loaded predictive maintenance state from {state['timestamp']}")
            except Exception as e:
                print(f"Error loading predictive maintenance state: {e}")

def monitor_and_predict(pm: PredictiveMaintenance):
    """Monitor services and generate maintenance predictions"""
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
                response_time = (time.time() - start_time) * 1000

                pm.record_metric(f"{service_name}_response_time", response_time)

                status = 1 if response.status_code == 200 else 0
                pm.record_metric(f"{service_name}_status", status)

                # Get additional metrics if available
                try:
                    metrics_response = requests.get(f"http://localhost:{port}/metrics", timeout=5)
                    if metrics_response.status_code == 200:
                        # Parse metrics (assuming JSON format)
                        metrics_data = metrics_response.json()
                        for key, value in metrics_data.items():
                            if isinstance(value, (int, float)):
                                pm.record_metric(f"{service_name}_{key}", value)
                except:
                    pass  # Metrics endpoint not available

            except Exception as e:
                pm.record_metric(f"{service_name}_status", 0)
                pm.record_metric(f"{service_name}_response_time", 0)

        # Save state periodically
        pm.save_state()

        time.sleep(60)  # Check every minute

def main():
    print("Starting NOA Predictive Maintenance System...")

    pm = PredictiveMaintenance()
    pm.load_state()

    try:
        monitor_and_predict(pm)
    except KeyboardInterrupt:
        print("\nShutting down predictive maintenance...")
        pm.save_state()

        # Print maintenance recommendations
        recommendations = pm.get_maintenance_recommendations(24)
        if recommendations:
            print(f"\nMaintenance recommendations (last 24h): {len(recommendations)}")
            for rec in recommendations[-5:]:  # Show last 5
                pred = rec['prediction']
                print(f"- {pred['type']}: {pred['description']} (urgency: {pred['urgency']})")
        else:
            print("\nNo maintenance recommendations in the last 24 hours.")

        # Print service health scores
        print("\nService Health Scores:")
        for service in ['conversation', 'personal-assistant', 'development-assistant', 'agent']:
            score = pm.get_service_health_score(service)
            if score is not None:
                print(".1f")

if __name__ == "__main__":
    main()
