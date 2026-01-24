# Pyramid of Pain - Deep Reference

## Original Concept (David Bianco, 2013)

The Pyramid of Pain shows how much "pain" you cause adversaries when you detect and respond to different types of indicators.

```
                    /\
                   /  \        Level 6: TTPs
                  /    \       Behaviors are hardest to change
                 /──────\
                /        \     Level 5: Tools
               /          \    Must find or create new tools
              /────────────\
             /              \  Level 4: Network/Host Artifacts
            /                \ Registry keys, mutex names, etc.
           /──────────────────\
          /                    \    Level 3: Domain Names
         /                      \   Simple to register new ones
        /────────────────────────\
       /                          \  Level 2: IP Addresses
      /                            \ Easy to change infrastructure
     /──────────────────────────────\
    /                                \   Level 1: Hash Values
   /                                  \  Trivial - just recompile
  /____________________________________\
```

## Detailed Level Analysis

### Level 1: Hash Values (Trivial)

**What they are:**
- MD5, SHA1, SHA256 file hashes
- Specific to exact file contents

**Adversary pain:** Near zero
- Recompile binary → new hash
- Add random bytes → new hash
- Polymorphic packers automate this

**Detection value:** Very low
- Only catches exact known samples
- Easily evaded

**Example:**
```
# Blocking a specific malware hash
sha256:e99a18c428cb38d5f260853678922e03
# Adversary adds a NOP instruction → completely new hash
```

### Level 2: IP Addresses (Easy)

**What they are:**
- C2 server IPs
- Exfiltration destinations
- Scanning sources

**Adversary pain:** Low
- Cloud IPs are cheap
- Botnets provide proxy layers
- Tor/VPN hide real infrastructure

**Detection value:** Low-Medium
- Can disrupt current campaign
- Quickly replaced

**Example:**
```
# Block C2 IP
192.168.1.100
# Adversary spins up new VPS in 5 minutes
```

### Level 3: Domain Names (Simple)

**What they are:**
- C2 domains
- Phishing domains
- Malware distribution sites

**Adversary pain:** Low-Medium
- Domain registration is cheap
- DGAs generate thousands of domains
- Fast-flux DNS adds resilience

**Detection value:** Medium
- Takedowns possible
- DNS monitoring valuable
- But easily replaced

### Level 4: Network/Host Artifacts (Annoying)

**What they are:**
- Registry keys
- File paths
- Mutex names
- User agents
- URI patterns
- Certificates

**Adversary pain:** Medium
- Requires reconfiguring tools
- May need new versions
- Some artifacts are hardcoded

**Detection value:** Medium-High
- Catches tool families
- Harder to change than IOCs

**Examples:**
```
Registry: HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Updater
Mutex: Global\MicrosoftUpdaterMutex
User-Agent: Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 5.1; SV1)
URI Pattern: /wp-content/plugins/update.php?id=
```

### Level 5: Tools (Challenging)

**What they are:**
- Attack frameworks (Cobalt Strike, Metasploit)
- Custom malware families
- Exploitation tools

**Adversary pain:** High
- Tools take time to develop
- Buying new tools is expensive
- Behavioral signatures persist

**Detection value:** High
- Catches entire campaigns
- Attribution value
- Forces tool change

**Example:**
```
# Detecting Cobalt Strike beacon patterns
- Sleep patterns
- Named pipe formats
- Malleable C2 profiles
```

### Level 6: TTPs (Tough!)

**What they are:**
- MITRE ATT&CK techniques
- Attack methodologies
- Behavioral patterns

**Adversary pain:** Maximum
- Must change how they operate
- Requires new tradecraft
- Can't easily retrain operators

**Detection value:** Highest
- Catches unknown variants
- Survives tool changes
- Attribution value

**Examples:**
```
T1003.001 - LSASS Memory Access
T1059.001 - PowerShell Execution
T1021.002 - SMB/Windows Admin Shares
```

## Operationalizing the Pyramid

### Detection Investment Strategy

| Level | Investment | Focus |
|-------|------------|-------|
| TTPs | 40% | Build behavioral detections |
| Tools | 25% | Signature tool behaviors |
| Artifacts | 20% | Key artifacts, not all |
| Domains/IPs | 10% | Automated feeds |
| Hashes | 5% | Automated, low effort |

### Detection ROI Calculation

```python
roi = (adversary_pain * detection_durability) / implementation_effort

# TTP detection example
ttp_roi = (high_pain * high_durability) / medium_effort = BEST

# Hash detection example  
hash_roi = (trivial_pain * low_durability) / low_effort = WORST
```

### Pyramid-Aligned Hunt Priorities

1. **Start with TTPs**: "Is anyone using PowerShell to download and execute?"
2. **Add tool signatures**: "Are there Cobalt Strike beacons?"
3. **Layer artifacts**: "Are there known malware registry keys?"
4. **Feed automation**: Domains, IPs, hashes via threat intel feeds

## Common Mistakes

### Over-investing in Hashes
❌ "We have 50 million hash IOCs!"
✅ "We detect the credential dumping behavior regardless of tool"

### Ignoring the Middle
❌ Only TTPs and hashes
✅ Artifacts and tools provide valuable middle ground

### Static Pyramid
❌ Set and forget
✅ Continuous reassessment as adversaries evolve

## References

1. Bianco, David. "The Pyramid of Pain" (2013)
2. SANS Threat Hunting Summit presentations
3. MITRE ATT&CK framework documentation
