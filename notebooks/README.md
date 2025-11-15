# NOA ARK OS - Jupyter Notebooks

## Overview

Interactive notebooks for development, analysis, experimentation, and documentation of NOA ARK OS components.

## Directory Structure

```
notebooks/
├── development/          # Development notebooks
├── analysis/            # Data analysis
├── experiments/         # Experiments and R&D
├── tutorials/           # Learning tutorials
├── demos/               # Interactive demos
└── reports/             # Generated reports
```

## Notebooks Categories

### 1. Development (`development/`)

**Purpose**: Development and debugging workflows

**Notebooks**:
- `01_core_os_development.ipynb` - Core OS development
- `02_crc_testing.ipynb` - CRC system testing
- `03_agent_debugging.ipynb` - Agent debugging
- `04_workflow_design.ipynb` - Workflow design
- `05_sandbox_validation.ipynb` - Sandbox validation

### 2. Analysis (`analysis/`)

**Purpose**: Data analysis and metrics

**Notebooks**:
- `performance_analysis.ipynb` - Performance metrics
- `deployment_analysis.ipynb` - Deployment statistics
- `code_quality_metrics.ipynb` - Code quality analysis
- `ai_confidence_trends.ipynb` - AI confidence tracking
- `resource_utilization.ipynb` - Resource usage analysis

### 3. Experiments (`experiments/`)

**Purpose**: Research and experimentation

**Notebooks**:
- `ml_model_experiments.ipynb` - ML model testing
- `optimization_experiments.ipynb` - Performance optimization
- `new_algorithms.ipynb` - Algorithm research
- `integration_prototypes.ipynb` - Integration prototypes

### 4. Tutorials (`tutorials/`)

**Purpose**: Learning and onboarding

**Notebooks**:
- `01_getting_started.ipynb` - Getting started
- `02_crc_workflow.ipynb` - CRC workflow tutorial
- `03_agent_creation.ipynb` - Agent creation
- `04_ci_cd_pipeline.ipynb` - CI/CD tutorial
- `05_observability.ipynb` - Monitoring setup

### 5. Demos (`demos/`)

**Purpose**: Interactive demonstrations

**Notebooks**:
- `complete_system_demo.ipynb` - Full system demo
- `code_drop_demo.ipynb` - Code drop workflow
- `sandbox_merge_demo.ipynb` - Sandbox merge
- `deployment_demo.ipynb` - Deployment process

### 6. Reports (`reports/`)

**Purpose**: Generated reports and documentation

**Notebooks**:
- `weekly_metrics_report.ipynb` - Weekly metrics
- `deployment_report.ipynb` - Deployment summary
- `system_health_report.ipynb` - Health status
- `security_audit_report.ipynb` - Security audit

## Setup

### Prerequisites

```bash
# Install Jupyter
pip install jupyter jupyterlab

# Install Rust kernel (optional)
cargo install evcxr_jupyter
evcxr_jupyter --install

# Install Python dependencies
pip install -r notebooks/requirements.txt
```

### Launch Jupyter

```bash
# Start JupyterLab
jupyter lab

# Or classic Jupyter
jupyter notebook

# Specific port
jupyter lab --port 8888
```

## Digest Research Quickstart

### Authenticate with the gateway

1. Export `NOA_CAPABILITY_TOKEN_SECRET` if you changed the kernel default.
2. Launch the policy gateway and digest agent services (`make run-gateway` or the appropriate profile).
3. Inside a notebook, instantiate the bridge. Tokens are issued automatically for the `research-notebook` client id.

```python
from notebooks.lib.research_bridge import ResearchBridge

bridge = ResearchBridge()
bridge.list_sample_datasets()
```

### Submit research or knowledge calls

```python
from notebooks.lib.research_bridge import GatewayRequestError

try:
    response = bridge.research_query(
        query="Sanitised recap of digest experiments",
        data_sources=["research_notes_sample.csv"],
    )
    print(response)
except GatewayRequestError as exc:
    print(f"Gateway offline: {exc}")
```

### Persist analytics artefacts

All summaries and metrics should land in `storage/analytics/pipelines/` so they are reproducible in CI and reviews.

```python
metrics_path = bridge.push_metrics(
    pipeline_name="digest-agent-metrics",
    metrics={"status": "draft", "source": "research_summary_template"},
)
print(f"Metrics persisted at {metrics_path}")
```

