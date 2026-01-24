# Sigma Rules Reference

## Rule Structure

### Complete Template

```yaml
title: Descriptive Title Here
id: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx  # UUID
related:
    - id: yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy
      type: derived  # derived | obsoletes | merged | renamed | similar

status: experimental  # test | stable | deprecated | unsupported

description: |
    Detailed description of what this rule detects.
    Include context about the attack technique.

references:
    - https://attack.mitre.org/techniques/TXXXX/
    - https://relevant-blog-post.com/

author: Your Name
date: 2024/01/15
modified: 2024/02/20

tags:
    - attack.execution
    - attack.t1059.001
    - detection.emerging_threats

logsource:
    category: process_creation
    product: windows
    service: sysmon
    definition: 'Requires Sysmon with process creation logging'

detection:
    selection_process:
        Image|endswith:
            - '\powershell.exe'
            - '\pwsh.exe'
    
    selection_cmdline:
        CommandLine|contains|all:
            - 'Invoke-'
            - 'http'
    
    filter_legitimate:
        CommandLine|contains:
            - 'Microsoft Update'
            - 'Windows Update'
    
    condition: (selection_process and selection_cmdline) and not filter_legitimate

falsepositives:
    - Administrative scripts
    - Software deployment tools
    - Legitimate automation

level: high  # informational | low | medium | high | critical

fields:
    - CommandLine
    - ParentCommandLine
    - User
    - Computer
```

## Detection Modifiers

### String Modifiers

| Modifier | Description | Example |
|----------|-------------|---------|
| `contains` | Substring match | `*value*` |
| `startswith` | Prefix match | `value*` |
| `endswith` | Suffix match | `*value` |
| `all` | All values must match | AND logic |
| `base64` | Base64 decode first | Encoded strings |
| `base64offset` | Base64 with offset | Partial encoding |
| `re` | Regular expression | Complex patterns |
| `cidr` | CIDR notation | IP ranges |
| `expand` | Placeholder expansion | Variables |

### Combining Modifiers

```yaml
detection:
    # Contains any of these
    selection_any:
        CommandLine|contains:
            - 'mimikatz'
            - 'sekurlsa'
    
    # Contains ALL of these
    selection_all:
        CommandLine|contains|all:
            - 'Invoke-'
            - '-enc'
    
    # Ends with (case insensitive by default)
    selection_end:
        Image|endswith: '\cmd.exe'
    
    # Regular expression
    selection_regex:
        CommandLine|re: '.*-[eE][nN][cC]\s+[A-Za-z0-9+/=]{20,}.*'
```

## Log Sources

### Windows Categories

```yaml
# Process creation
logsource:
    category: process_creation
    product: windows

# File events  
logsource:
    category: file_event
    product: windows

# Registry events
logsource:
    category: registry_event
    product: windows

# Network connection
logsource:
    category: network_connection
    product: windows

# DNS queries
logsource:
    category: dns_query
    product: windows

# Image load (DLL)
logsource:
    category: image_load
    product: windows

# Pipe events
logsource:
    category: pipe_created
    product: windows
```

### Specific Services

```yaml
# Windows Security Log
logsource:
    product: windows
    service: security

# Sysmon
logsource:
    product: windows
    service: sysmon

# PowerShell
logsource:
    product: windows
    service: powershell
    category: ps_script

# Windows Defender
logsource:
    product: windows
    service: windefend
```

## Condition Logic

### Basic Operators

```yaml
detection:
    sel1:
        FieldA: value1
    sel2:
        FieldB: value2
    filter:
        FieldC: value3
    
    # AND
    condition: sel1 and sel2
    
    # OR
    condition: sel1 or sel2
    
    # NOT
    condition: sel1 and not filter
    
    # Grouping
    condition: (sel1 or sel2) and not filter
    
    # All of pattern
    condition: all of sel*
    
    # 1 of pattern
    condition: 1 of sel*
```

### Advanced Conditions

```yaml
detection:
    selection:
        Image|endswith: '\cmd.exe'
    
    filter_system:
        User: 'SYSTEM'
    
    filter_parent:
        ParentImage|endswith:
            - '\services.exe'
            - '\svchost.exe'
    
    # Complex condition
    condition: selection and not (filter_system or filter_parent)
```

## Field Mappings

### Common Field Names

| Sigma Field | Sysmon | Windows Security | Elastic ECS |
|-------------|--------|------------------|-------------|
| `Image` | Image | NewProcessName | process.executable |
| `CommandLine` | CommandLine | CommandLine | process.command_line |
| `ParentImage` | ParentImage | ParentProcessName | process.parent.executable |
| `User` | User | SubjectUserName | user.name |
| `Computer` | Computer | Workstation | host.name |
| `TargetFilename` | TargetFilename | ObjectName | file.path |
| `DestinationIp` | DestinationIp | DestAddress | destination.ip |

## Common Detection Patterns

### Process Creation Patterns

```yaml
# Suspicious parent-child relationship
detection:
    selection:
        ParentImage|endswith: '\winword.exe'
        Image|endswith:
            - '\cmd.exe'
            - '\powershell.exe'
    condition: selection
```

### File Write Patterns

```yaml
# Executable written to temp
detection:
    selection:
        TargetFilename|contains: '\Temp\'
        TargetFilename|endswith:
            - '.exe'
            - '.dll'
    condition: selection
```

### Registry Patterns

```yaml
# Run key modification
detection:
    selection:
        TargetObject|contains:
            - '\CurrentVersion\Run'
            - '\CurrentVersion\RunOnce'
        EventType: SetValue
    condition: selection
```

### Network Patterns

```yaml
# Unusual outbound connection
detection:
    selection:
        Initiated: 'true'
        DestinationPort:
            - 4444
            - 5555
            - 8080
    condition: selection
```

## Best Practices

### Do

- Use specific field names
- Include MITRE ATT&CK tags
- Document false positives
- Test with real data
- Use filters for known-good

### Don't

- Overly broad selections
- Missing false positive documentation
- Hardcoded environment-specific values
- Untested rules in production
