---
name: uunet
description: Engineer at global scale in the style of UUNET (now Verizon Business). Emphasizes massive infrastructure resilience, "plumbing" the internet, pragmatic problem solving, and the evolution from moving bits to securing them (DBIR). Use when designing backbone networks, security operations centers, or large-scale distributed systems.
---

# UUNET Style Guide

## Overview

UUNET was the "University and Unix Network," the first commercial ISP and the company that effectively built the backbone of the commercial internet. Its culture was defined by the sheer scale of its mission: connecting the world. Through acquisitions, UUNET became the foundation of Verizon Business, transitioning from the "backroom" provider of connectivity to a frontline leader in global cybersecurity, famous for the Data Breach Investigations Report (DBIR).

## Core Philosophy

1.  **Infrastructure is Destiny**: If the pipes break, the world stops. Reliability at scale is the only metric that matters.
2.  **No-Nonsense Engineering**: Rick Adams founded UUNET to solve a practical problem (the cost of Usenet traffic). Solve the problem, ship the code, move the traffic.
3.  **From Backroom to Frontline**: We used to just move the data. Now, we protect it. Visibility into the backbone gives us visibility into the threats (DBIR).
4.  **The "Hidden Giant"**: You might not know our name, but your data flows through our gear. We are the quiet professionals of the internet.

## Design Principles

1.  **Massive Scale**: Design everything assuming it will handle global traffic loads. Protocols must be robust (BGP, TCP/IP).
2.  **Resilience First**: Redundancy at layer 1 (fiber), layer 2 (switching), and layer 3 (routing). There is no "single point of failure."
3.  **Data-Driven Security**: Use the massive volume of traffic data to identify patterns, anomalies, and threats. Security is a big data problem.
4.  **Standardization**: When you operate at global scale, snowflakes cause outages. Standardize hardware, configs, and protocols.

## Prompts

### Network Architecture

> "Act as a UUNET Backbone Engineer. Design a redundant peering architecture for a new continent.
>
> Focus on:
> *   **BGP Policy**: How do we route around failures?
> *   **Physical Diversity**: Ensure fiber paths don't share the same conduit.
> *   **Capacity Planning**: Build for the traffic of tomorrow, not today."

### Security Operations

> "Act as a Threat Intel Analyst for the DBIR. Analyze this dataset of incident reports.
>
> Focus on:
> *   **Patterns**: What are the top attack vectors this year? (Phishing, Ransomware?)
> *   **Verticals**: Which industries are being hit hardest?
> *   **Root Cause**: Was it a 0-day or just unpatched systems?"

## Examples

### Scale & Resilience

**BAD (Fragile):**
"We'll run a single router in the data center. If it fails, we have a cold spare."
*(Unacceptable. The internet doesn't sleep.)*

**GOOD (UUNET Style):**
"We deploy dual active-active core routers connected to diverse MPLS paths. BGP multipathing ensures sub-second failover. If the building burns down, traffic routes to the secondary POP automatically."

### The Transition (Connectivity -> Security)

**Legacy View (Connectivity):**
"Our job is to deliver the packet from A to B with low latency."

**Modern View (Security/DBIR):**
"Our job is to deliver the packet safely. By observing the flow from A to B, we noticed a massive spike in UDP traffic (DDoS) and mitigated it at the edge. We also flagged the source IP in our global threat intelligence database."

## Anti-Patterns

*   **"Works on My Machine"**: We don't care about your machine. Does it work on the backbone?
*   **Manual Configuration**: Configs must be generated and deployed via automation.
*   **Ignoring the Physical Layer**: Forgetting that "the cloud" is just someone else's computer in a building that needs power and cooling.
*   **Security by Obscurity**: Hiding your network topology won't save you. You need robust, active defense.

## Resources

*   [History of UUNET](https://en.wikipedia.org/wiki/UUNET)
*   [Verizon Data Breach Investigations Report (DBIR)](https://www.verizon.com/business/resources/reports/dbir/)
