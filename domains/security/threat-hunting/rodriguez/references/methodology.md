# Roberto Rodriguez Threat Hunting Methodology

## The Open Threat Research Framework

Rodriguez's approach emphasizes open, reproducible threat research.

### Core Principles

1. **Open Data** - Share datasets publicly
2. **Open Source** - Tools and techniques available to all
3. **Reproducibility** - Anyone can recreate the research
4. **Community** - Collaborative threat intelligence

## HELK: The Hunting ELK

HELK is Rodriguez's flagship hunting platform.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         HELK Stack                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Kibana    │  │   Jupyter   │  │    Spark    │         │
│  │  Dashboard  │  │  Notebooks  │  │  Analytics  │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                 │
│  ┌──────┴────────────────┴────────────────┴──────┐         │
│  │              Elasticsearch                     │         │
│  │         (Search & Analytics Engine)           │         │
│  └──────────────────────┬────────────────────────┘         │
│                         │                                   │
│  ┌──────────────────────┴────────────────────────┐         │
│  │                  Logstash                      │         │
│  │           (Log Processing Pipeline)           │         │
│  └──────────────────────┬────────────────────────┘         │
│                         │                                   │
│  ┌──────────────────────┴────────────────────────┐         │
│  │                   Kafka                        │         │
│  │            (Message Streaming)                │         │
│  └──────────────────────┬────────────────────────┘         │
│                         │                                   │
│  ┌──────────────────────┴────────────────────────┐         │
│  │              Data Sources                      │         │
│  │   (Sysmon, WinEvent, Zeek, Packetbeat...)    │         │
│  └───────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

### Deployment Options

| Option | Description | Use Case |
|--------|-------------|----------|
| HELK Basic | ELK only | Development/Learning |
| HELK + Kafka | With message queue | Production |
| HELK + Spark | With analytics | Advanced hunting |
| HELK Full | All components | Enterprise |

## Mordor Datasets

Pre-recorded attack datasets for research and training.

### Dataset Structure

```
mordor/
├── small_datasets/
│   ├── windows/
│   │   ├── credential_access/
│   │   │   ├── host/
│   │   │   │   └── empire_mimikatz_logonpasswords.zip
│   │   │   └── metadata.yaml
│   │   └── execution/
│   │       └── ...
│   └── linux/
│       └── ...
└── large_datasets/
    └── apt29/
        ├── day1/
        └── day2/
```

### Metadata Format

```yaml
title: Empire Mimikatz Logonpasswords
id: SDWIN-190518201207
author: Roberto Rodriguez @Cyb3rWard0g
creation_date: 2019/05/18
modification_date: 2020/09/20
platform: Windows
attack:
  - technique: T1003.001
    tactic: credential-access
datasets:
  - type: Host
    log_source: Microsoft-Windows-Sysmon/Operational
```

## The Threat Hunter Playbook

### Playbook Structure

```markdown
# Technique Name

## Metadata
- ATT&CK ID: TXXXX
- ATT&CK Tactic: Tactic Name
- Author: @handle
- Creation Date: YYYY/MM/DD

## Technical Description
Detailed explanation of the technique...

## Hypothesis
A hypothesis about observable behaviors...

## Analytics
### Detection Logic
Query or rule for detection...

### Data Sources
- Source 1
- Source 2

## False Positives
Known benign activities that may trigger...

## Validation
Steps to validate the detection...

## References
- Link 1
- Link 2
```

### Hunting Workflow

```
┌──────────────┐
│  Hypothesis  │
│  Generation  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│    Data      │
│  Collection  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Hunting    │◄────┐
│   Session    │     │
└──────┬───────┘     │
       │             │
       ▼             │
┌──────────────┐     │
│   Analysis   │     │ Iterate
│   & Triage   │     │
└──────┬───────┘     │
       │             │
       ▼             │
┌──────────────┐     │
│   Refine     │─────┘
│  Detection   │
└──────────────┘
```

## Jupyter for Hunting

Rodriguez pioneered using Jupyter notebooks for threat hunting.

### Notebook Template

```python
# %% [markdown]
# # Hunting: [Technique Name]
# 
# **ATT&CK ID**: TXXXX
# **Author**: @Cyb3rWard0g

# %% [markdown]
# ## Hypothesis
# Adversaries may use [technique] to [objective].

# %%
# Import libraries
from pyspark.sql import SparkSession
import pandas as pd

# %%
# Initialize Spark
spark = SparkSession.builder \
    .appName("Threat Hunting") \
    .getOrCreate()

# %%
# Load data
df = spark.read.json("/path/to/logs")

# %%
# Hunting query
results = spark.sql("""
    SELECT 
        SourceHostname,
        TargetHostname,
        User,
        ProcessName
    FROM security_events
    WHERE EventID = 4624
    AND LogonType = 10
""")

# %%
# Analyze results
results.show()
```

## Key Analytics Patterns

### Process Creation Chain

```sql
-- Parent-child process relationships
SELECT 
    a.Image as ParentImage,
    b.Image as ChildImage,
    b.CommandLine,
    COUNT(*) as count
FROM sysmon_events a
JOIN sysmon_events b 
    ON a.ProcessGuid = b.ParentProcessGuid
WHERE a.EventCode = 1 
    AND b.EventCode = 1
GROUP BY a.Image, b.Image, b.CommandLine
ORDER BY count DESC
```

### Lateral Movement Detection

```sql
-- RDP connections with process creation
SELECT 
    src.SourceHostname,
    dst.TargetHostname,
    proc.ProcessName,
    proc.CommandLine
FROM network_connections src
JOIN logon_events dst 
    ON src.DestinationIp = dst.IpAddress
JOIN process_creation proc 
    ON dst.LogonId = proc.LogonId
WHERE dst.LogonType = 10
    AND proc.Timestamp > dst.Timestamp
```

## Famous Quotes

> "The best threat hunters are those who can think like an adversary while analyzing data like a scientist."

> "Sharing is caring. Open source threat research benefits everyone except the adversaries."

> "A hypothesis without data is just a guess. Data without a hypothesis is just noise."

## Resources

1. **Threat Hunter Playbook**: https://threathunterplaybook.com
2. **HELK**: https://github.com/Cyb3rWard0g/HELK
3. **Mordor**: https://github.com/OTRF/mordor
4. **OSSEM**: https://github.com/OTRF/OSSEM
