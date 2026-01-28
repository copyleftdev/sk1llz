# Google Engineering

> "Focus on the user and all else will follow."

## Engineering Philosophy

Google's engineering culture emphasizes scale, reliability, data-driven decision making, and delightful user experiences. Their practices have shaped how the industry thinks about operating large-scale systems and designing beautiful interfaces.

## Techniques

### Design Systems

- **[Material Design](material-design/)** — Bold graphic design, intentional motion, adaptive layouts, material metaphor
  - *Guidelines*: Material Design 3 (2021) - Dynamic color, tonal palettes, expressive components
  - *Impact*: The industry standard for cross-platform UI design, adopted by millions of apps

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
6. **User-Centric Design**: Beautiful, functional, accessible experiences

## How to Use These Skills

1. **Start with Material Design** for building beautiful, accessible interfaces
2. **Apply SRE** for operational philosophy and metrics
3. **Add Continuous Fuzzing** for proactive bug discovery

## Key Papers & Resources

- [Material Design Guidelines](https://m3.material.io/)
- [Google SRE Book](https://sre.google/sre-book/table-of-contents/)
- [Google SRE Workbook](https://sre.google/workbook/table-of-contents/)
- [OSS-Fuzz Documentation](https://google.github.io/oss-fuzz/)
- [Google Research Publications](https://research.google/pubs/)
