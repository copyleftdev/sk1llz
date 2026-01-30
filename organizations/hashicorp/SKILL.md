---
name: hashicorp
description: Build systems and organizations using the "Tao of HashiCorp". Emphasizes workflows over technologies, simple modular composable tools, immutability, and versioning driven by code. Use when designing platform engineering initiatives, DevOps workflows, or infrastructure automation.
---

# HashiCorp Style Guide

## Overview

HashiCorp is defined by its "Tao," a set of engineering principles that guided the creation of tools like Terraform, Consul, Vault, and Nomad. These principles focus on solving the right problems with the right level of abstraction, favoring consistent workflows for users even as the underlying technologies change.

## The Tao of HashiCorp

1.  **Workflows, Not Technologies**: Technologies change (VMs -> Containers -> Serverless), but the workflow (Provision, Secure, Connect, Run) remains the same. Design the workflow first.
2.  **Simple, Modular, Composable**: Do one thing and do it well. Tools should be loosely coupled. The "Unix Philosophy" applied to cloud infrastructure.
3.  **Communicating via Explicit APIs**: Systems should interact through well-defined, versioned APIs, not by sharing database state or internal memory.
4.  **Immutability**: Once something is created, it should not change. To change it, destroy it and create a new version. This applies to infrastructure (servers), artifacts (containers), and configurations.
5.  **Versioning through Codification**: "Infrastructure as Code." Everything—policy, security, networking—should be code, stored in version control, and reviewable.
6.  **Automation through Codification**: Since everything is code, everything can be automated. Human intervention should be the exception.
7.  **Resilient Systems**: Failure is inevitable. Design systems that expect failure and recover automatically (self-healing).
8.  **Pragmatism**: Solve real problems. Don't over-engineer.

## Prompts

### Platform Engineering Design

> "Act as a HashiCorp Systems Architect. Design a platform for our internal developers.
>
> Focus on:
> *   **Workflow Abstraction**: How do developers deploy? Does it change if we move from AWS to Azure? (It shouldn't).
> *   **Composability**: Are we building a monolith platform or composing small tools?
> *   **Self-Service**: How can we use code (HCL, YAML) to let developers serve themselves?"

### Workflow Review

> "Review this CI/CD pipeline against the Tao of HashiCorp.
>
> Questions:
> *   **Immutability**: Are we patching running servers (Mutable) or replacing them (Immutable)?
> *   **Codification**: Is the security policy defined in a GUI or in code (Sentinel/OPA)?
> *   **Explicit APIs**: Are the build and deploy steps decoupled?"

## Examples

### Workflows > Technologies

**BAD (Technology Focused):**
"We need to build a Docker Swarm cluster to run our apps."
*(Fragile. What happens when Kubernetes wins?)*

**GOOD (Workflow Focused):**
"We need a standardized workflow for applications to declare their runtime requirement (cpu, mem, ports) and be scheduled."
*(Robust. The scheduler can be Swarm, Nomad, or K8s. The user workflow (`submit job`) remains constant.)*

### Immutability

**BAD (Mutable Infrastructure):**
```bash
# SSH into server
ssh admin@server-01
apt-get update
apt-get install nginx
# Edit config file effectively in production
vi /etc/nginx/nginx.conf
service nginx reload
```
*(Drift happens. Snowflake servers emerge.)*

**GOOD (Immutable Infrastructure):**
```hcl
# Packer Template to build image
source "amazon-ebs" "ubuntu" {
  ami_name = "nginx-web-{{timestamp}}"
  # ...
}

build {
  sources = ["source.amazon-ebs.ubuntu"]
  provisioner "shell" {
    inline = [
      "apt-get install -y nginx",
      "mv /tmp/nginx.conf /etc/nginx/nginx.conf"
    ]
  }
}
# Result: An AMI ID.
# To update: Build new AMI, replace old instances with new ones via Terraform/ASG.
```

### Codification (Policy as Code)

**BAD:**
Security team reviews a Word document checklist before deployment.

**GOOD (Sentinel/OPA):**
```hcl
# Policy explicitly defined in code
import "tfplan"

main = rule {
  all tfplan.resource_changes as _, rc {
    rc.type is "aws_security_group" implies
      all rc.change.after.ingress as ingress {
        ingress.cidr_blocks not contains "0.0.0.0/0"
      }
  }
}
```
*(Automated. Versioned. Auditable.)*

## Anti-Patterns

*   **The "Golden Hammer"**: Using one tool for everything (e.g., using Terraform to configure OS internals instead of Ansible/Packer).
*   **ClickOps**: Managing infrastructure via the AWS Console UI.
*   **Long-Lived Pets**: Servers that are never destroyed because "we don't know how to rebuild them."
*   **Implicit Dependencies**: Service A reads Service B's database directly.

## Resources

*   [The Tao of HashiCorp](https://www.hashicorp.com/tao-of-hashicorp)
*   [HashiCorp Principles](https://works.hashicorp.com/articles/principles)
