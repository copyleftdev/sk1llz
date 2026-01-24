# Google Engineering

> "Focus on the user and all else will follow."

## Engineering Philosophy

Google's engineering culture emphasizes scale, reliability, and data-driven decision making. Their practices have shaped how the industry thinks about operating large-scale systems.

## Techniques

### Testing & Quality

- **[Continuous Fuzzing](continuous-fuzzing/)** — OSS-Fuzz, ClusterFuzz, coverage-guided bug finding
  - *Paper*: "OSS-Fuzz - Google's continuous fuzzing service for open source software" (USENIX Security '17)
  - *Impact*: Found 10,000+ bugs across 1,000+ open source projects

### Operations & Reliability

- **[Site Reliability Engineering](sre/)** — Error budgets, toil elimination, SLO-driven operations
  - *Book*: "Site Reliability Engineering: How Google Runs Production Systems"
  - *Impact*: Defined the SRE discipline adopted industry-wide

## Shared Principles

1. **Automate Everything**: If a human does it twice, automate it
2. **Measure Relentlessly**: Data drives decisions
3. **Design for Failure**: Assume components will fail
4. **Error Budgets**: Balance reliability with velocity
5. **Blameless Postmortems**: Learn from failures, don't punish

## How to Use These Skills

1. **Start with SRE** for operational philosophy and metrics
2. **Add Continuous Fuzzing** for proactive bug discovery

## Key Papers & Resources

- [Google SRE Book](https://sre.google/sre-book/table-of-contents/)
- [Google SRE Workbook](https://sre.google/workbook/table-of-contents/)
- [OSS-Fuzz Documentation](https://google.github.io/oss-fuzz/)
- [Google Research Publications](https://research.google/pubs/)
