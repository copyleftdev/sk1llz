# Threat Hunting Masters

> "The goal of threat hunting is to reduce dwell time by finding adversaries before they achieve their objectives."

## The Pantheon

### Framework Architects

- **[David Bianco](bianco/)** — Pyramid of Pain, Threat Hunting Maturity Model
  - *Focus*: Indicator value hierarchy, hunting program maturity
  - *Impact*: Defined how to measure detection effectiveness

- **[MITRE ATT&CK](mitre-attack/)** — Adversary Tactics, Techniques, and Procedures
  - *Focus*: Universal taxonomy of adversary behavior
  - *Impact*: The common language of threat intelligence

### Detection Engineers

- **[Florian Roth](roth/)** — YARA rules, Sigma rules, THOR scanner
  - *Focus*: Detection rule creation, signature sharing
  - *Impact*: Made detection logic portable and shareable

- **[Roberto Rodriguez](rodriguez/)** — Threat Hunter Playbook, HELK
  - *Focus*: Hunt playbooks, open source hunting infrastructure
  - *Impact*: Democratized threat hunting methodology

## Shared Principles

1. **Assume Breach**: Adversaries are already inside
2. **Hypothesis-Driven**: Hunt with intent, not hope
3. **TTPs Over IOCs**: Behaviors are harder to change than indicators
4. **Continuous Improvement**: Every hunt teaches something
5. **Share Knowledge**: The community is stronger together

## The Hunt Cycle

```
Hypothesis → Data Collection → Investigation → Response → Document
     ↑                                                        |
     └────────────────── Learn & Iterate ─────────────────────┘
```

## How to Use These Skills

1. **Start with MITRE ATT&CK** for the common language
2. **Add Bianco** for prioritizing what to hunt
3. **Layer Roth** for detection rule creation
4. **Apply Rodriguez** for hunt playbooks and methodology

## Key Resources

- [MITRE ATT&CK](https://attack.mitre.org/)
- [Sigma Rules](https://sigmahq.io/)
- [Threat Hunter Playbook](https://threathunterplaybook.com/)
- [SANS Threat Hunting](https://www.sans.org/cyber-security-courses/threat-hunting-dfir/)
