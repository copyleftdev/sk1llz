# MITRE ATT&CK Key Techniques Reference

## High-Priority Techniques for Hunting

### Initial Access (TA0001)

#### T1566 - Phishing
**Sub-techniques:**
- T1566.001 - Spearphishing Attachment
- T1566.002 - Spearphishing Link
- T1566.003 - Spearphishing via Service

**Hunt queries:**
```
# Email with suspicious attachments
file_extension:(exe OR dll OR js OR vbs OR hta OR ps1) AND
email_attachment:true

# Links to unusual TLDs
url_tld:(ru OR cn OR tk OR top) AND email_body:true
```

### Execution (TA0002)

#### T1059 - Command and Scripting Interpreter
**Sub-techniques:**
- T1059.001 - PowerShell
- T1059.003 - Windows Command Shell
- T1059.005 - Visual Basic
- T1059.007 - JavaScript

**Hunt queries:**
```
# PowerShell download cradle
process_name:powershell.exe AND 
command_line:(*DownloadString* OR *IEX* OR *Invoke-Expression*)

# Encoded PowerShell
command_line:*-enc* OR command_line:*-EncodedCommand*
```

#### T1204 - User Execution
- T1204.001 - Malicious Link
- T1204.002 - Malicious File

### Persistence (TA0003)

#### T1547 - Boot or Logon Autostart Execution
**Sub-techniques:**
- T1547.001 - Registry Run Keys
- T1547.004 - Winlogon Helper DLL
- T1547.009 - Shortcut Modification

**Registry keys to monitor:**
```
HKCU\Software\Microsoft\Windows\CurrentVersion\Run
HKLM\Software\Microsoft\Windows\CurrentVersion\Run
HKCU\Software\Microsoft\Windows\CurrentVersion\RunOnce
HKLM\Software\Microsoft\Windows\CurrentVersion\RunOnce
```

#### T1053 - Scheduled Task/Job
- T1053.005 - Scheduled Task
- T1053.003 - Cron

**Hunt queries:**
```
# Suspicious scheduled task creation
event_id:4698 AND 
NOT user_name:(SYSTEM OR *$)

# schtasks.exe usage
process_name:schtasks.exe AND 
command_line:*/create*
```

### Privilege Escalation (TA0004)

#### T1548 - Abuse Elevation Control Mechanism
- T1548.002 - Bypass User Account Control

**Hunt queries:**
```
# UAC bypass via eventvwr
process_name:eventvwr.exe AND
child_process_name:(cmd.exe OR powershell.exe)

# fodhelper UAC bypass
process_name:fodhelper.exe AND
registry_path:*ms-settings*
```

### Defense Evasion (TA0005)

#### T1070 - Indicator Removal
- T1070.001 - Clear Windows Event Logs
- T1070.004 - File Deletion

**Hunt queries:**
```
# Event log clearing
event_id:(1102 OR 104) OR
command_line:*wevtutil*cl*

# Timestomping
process_name:timestomp* OR
file_modification_time_changed:true
```

#### T1027 - Obfuscated Files or Information
**Hunt queries:**
```
# Base64 in command line
command_line:*base64* OR
command_line:*FromBase64String*

# XOR operations
command_line:*-bxor* OR
command_line:*xor*
```

### Credential Access (TA0006)

#### T1003 - OS Credential Dumping
**Sub-techniques:**
- T1003.001 - LSASS Memory
- T1003.002 - Security Account Manager
- T1003.003 - NTDS
- T1003.004 - LSA Secrets

**Hunt queries:**
```
# LSASS access
target_process:lsass.exe AND
access_mask:(0x1010 OR 0x1410 OR 0x1438)

# SAM registry access
registry_path:*SAM* AND
access_type:read

# ntdsutil.exe usage
process_name:ntdsutil.exe
```

#### T1110 - Brute Force
- T1110.001 - Password Guessing
- T1110.003 - Password Spraying

**Hunt queries:**
```
# Multiple failed logins
event_id:4625 | stats count by src_ip, user
| where count > 5

# Password spray pattern
event_id:4625 | stats dc(user) as users by src_ip
| where users > 10
```

### Discovery (TA0007)

#### T1087 - Account Discovery
**Hunt queries:**
```
# net user enumeration
process_name:net.exe AND
command_line:(*user* OR *group* OR *localgroup*)

# AD enumeration
process_name:dsquery.exe OR
command_line:*Get-ADUser*
```

#### T1082 - System Information Discovery
**Hunt queries:**
```
# systeminfo execution
process_name:systeminfo.exe

# WMI queries
command_line:*wmic* AND
command_line:(*os* OR *computersystem* OR *process*)
```

### Lateral Movement (TA0008)

#### T1021 - Remote Services
- T1021.001 - Remote Desktop Protocol
- T1021.002 - SMB/Windows Admin Shares
- T1021.003 - Distributed Component Object Model
- T1021.006 - Windows Remote Management

**Hunt queries:**
```
# PsExec-like behavior
process_name:psexec.exe OR
named_pipe:*PSEXESVC*

# WMI lateral movement
process_name:wmiprvse.exe AND
parent_process_name:(cmd.exe OR powershell.exe)

# RDP connections
event_id:4624 AND logon_type:10
```

#### T1570 - Lateral Tool Transfer
**Hunt queries:**
```
# SMB file copy
network_protocol:smb AND
file_operation:write AND
file_extension:(exe OR dll OR ps1)
```

### Collection (TA0009)

#### T1560 - Archive Collected Data
- T1560.001 - Archive via Utility

**Hunt queries:**
```
# Archive creation
process_name:(7z.exe OR rar.exe OR zip.exe) OR
command_line:*Compress-Archive*
```

### Command and Control (TA0011)

#### T1071 - Application Layer Protocol
- T1071.001 - Web Protocols
- T1071.004 - DNS

**Hunt queries:**
```
# DNS tunneling indicators
dns_query_length > 50 OR
dns_query_type:TXT

# Beaconing pattern
network_connection | stats count by dest_ip, time_bucket
| where count > 100 AND is_periodic
```

### Exfiltration (TA0010)

#### T1041 - Exfiltration Over C2 Channel
#### T1567 - Exfiltration Over Web Service
- T1567.002 - Exfiltration to Cloud Storage

**Hunt queries:**
```
# Large outbound transfers
bytes_out > 100000000 AND
dest_port:(443 OR 80)

# Cloud storage uploads
url:(*dropbox* OR *drive.google* OR *onedrive*)
```

## Detection Coverage Matrix

| Tactic | High Priority | Medium Priority | Techniques |
|--------|---------------|-----------------|------------|
| Initial Access | T1566 | T1190, T1133 | 9 |
| Execution | T1059 | T1204, T1053 | 14 |
| Persistence | T1547, T1053 | T1543, T1546 | 19 |
| Privilege Escalation | T1548, T1055 | T1134 | 13 |
| Defense Evasion | T1070, T1027 | T1562, T1036 | 42 |
| Credential Access | T1003, T1110 | T1558 | 17 |
| Discovery | T1087, T1082 | T1083, T1057 | 31 |
| Lateral Movement | T1021, T1570 | T1072 | 9 |
| Collection | T1560, T1005 | T1074 | 17 |
| C2 | T1071, T1095 | T1573 | 16 |
| Exfiltration | T1041, T1567 | T1048 | 9 |
| Impact | T1486, T1490 | T1489 | 13 |
