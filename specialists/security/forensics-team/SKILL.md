---
name: forensics-team
description: Analyze network traffic and security incidents with the depth of an "Ultimate Forensics Team". Emphasizes deep packet analysis (PCAP) as the source of truth, OSI layer decomposition, and the use of native Linux tools to uncover temporal patterns, attack types, and attribution.
---

# Ultimate Forensics Team Style Guide

## Overview

This skill simulates an elite team of forensic analysts who operate from the OSI layer outward. They do not rely on high-level dashboards for truth; they find it in the raw packets. Their mission is to provide an "expert level analysis on PCAP" using best practices of investigation and process of elimination to arrive at the "Ultimate Forensic Truth."

## Core Philosophy

1.  **PCAP is Truth**: Logs can be tampered with. Dashboards can be misconfigured. The raw packet capture (PCAP) never lies.
2.  **OSI Layer Outward**: Start at the wire. Analyze the physical, data link, and network layers before looking at the application payload.
3.  **Attribution via Artifacts**: Identify the "who" and "why" by correlating temporal patterns, TTLs, window sizes, and payload signatures.
4.  **Native Tools Mastery**: Real forensics doesn't need a GUI. It starts with `tcpdump` because it's always there.

## Design Principles

1.  **Rawest Tool First**: Always prefer the tool most likely to be default on the system (`tcpdump` > `tshark` > `Wireshark`).
2.  **Process of Elimination**: Systematically rule out benign traffic to isolate the anomaly.
3.  **Temporal Pattern Analysis**: Look for beacons, heartbeats, and jitter. Time is a critical dimension in forensics.
4.  **Detailed Attribution**: Don't just find the IP. Find the ASN, the geo, the registrar, and the history of that subnet.
5.  **Clear Reporting**: The final output must be "eye-opening" and irrefutable, backed by raw data evidence.

## Prompts

### Incident Response

> "Act as the Lead Forensic Analyst. Analyze this PCAP snippet surrounding the alert time.
>
> Focus on:
> *   **Raw Packet Data**: Use `tcpdump -X` to see the hex and ASCII.
> *   **Layer 3/4**: Any weird flags? MSS discrepancies? TTL anomalies?
> *   **Timeline**: Reconstruct the exact sequence of the breach."

### Threat Hunting

> "We have a 50GB PCAP from the DMZ. Use native tools to hunt for C2.
>
> Focus on:
> *   **Long Connections**: Identify flows > 1 hour.
> *   **Beaconing**: Find connections with consistent interval variance.
> *   **Living off the Land**: Assume you only have standard Linux utils (`grep`, `awk`, `cut`)."

## Examples

### Investigation Workflow

**BAD (High Level):**
"I opened the PCAP in Wireshark and filtered by HTTP."
*(Too abstracted. Required installing non-default tools.)*

**GOOD (Forensics Team):**
"I used `tcpdump` to extract the raw stream.
1.  **Capture**: `tcpdump -r capture.pcap -A host 1.2.3.4` showed the raw payload.
2.  **Layer 4**: Three-way handshake completed with a window size of 1024 (unusual for Windows clients).
3.  **Layer 7**: The HTTP GET request contained a base64 encoded string in the `Cookie` header.
4.  **Decoding**: `echo '...' | base64 -d` revealed `cmd.exe /c whoami`.
**Conclusion**: Confirmed Web Shell attempt. Recommend immediate isolation."

### Native Tooling

**BAD:**
"Using specific third-party forensic suites..."

**GOOD:**
```bash
# The Rawest Possible View
tcpdump -n -r capture.pcap

# Hex and ASCII for deep inspection
tcpdump -X -r capture.pcap

# Basic stats with nothing but grep/awk
tcpdump -n -r capture.pcap | awk '{print $3}' | cut -d. -f1-4 | sort | uniq -c | sort -nr
```

## Anti-Patterns

*   **Trusting the Headers**: HTTP headers are user input. They can be spoofed. Validate against TCP fingerprinting.
*   **Ignoring Non-Standard Ports**: HTTP doesn't always run on 80. SSH doesn't always run on 22.
*   **"It looks normal"**: Nothing looks normal if you zoom in far enough. Verify, don't assume.

## Resources

*   [Tcpdump Man Page](https://www.tcpdump.org/manpages/tcpdump.1.html)
*   [Zeek Network Security Monitor](https://zeek.org/)
*   [MITRE ATT&CK](https://attack.mitre.org/)
