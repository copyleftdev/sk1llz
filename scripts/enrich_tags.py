#!/usr/bin/env python3
"""Batch-enrich SKILL.md frontmatter with domain-specific tags."""

import os
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent

# Map skill ID -> comma-separated rich tags
# These are domain-specific keywords that help the NLP team engine match skills to queries
TAG_MAP = {
    # ─── Languages / Rust ───
    "bos-concurrency-rust": "concurrency, atomics, locks, threads, parallel, memory-model, mutex, sync, async, low-level, systems, performance",
    "hoare-rust-origins": "ownership, borrowing, memory-safety, type-system, systems, compiler, safety, zero-cost-abstractions, performance",
    "klabnik-teaching-rust": "documentation, learning, idioms, traits, generics, error-handling, cargo, crates, beginner-friendly, practical",
    "levien-native-ui-mastery": "gui, native-ui, rendering, gpu, graphics, 2d, vector, canvas, widget, reactive, layout, vello, druid",
    "matsakis-ownership-mastery": "ownership, lifetimes, borrow-checker, type-system, generics, traits, async, compiler, language-design",
    "nichols-practical-rust": "idioms, practical, error-handling, testing, documentation, crates, community, beginner-friendly, patterns",
    "turon-api-design": "api-design, async, futures, ecosystem, library-design, traits, concurrency, ergonomics, public-api",

    # ─── Languages / Go ───
    "cheney-practical-go": "error-handling, interfaces, composition, testing, performance, profiling, simplicity, practical, production",
    "griesemer-go-generics": "generics, type-parameters, interfaces, compiler, language-design, type-system, constraints",
    "kennedy-mechanical-sympathy": "performance, data-oriented, cache, memory, profiling, benchmarking, goroutines, scheduler, hardware",
    "pike-simplicity-first": "simplicity, concurrency, channels, goroutines, composition, unix, plan9, interfaces, communication",

    # ─── Languages / Python ───
    "beazley-deep-python": "generators, coroutines, metaprogramming, decorators, async, concurrency, internals, advanced, performance",
    "hettinger-elegant-python": "itertools, collections, decorators, idioms, readability, functional, generators, clean-code, pythonic",
    "ramalho-fluent-python": "data-model, protocols, type-hints, iterators, generators, descriptors, metaprogramming, advanced, idioms",
    "vanrossum-pythonic": "zen, readability, simplicity, pep8, language-design, clean-code, standard-library, idioms, beginner-friendly",

    # ─── Languages / JavaScript ───
    "abramov-react-mental-models": "react, components, state, hooks, rendering, virtual-dom, ui, frontend, mental-models, declarative",
    "crockford-good-parts": "closures, prototypes, objects, functions, scope, json, patterns, clean-code, browser, web",
    "osmani-patterns-performance": "design-patterns, performance, web, loading, caching, lazy-loading, bundling, optimization, frontend, browser",
    "simpson-you-dont-know-js": "scope, closures, this, prototypes, async, promises, generators, coercion, types, deep-understanding",

    # ─── Languages / TypeScript ───
    "vanderkam-effective-typescript": "types, generics, type-inference, type-guards, utility-types, strict-mode, migration, best-practices, web, frontend",

    # ─── Languages / C++ ───
    "alexandrescu-modern-cpp": "templates, metaprogramming, generic-programming, policy-design, smart-pointers, move-semantics, patterns",
    "meyers-effective-cpp": "raii, smart-pointers, const, move-semantics, templates, exceptions, best-practices, resource-management",
    "parent-better-code": "algorithms, data-structures, value-semantics, no-raw-loops, stl, generic-programming, clean-code",
    "stepanov-generic-programming": "generic-programming, algorithms, iterators, concepts, stl, mathematical, abstraction, templates",
    "stroustrup-cpp-principles": "oop, raii, type-safety, resource-management, templates, exceptions, language-design, performance, systems",
    "sutter-modern-cpp": "concurrency, move-semantics, smart-pointers, const, exceptions, guidelines, modern, best-practices",

    # ─── Languages / Zig ───
    "cro-zig-community": "zig, comptime, allocators, safety, systems, build-system, cross-compilation, simplicity",
    "kelley-zig-philosophy": "zig, comptime, allocators, manual-memory, simplicity, explicit, no-hidden-control-flow, systems, safety",

    # ─── Languages / Java ───
    "bloch-effective-java": "collections, generics, enums, lambdas, streams, concurrency, immutability, builder-pattern, best-practices, api-design",
    "lea-concurrent-java": "concurrency, threads, locks, executors, futures, concurrent-collections, parallel, synchronization, scalability",

    # ─── Languages / Ruby ───
    "metz-practical-oo": "oop, solid, refactoring, dependency-injection, composition, interfaces, testing, clean-code, design-patterns",

    # ─── Domains / Testing ───
    "beck-tdd": "tdd, test-driven, red-green-refactor, unit-testing, design, refactoring, xp, agile, clean-code",
    "fowler-refactoring": "refactoring, code-smells, clean-code, design-patterns, legacy-code, incremental, testing, oop",
    "hamilton-safety-critical": "safety-critical, nasa, fault-tolerance, verification, reliability, real-time, embedded, mission-critical, space",
    "hughes-property-testing": "property-based-testing, quickcheck, generators, shrinking, invariants, fuzzing, random-testing, formal, specification",
    "hunt-pragmatic-programmer": "pragmatic, automation, testing, debugging, estimation, career, tools, learning, craftsmanship",
    "lane-test-strategy": "test-strategy, integration-testing, end-to-end, test-pyramid, ci-cd, coverage, quality, regression, automation",
    "marick-agile-testing": "agile-testing, exploratory, acceptance-testing, test-quadrants, collaboration, user-stories, quality",

    # ─── Domains / Systems Architecture ───
    "brooks-mythical-man-month": "project-management, team-size, complexity, software-engineering, estimation, second-system, communication",
    "conway-system-design": "conways-law, organization, architecture, microservices, team-topology, communication, modular-design",
    "gamma-design-patterns": "design-patterns, oop, factory, observer, strategy, decorator, adapter, composite, singleton, gof",
    "martin-clean-architecture": "clean-architecture, solid, dependency-inversion, layers, hexagonal, ports-adapters, boundaries, oop",
    "ousterhout-philosophy": "software-design, complexity, deep-modules, abstraction, interfaces, documentation, simplicity, readability",
    "raymond-unix-philosophy": "unix, pipes, composition, text-streams, small-tools, modularity, simplicity, cli, shell, filtering",

    # ─── Domains / Databases ───
    "pavlo-database-performance": "databases, query-optimization, indexing, storage-engines, btree, lsm-tree, olap, oltp, benchmarking, sql",
    "stonebraker-database-design": "databases, relational, acid, transactions, query-processing, storage, schema-design, sql, distributed-db",
    "helland-distributed-data": "distributed, transactions, idempotency, scalability, partitioning, replication, eventual-consistency, microservices",

    # ─── Domains / Networking ───
    "gettys-bufferbloat": "networking, latency, bufferbloat, tcp, congestion, aqm, queuing, real-time, bandwidth, packet-loss",
    "s2-geometry-spatial-indexing": "spatial-indexing, geospatial, s2, cells, regions, coordinates, geographic, mapping, location, queries",
    "google-search-architecture": "search, indexing, ranking, crawling, information-retrieval, pagerank, distributed, scale, query, text",

    # ─── Domains / Security ───
    "bianco-pyramid-of-pain": "threat-intelligence, indicators, detection, adversary, ioc, ttps, hunting, defense, attribution",
    "lee-threat-intelligence": "threat-intelligence, cti, diamond-model, kill-chain, attribution, apt, incident-response, analysis",
    "mitre-attack": "mitre, attack-framework, ttps, tactics, techniques, detection, adversary, threat-model, defense, coverage",
    "rodriguez-threat-hunting": "threat-hunting, jupyter, analytics, detection, siem, log-analysis, sigma, endpoint, investigation",
    "roth-detection-engineering": "yara, sigma, detection-rules, malware, ioc, scanning, community, open-source, threat-detection",

    # ─── Domains / API Design ───
    "fielding-rest": "rest, http, stateless, hateoas, caching, uniform-interface, web-api, resources, hypermedia, architecture",
    "mendez-async-api": "async-api, event-driven, message-broker, pubsub, kafka, rabbitmq, websocket, streaming, microservices, mqtt",

    # ─── Domains / CLI Design ───
    "hashimoto-cli-ux": "cli, command-line, terminal, flags, help-text, output-formatting, progressive-disclosure, ux, tool, shell",

    # ─── Domains / Problem Solving ───
    "feynman-first-principles": "first-principles, reasoning, simplification, mental-models, physics, teaching, curiosity, debugging, fundamentals",
    "polya-problem-solving": "heuristics, decomposition, analogy, working-backward, problem-solving, mathematical, strategy, systematic",

    # ─── Organizations ───
    "aqr-factor-investing": "quantitative, factor-investing, risk-management, portfolio, backtesting, finance, statistics, hedge-fund",
    "citadel-low-latency-systems": "trading, low-latency, market-making, hft, fpga, networking, systems, performance, finance, real-time",
    "cloudflare-performance-engineering": "cdn, networking, performance, edge-computing, dns, http, tls, security, proxy, scale, systems, rust",
    "de-shaw-computational-finance": "trading, computational-finance, quantitative, simulation, risk-management, hpc, finance, algorithms",
    "google-continuous-fuzzing": "fuzzing, oss-fuzz, coverage-guided, sanitizers, security, testing, crash, vulnerability, automated",
    "google-material-design": "material-design, ui, ux, design-system, components, animation, accessibility, responsive, mobile, web, visual",
    "google-sre": "sre, reliability, monitoring, slo, error-budget, incident-response, toil, automation, observability, on-call",
    "gremlin-enterprise-chaos": "chaos-engineering, fault-injection, resilience, gameday, blast-radius, reliability, failure-modes, testing",
    "hashicorp": "infrastructure, terraform, vault, consul, nomad, iac, devops, automation, workflows, cloud, deployment",
    "jane-street-functional-trading": "ocaml, functional, trading, market-making, finance, type-safety, correctness, low-latency, quantitative",
    "jump-trading-fpga-hft": "fpga, hft, low-latency, trading, hardware, networking, market-making, colocation, systems, real-time",
    "netflix-chaos-engineering": "chaos-engineering, resilience, fault-tolerance, microservices, distributed, failure, testing, reliability, cloud",
    "renaissance-statistical-arbitrage": "statistical-arbitrage, quantitative, machine-learning, finance, signals, backtesting, hedge-fund, data",
    "two-sigma-ml-at-scale": "machine-learning, data-science, scale, pipelines, finance, quantitative, modeling, infrastructure, ml-ops",
    "uunet": "networking, infrastructure, bgp, peering, isp, routing, scale, backbone, internet, operations",
    "virtu-market-microstructure": "market-microstructure, electronic-trading, order-book, execution, hft, finance, latency, market-making",

    # ─── Paradigms / Distributed ───
    "dean-large-scale-systems": "mapreduce, bigtable, spanner, distributed, scale, fault-tolerance, infrastructure, google, data-processing, parallel",
    "kleppmann-data-intensive": "crdt, replication, streaming, consistency, event-sourcing, databases, partitioning, consensus, data-pipelines, real-time, collaborative",
    "lamport-formal-distributed": "paxos, consensus, tla+, formal-verification, logical-clocks, state-machine-replication, byzantine, distributed-transactions",

    # ─── Paradigms / Functional ───
    "graham-hackers-painters": "lisp, macros, bottom-up, brevity, abstraction, startups, essays, language-design, metaprogramming",
    "hickey-simple-made-easy": "simplicity, immutability, data-oriented, values, state-management, composition, clojure, functional, decoupling",
    "peyton-jones-practical-haskell": "haskell, monads, type-classes, lazy-evaluation, purity, ghc, compiler, type-system, functional, academic",
    "wadler-monadic-elegance": "monads, type-theory, propositions-as-types, lambda-calculus, category-theory, functional, haskell, formal",

    # ─── Paradigms / Systems ───
    "bellard-minimalist-wizardry": "emulators, compilers, minimal-code, qemu, ffmpeg, tinycc, virtualization, systems, performance, low-level",
    "blow-compiler-gamedev": "game-engine, compiler, language-design, performance, jai, game-dev, rendering, real-time, iteration",
    "click-jvm-optimization": "jit, jvm, hotspot, optimization, compiler, garbage-collection, escape-analysis, inlining, performance, sea-of-nodes",
    "hejlsberg-language-design": "typescript, language-design, type-system, c#, turbo-pascal, delphi, generics, ide, tooling, developer-experience",
    "kay-inventing-the-future": "oop, message-passing, smalltalk, late-binding, biological-metaphor, gui, xerox-parc, education, vision",
    "lattner-compiler-infrastructure": "llvm, clang, swift, mlir, compiler, ir, optimization, toolchain, language-design, code-generation",
    "muratori-performance-aware": "performance, profiling, cache, simd, optimization, hardware-sympathy, benchmarking, low-level, data-oriented, systems",
    "pall-jit-mastery": "jit, luajit, trace-compilation, interpreter, bytecode, optimization, dynamic-languages, performance, low-level",
    "ritchie-c-mastery": "c, unix, systems-programming, pointers, memory, standard-library, portable, low-level, kernel, operating-system",
    "thompson-elegant-systems": "unix, utf-8, grep, regex, plan9, systems, encoding, text-processing, simplicity, operating-system, shell, pipes",
    "torvalds-kernel-pragmatism": "linux, kernel, git, systems, open-source, pragmatic, performance, drivers, scheduling, memory-management",

    # ─── Specialists / Security ───
    "forensics-team": "forensics, network-analysis, incident-response, pcap, malware-analysis, memory-forensics, disk-forensics, investigation",

    # ─── Unmapped batch 2 (actual frontmatter IDs) ───
    "lane-api-evangelist": "api, developer-experience, documentation, evangelism, sdk, onboarding, api-design, rest, community",
    "stonebraker-database-architecture": "databases, relational, acid, transactions, query-processing, storage, schema-design, sql, distributed-db",
    "jacobson-network-performance": "networking, performance, measurement, protocols, tcp, congestion, monitoring, throughput, latency",
    "stevens-network-protocols": "networking, tcp, udp, sockets, unix, protocols, ip, http, systems-programming, low-level",
    "polya-how-to-solve-it": "heuristics, decomposition, analogy, working-backward, problem-solving, mathematical, strategy, systematic",
    "mitre-attack-framework": "mitre, attack-framework, ttps, tactics, techniques, detection, adversary, threat-model, defense, coverage",
    "rodriguez-threat-hunter-playbook": "threat-hunting, jupyter, analytics, detection, siem, log-analysis, sigma, endpoint, investigation",
    "fowler": "refactoring, code-smells, clean-code, design-patterns, legacy-code, incremental, testing, oop, architecture",
    "gray-transaction-systems": "transactions, acid, recovery, fault-tolerance, two-phase-commit, databases, reliability, distributed, logging",
    "lamport-distributed-systems": "paxos, consensus, tla+, formal-verification, logical-clocks, state-machine-replication, byzantine, distributed-transactions",
    "lampson-system-design": "system-design, abstraction, interfaces, security, hints, architecture, naming, performance, modularity",
    "vogels-cloud-architecture": "cloud, aws, eventual-consistency, api, microservices, scalability, failure, distributed, serverless, dynamo",
    "bach-exploratory-testing": "exploratory-testing, test-design, heuristics, risk-based, session-based, manual-testing, quality, critical-thinking",
    "beck-test-driven-development": "tdd, test-driven, red-green-refactor, unit-testing, design, refactoring, xp, agile, clean-code",
    "bolton-rapid-software-testing": "rapid-testing, context-driven, heuristics, oracles, risk-based, exploratory, skills, critical-thinking",
    "tigerbeetle-deterministic-simulation": "deterministic-simulation, distributed, consensus, fault-injection, time-compression, correctness, testing, reliability",
    "maciver-hypothesis-testing": "property-based-testing, hypothesis, python, generators, shrinking, fuzzing, automated-testing, strategies",
    "lipton-mutation-testing": "mutation-testing, test-quality, fault-injection, test-effectiveness, coverage, test-generation, quality",
    "hughes-property-based-testing": "property-based-testing, quickcheck, generators, shrinking, invariants, fuzzing, random-testing, formal, specification",
    "scarface-mean-reversion": "mean-reversion, trading, statistical, quantitative, pairs-trading, finance, signals, backtesting, strategy",
    "minervini-swing-trading": "swing-trading, momentum, technical-analysis, stocks, trading, finance, trend-following, risk-management, screening",
    "alexandrescu-modern-cpp-design": "templates, metaprogramming, generic-programming, policy-design, smart-pointers, move-semantics, patterns",
    "parent-no-raw-loops": "algorithms, data-structures, value-semantics, no-raw-loops, stl, generic-programming, clean-code",
    "stroustrup-cpp-style": "oop, raii, type-safety, resource-management, templates, exceptions, language-design, performance, systems",
    "sutter-exceptional-cpp": "concurrency, move-semantics, smart-pointers, const, exceptions, guidelines, modern, best-practices",
    "cox-tooling-excellence": "tooling, go, modules, testing, build-systems, developer-experience, code-review, automation, standard-library",
    "griesemer-precise-go": "generics, type-parameters, interfaces, compiler, language-design, type-system, constraints",
    "thompson-unix-philosophy": "unix, utf-8, grep, regex, plan9, systems, encoding, text-processing, simplicity, operating-system, shell, pipes",
    "abramov-state-composition": "react, components, state, hooks, rendering, virtual-dom, ui, frontend, mental-models, declarative",
    "dodds-testing-practices": "testing, react, javascript, integration-testing, test-library, accessibility, best-practices, frontend, automation",
    "eich-language-fundamentals": "javascript, language-design, prototypes, closures, browser, web, ecmascript, dynamic, scripting",
    "hettinger-idiomatic-python": "itertools, collections, decorators, idioms, readability, functional, generators, clean-code, pythonic",
    "reitz-api-design": "python, api-design, requests, http, packaging, simplicity, developer-experience, library-design, pythonic",
    "ronacher-pragmatic-design": "python, flask, web, wsgi, templating, api, middleware, pragmatic, library-design, click, cli",
    "vanrossum-pythonic-style": "zen, readability, simplicity, pep8, language-design, clean-code, standard-library, idioms, beginner-friendly",
    "cro-practical-zig": "zig, comptime, allocators, safety, systems, build-system, cross-compilation, simplicity",
}