Sanitised starter datasets live in `.workspace/metrics/`. The bridge exposes `sample_dataset_path()` to locate them without hardcoding filesystem logic.

## Notebook Templates

### Development Notebook Template

```python
# NOA ARK OS Development Notebook
# Component: [Component Name]
# Purpose: [Purpose]
# Date: [Date]

import sys
sys.path.append('../')

from noa_core import *
from noa_crc import *
from noa_agents import *

# Configuration
%load_ext autoreload
%autoreload 2

# Setup logging
import logging
logging.basicConfig(level=logging.INFO)

# Notebook content here...
```

### Analysis Notebook Template

```python
# NOA ARK OS Analysis Notebook
# Analysis: [Analysis Type]
# Date: [Date]

import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns

# Configuration
%matplotlib inline
sns.set_style("whitegrid")

# Load data
data = pd.read_csv('data.csv')

# Analysis here...
```

## Common Operations

### Load NOA ARK OS Components

```python
# Load Core OS
from noa_core import CoreOS
core = CoreOS()
core.init()

# Load CRC System
from noa_crc import CRCSystem
crc = CRCSystem()

# Load Agents
from noa_agents import AgentFactory
factory = AgentFactory()
```

### Query Metrics

```python
# Query Prometheus metrics
from prometheus_api_client import PrometheusConnect

prom = PrometheusConnect(url="http://localhost:9090")
metrics = prom.get_current_metric_value(metric_name='http_requests_total')

# Plot metrics
import pandas as pd
df = pd.DataFrame(metrics)
df.plot()
```

### Analyze CL Tree

```python
# Load CL Tree
import json
with open('.workspace/cl_tree/tree.json') as f:
    cl_tree = json.load(f)

# Analyze deployment history
nodes = cl_tree['nodes']
deployed = [n for n in nodes.values() if n['status'] == 'Deployed']
print(f"Total deployments: {len(deployed)}")
```

## Best Practices

### DO:
✅ Version control notebooks (use nbstripout)
✅ Clear outputs before committing
✅ Use markdown cells for documentation
✅ Include setup cells at the top
✅ Use meaningful cell names
✅ Test code before running full notebook

### DON'T:
❌ Commit with outputs (use nbstripout)
❌ Hardcode secrets
❌ Leave debugging code
❌ Create notebooks without documentation
❌ Run untested code in production

## Integration

### With CI/CD

```yaml
# Execute notebooks in CI
test_notebooks:
  script:
    - jupyter nbconvert --execute --to notebook notebooks/tests/*.ipynb
```

### With Documentation

```bash
# Generate HTML documentation
jupyter nbconvert --to html notebooks/tutorials/*.ipynb
mv *.html docs/tutorials/
```

## Requirements

**`notebooks/requirements.txt`**:
```
jupyter>=1.0.0
jupyterlab>=4.0.0
pandas>=2.0.0
numpy>=1.24.0
matplotlib>=3.7.0
seaborn>=0.12.0
plotly>=5.14.0
scikit-learn>=1.2.0
prometheus-api-client>=0.5.0
requests>=2.31.0
```

## Examples

### Performance Analysis Example

```python
import pandas as pd
import matplotlib.pyplot as plt

# Load deployment data
df = pd.read_csv('.workspace/metrics/deployments.csv')

# Calculate average deployment time
avg_time = df['duration_minutes'].mean()
print(f"Average deployment time: {avg_time:.2f} minutes")

# Plot deployment times over time
df.plot(x='date', y='duration_minutes', kind='line')
plt.title('Deployment Duration Trends')
plt.ylabel('Duration (minutes)')
plt.show()
```

### Agent Analysis Example

```python
from noa_agents import AgentFactory, HiveMind

# Create agents
factory = AgentFactory()
hive = HiveMind()

# Analyze agent performance
agents = factory.list_agents()
for agent_id in agents:
    metrics = factory.get_agent_metrics(agent_id)
    print(f"{agent_id}: {metrics['tasks_completed']} tasks completed")
```

## Security

- Never commit notebooks with sensitive data
- Use environment variables for secrets
- Clear outputs before sharing
- Review notebooks before publishing

## Maintenance

```bash
# Update dependencies
pip install --upgrade -r notebooks/requirements.txt

# Clean outputs
jupyter nbconvert --clear-output --inplace notebooks/**/*.ipynb

# Check for secrets
detect-secrets scan notebooks/
```