def enrich_skill(skill_path: Path, skill_id: str, tags: str):
    """Add tags to a SKILL.md frontmatter."""
    content = skill_path.read_text()

    if not content.startswith("---"):
        print(f"  SKIP {skill_id}: no frontmatter")
        return False

    parts = content.split("---", 2)
    if len(parts) < 3:
        print(f"  SKIP {skill_id}: invalid frontmatter")
        return False

    frontmatter = parts[1].strip()

    # Check if tags already exist
    if "\ntags:" in frontmatter or frontmatter.startswith("tags:"):
        print(f"  SKIP {skill_id}: already has tags")
        return False

    # Add tags line after description
    new_frontmatter = frontmatter + f"\ntags: {tags}"
    new_content = f"---\n{new_frontmatter}\n---{parts[2]}"

    skill_path.write_text(new_content)
    return True


def main():
    enriched = 0
    skipped = 0
    missing = 0

    # Walk all SKILL.md files
    for skill_path in sorted(REPO_ROOT.rglob("SKILL.md")):
        if "skill-template" in str(skill_path):
            continue

        # Read frontmatter to get skill ID
        content = skill_path.read_text()
        if not content.startswith("---"):
            continue

        parts = content.split("---", 2)
        if len(parts) < 3:
            continue

        # Extract name from frontmatter
        skill_id = None
        for line in parts[1].strip().split("\n"):
            if line.startswith("name:"):
                skill_id = line.split(":", 1)[1].strip()
                break

        if not skill_id:
            skill_id = skill_path.parent.name

        if skill_id in TAG_MAP:
            if enrich_skill(skill_path, skill_id, TAG_MAP[skill_id]):
                enriched += 1
                print(f"  ✓ {skill_id}")
            else:
                skipped += 1
        else:
            missing += 1
            print(f"  ? {skill_id}: no tag mapping defined")

    print(f"\nDone: {enriched} enriched, {skipped} skipped, {missing} unmapped")


if __name__ == "__main__":
    main()
