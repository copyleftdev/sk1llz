use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use colored::Colorize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn get_manifest_url() -> String {
    std::env::var("SKILLZ_MANIFEST_URL").unwrap_or_else(|_| {
        "https://raw.githubusercontent.com/copyleftdev/sk1llz/master/skills.json".to_string()
    })
}

fn get_raw_base_url() -> String {
    std::env::var("SKILLZ_RAW_BASE_URL").unwrap_or_else(|_| {
        "https://raw.githubusercontent.com/copyleftdev/sk1llz/master".to_string()
    })
}

#[derive(Parser)]
#[command(name = "sk1llz")]
#[command(author = "copyleftdev")]
#[command(version)]
#[command(about = "A package manager for AI coding skills", long_about = None)]
#[command(after_help = "Examples:
  sk1llz list                    List all available skills
  sk1llz search rust             Search for Rust-related skills
  sk1llz install torvalds        Install a skill by name
  sk1llz info lamport            Show details about a skill

Use 'sk1llz <command> --help' for more information about a command.")]
struct Cli {
    /// Output format (text or json)
    #[arg(long, short = 'o', global = true, value_enum, default_value = "text")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Copy, PartialEq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available skills
    List {
        /// Filter by category (languages, paradigms, domains, organizations)
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Search skills by name or description
    Search {
        /// Search query
        query: String,
    },
    /// Show detailed information about a skill
    Info {
        /// Skill name or ID
        name: String,
    },
    /// Install a skill to your Claude skills directory
    Install {
        /// Skill name or ID
        name: String,
        /// Target directory (overrides automatic detection)
        #[arg(short, long)]
        target: Option<PathBuf>,
        /// Install to global ~/.claude/skills instead of project-local
        #[arg(short, long)]
        global: bool,
    },
    /// Show where skills would be installed
    Where,
    /// Update the local skill index
    Update,
    /// Initialize skill directory in current project
    Init,
    /// Remove an installed skill
    Uninstall {
        /// Skill name
        name: String,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Check your setup for common issues
    Doctor,
    /// Generate shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Assemble the perfect team of skills for a project
    #[command(subcommand)]
    Team(TeamCommand),
}

#[derive(Subcommand)]
enum TeamCommand {
    /// Use AI to assemble the ideal skill team for a project description
    Assemble {
        /// Project description (what you're building)
        description: String,
        /// Automatically install the recommended skills
        #[arg(long)]
        install: bool,
        /// Install skills globally
        #[arg(short, long)]
        global: bool,
    },
    /// Analyze the current project and recommend skills
    Analyze {
        /// Path to analyze (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Automatically install the recommended skills
        #[arg(long)]
        install: bool,
        /// Install skills globally
        #[arg(short, long)]
        global: bool,
    },
    /// Save currently installed skills as a reusable team
    Save {
        /// Team name
        name: String,
        /// Description of the team
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List saved teams
    List,
    /// Install all skills from a saved team
    Install {
        /// Team name
        name: String,
        /// Install skills globally
        #[arg(short, long)]
        global: bool,
    },
    /// Show details of a saved team
    Show {
        /// Team name
        name: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Manifest {
    version: String,
    generated_at: String,
    repository: String,
    raw_base_url: String,
    skill_count: usize,
    skills: Vec<Skill>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Skill {
    id: String,
    name: String,
    description: String,
    category: String,
    subcategory: Option<String>,
    path: String,
    files: Vec<String>,
    tags: Vec<String>,
}

fn get_cache_dir() -> Result<PathBuf> {
    let cache = dirs::cache_dir()
        .context("Could not find cache directory")?
        .join("sk1llz");
    fs::create_dir_all(&cache)?;
    Ok(cache)
}

fn get_manifest_path() -> Result<PathBuf> {
    Ok(get_cache_dir()?.join("skills.json"))
}

/// Resolve the skills directory based on context:
/// 1. If in a project with .claude/skills/, use that (project-local)
/// 2. Otherwise, use ~/.claude/skills/ (global)
fn resolve_skills_dir(force_global: bool) -> Result<PathBuf> {
    if !force_global {
        // Check for project-local .claude/skills/ in current directory
        if let Ok(cwd) = std::env::current_dir() {
            let local_skills = cwd.join(".claude").join("skills");
            // Use local if .claude directory exists (even if skills/ doesn't yet)
            let local_claude = cwd.join(".claude");
            if local_claude.exists() && local_claude.is_dir() {
                return Ok(local_skills);
            }
        }
    }

    // Fall back to global
    let global = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".claude")
        .join("skills");
    Ok(global)
}

/// Get both local and global skill directories for display
fn get_skill_locations() -> (Option<PathBuf>, PathBuf) {
    let global = dirs::home_dir()
        .map(|h| h.join(".claude").join("skills"))
        .unwrap_or_else(|| PathBuf::from("~/.claude/skills"));

    let local = std::env::current_dir().ok().and_then(|cwd| {
        let local_claude = cwd.join(".claude");
        if local_claude.exists() && local_claude.is_dir() {
            Some(cwd.join(".claude").join("skills"))
        } else {
            None
        }
    });

    (local, global)
}

fn fetch_manifest() -> Result<Manifest> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching skill manifest...");

    let response = reqwest::blocking::get(get_manifest_url())
        .context("Failed to fetch manifest")?
        .json::<Manifest>()
        .context("Failed to parse manifest")?;

    // Cache it
    let cache_path = get_manifest_path()?;
    let json = serde_json::to_string_pretty(&response)?;
    fs::write(&cache_path, json)?;

    pb.finish_with_message(format!("Loaded {} skills", response.skill_count));
    Ok(response)
}

fn load_manifest() -> Result<Manifest> {
    let cache_path = get_manifest_path()?;

    if cache_path.exists() {
        let content = fs::read_to_string(&cache_path)?;
        serde_json::from_str(&content).context("Failed to parse cached manifest")
    } else {
        fetch_manifest()
    }
}

fn cmd_list(category: Option<String>, format: OutputFormat) -> Result<()> {
    let manifest = load_manifest()?;

    let skills: Vec<_> = match &category {
        Some(cat) => manifest
            .skills
            .iter()
            .filter(|s| s.category.to_lowercase() == cat.to_lowercase())
            .collect(),
        None => manifest.skills.iter().collect(),
    };

    if format == OutputFormat::Json {
        let output = serde_json::json!({
            "count": skills.len(),
            "skills": skills,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if skills.is_empty() {
        println!("{}", "No skills found.".yellow());
        return Ok(());
    }

    // Group by category
    let mut by_category: std::collections::BTreeMap<String, Vec<&Skill>> =
        std::collections::BTreeMap::new();
    for skill in &skills {
        by_category
            .entry(skill.category.clone())
            .or_default()
            .push(skill);
    }

    for (cat, cat_skills) in by_category {
        println!("\n{}", cat.to_uppercase().bold().cyan());
        println!("{}", "─".repeat(40).dimmed());

        for skill in cat_skills {
            let subcat = skill
                .subcategory
                .as_ref()
                .map(|s| format!("[{}]", s))
                .unwrap_or_default();

            println!(
                "  {} {} {}",
                skill.name.bold().green(),
                subcat.dimmed(),
                truncate(&skill.description, 50).dimmed()
            );
        }
    }

    println!(
        "\n{} skills available. Use {} for details.",
        skills.len().to_string().bold(),
        "sk1llz info <name>".cyan()
    );

    Ok(())
}

fn cmd_search(query: &str, format: OutputFormat) -> Result<()> {
    let manifest = load_manifest()?;
    let matcher = SkimMatcherV2::default();

    let mut results: Vec<(&Skill, i64)> = manifest
        .skills
        .iter()
        .filter_map(|skill| {
            let name_score = matcher.fuzzy_match(&skill.name, query).unwrap_or(0);
            let desc_score = matcher.fuzzy_match(&skill.description, query).unwrap_or(0);
            let id_score = matcher.fuzzy_match(&skill.id, query).unwrap_or(0);
            let tag_score: i64 = skill
                .tags
                .iter()
                .filter_map(|t| matcher.fuzzy_match(t, query))
                .max()
                .unwrap_or(0);

            let total = name_score * 3 + id_score * 2 + desc_score + tag_score;
            if total > 0 {
                Some((skill, total))
            } else {
                None
            }
        })
        .collect();

    results.sort_by(|a, b| b.1.cmp(&a.1));

    if format == OutputFormat::Json {
        let skills: Vec<_> = results.iter().map(|(s, _)| *s).collect();
        let output = serde_json::json!({
            "query": query,
            "count": skills.len(),
            "skills": skills,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if results.is_empty() {
        println!("{}", format!("No skills matching '{}'", query).yellow());
        return Ok(());
    }

    println!(
        "\n{} results for '{}':\n",
        results.len().to_string().bold(),
        query.cyan()
    );

    for (skill, _score) in results.iter().take(15) {
        let subcat = skill
            .subcategory
            .as_ref()
            .map(|s| format!("[{}]", s))
            .unwrap_or_default();

        println!(
            "  {} {} {}",
            skill.name.bold().green(),
            subcat.dimmed(),
            format!("({})", skill.category).dimmed()
        );
        println!("    {}", truncate(&skill.description, 70).dimmed());
    }

    if results.len() > 15 {
        println!(
            "\n  {} more results not shown.",
            (results.len() - 15).to_string().yellow()
        );
    }

    Ok(())
}

fn cmd_info(name: &str, format: OutputFormat) -> Result<()> {
    let manifest = load_manifest()?;

    let skill = manifest.skills.iter().find(|s| {
        s.name.to_lowercase() == name.to_lowercase() || s.id.to_lowercase() == name.to_lowercase()
    });

    let skill = match skill {
        Some(s) => s,
        None => {
            let suggestions = find_similar_skills(name, &manifest.skills);
            println!(
                "{} Skill '{}' not found.\n",
                "Error:".red().bold(),
                name.yellow()
            );
            if !suggestions.is_empty() {
                println!("{}", "Did you mean one of these?".cyan());
                for suggestion in &suggestions {
                    println!("  • {}", suggestion.green());
                }
                println!();
            }
            println!(
                "{} Use '{}' to see all available skills.",
                "Hint:".blue().bold(),
                "sk1llz list".cyan()
            );
            return Ok(());
        }
    };

    if format == OutputFormat::Json {
        println!("{}", serde_json::to_string_pretty(&skill)?);
        return Ok(());
    }

    println!("\n{}", skill.name.bold().cyan().underline());
    println!("{}: {}", "ID".bold(), skill.id);
    println!("{}: {}", "Category".bold(), skill.category);
    if let Some(sub) = &skill.subcategory {
        println!("{}: {}", "Subcategory".bold(), sub);
    }
    println!("\n{}", "Description".bold());
    println!("  {}", skill.description);

    println!("\n{}", "Files".bold());
    for file in &skill.files {
        println!("  • {}", file.green());
    }

    println!("\n{}", "Tags".bold());
    println!("  {}", skill.tags.join(", ").dimmed());

    println!("\n{}", "Install".bold());
    println!("  {}", format!("sk1llz install {}", skill.name).cyan());

    println!("\n{}", "View Online".bold());
    println!(
        "  {}",
        format!("{}/{}", manifest.repository, skill.path).blue()
    );

    Ok(())
}

fn cmd_install(name: &str, target: Option<PathBuf>, global: bool) -> Result<()> {
    let manifest = load_manifest()?;

    let skill = manifest
        .skills
        .iter()
        .find(|s| {
            s.name.to_lowercase() == name.to_lowercase()
                || s.id.to_lowercase() == name.to_lowercase()
        })
        .context(format!("Skill '{}' not found", name))?;

    // Determine location type before consuming target
    let location_type = if target.is_some() {
        "custom"
    } else if global {
        "global"
    } else {
        let (local, _) = get_skill_locations();
        if local.is_some() {
            "project-local"
        } else {
            "global"
        }
    };

    let target_dir = match target {
        Some(t) => t,
        None => resolve_skills_dir(global)?.join(&skill.name),
    };

    fs::create_dir_all(&target_dir)?;

    let pb = ProgressBar::new(skill.files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    for file in &skill.files {
        pb.set_message(file.clone());

        let url = format!("{}/{}/{}", get_raw_base_url(), skill.path, file);
        let content = reqwest::blocking::get(&url)
            .context(format!("Failed to fetch {}", file))?
            .text()?;

        let file_path = target_dir.join(file);
        fs::write(&file_path, content)?;

        pb.inc(1);
    }

    pb.finish_with_message("Done!");

    println!(
        "\n{} Installed {} to {} ({})",
        "✓".green().bold(),
        skill.name.cyan(),
        target_dir.display().to_string().green(),
        location_type.dimmed()
    );

    Ok(())
}

fn cmd_where() -> Result<()> {
    let (local, global) = get_skill_locations();

    println!("\n{}", "Skill Installation Locations".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());

    if let Some(local_path) = &local {
        let exists = local_path.exists();
        let status = if exists {
            format!("({} skills)", count_skills(local_path))
        } else {
            "(will be created)".to_string()
        };
        println!(
            "  {} {} {} {}",
            "→".green().bold(),
            "Project-local:".bold(),
            local_path.display().to_string().green(),
            status.dimmed()
        );
        println!("    {}", "(active - .claude/ detected)".cyan());
    } else {
        println!(
            "  {} {}",
            "○".dimmed(),
            "Project-local: not available (no .claude/ in current directory)".dimmed()
        );
    }

    let global_exists = global.exists();
    let global_status = if global_exists {
        format!("({} skills)", count_skills(&global))
    } else {
        "(will be created)".to_string()
    };

    let active = if local.is_none() { " (active)" } else { "" };
    println!(
        "  {} {} {} {}{}",
        if local.is_none() {
            "→".green().bold()
        } else {
            "○".dimmed()
        },
        "Global:".bold(),
        global.display().to_string().green(),
        global_status.dimmed(),
        active.cyan()
    );

    println!();
    println!("{}", "Usage".bold());
    println!(
        "  {} install to project-local (if .claude/ exists)",
        "sk1llz install <skill>".cyan()
    );
    println!(
        "  {} force global installation",
        "sk1llz install <skill> --global".cyan()
    );
    println!(
        "  {} initialize project-local skills",
        "mkdir -p .claude/skills".cyan()
    );

    Ok(())
}

fn count_skills(dir: &PathBuf) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .count()
        })
        .unwrap_or(0)
}

fn cmd_update() -> Result<()> {
    fetch_manifest()?;
    println!("{} Skill index updated.", "✓".green().bold());
    Ok(())
}

fn cmd_init() -> Result<()> {
    let cwd = std::env::current_dir().context("Could not get current directory")?;
    let claude_dir = cwd.join(".claude");
    let skills_dir = claude_dir.join("skills");

    if skills_dir.exists() {
        println!(
            "{} Project already initialized at {}",
            "✓".green().bold(),
            skills_dir.display().to_string().cyan()
        );
        return Ok(());
    }

    fs::create_dir_all(&skills_dir)?;
    fs::write(skills_dir.join(".gitkeep"), "")?;

    println!(
        "{} Initialized sk1llz in {}\n",
        "✓".green().bold(),
        skills_dir.display().to_string().cyan()
    );

    println!("{}", "Next steps:".bold());
    println!("  1. Install some skills:");
    println!("     {}", "sk1llz install torvalds".cyan());
    println!("  2. View installed skills:");
    println!("     {}", "sk1llz where".cyan());

    Ok(())
}

fn cmd_uninstall(name: &str, yes: bool) -> Result<()> {
    let (local, global) = get_skill_locations();

    let mut found_at: Option<PathBuf> = None;

    if let Some(local_path) = &local {
        let skill_path = local_path.join(name);
        if skill_path.exists() {
            found_at = Some(skill_path);
        }
    }

    if found_at.is_none() {
        let skill_path = global.join(name);
        if skill_path.exists() {
            found_at = Some(skill_path);
        }
    }

    let path = match found_at {
        Some(p) => p,
        None => {
            println!(
                "{} Skill '{}' is not installed.\n",
                "Error:".red().bold(),
                name.yellow()
            );
            println!(
                "{} Use '{}' to see installed skills.",
                "Hint:".blue().bold(),
                "sk1llz where".cyan()
            );
            return Ok(());
        }
    };

    if !yes {
        println!(
            "{} Remove skill '{}' from {}?",
            "Confirm:".yellow().bold(),
            name.cyan(),
            path.display().to_string().dimmed()
        );
        print!("  Type 'yes' to confirm: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "yes" {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    fs::remove_dir_all(&path)?;

    println!(
        "{} Removed {} from {}",
        "✓".green().bold(),
        name.cyan(),
        path.display().to_string().dimmed()
    );

    Ok(())
}

fn cmd_doctor() -> Result<()> {
    println!("\n{}", "sk1llz doctor".bold().cyan());
    println!("{}\n", "Checking your setup...".dimmed());

    let mut issues: Vec<String> = Vec::new();

    // Check 1: Cache directory
    print!("  Checking cache directory... ");
    match get_cache_dir() {
        Ok(path) if path.exists() => {
            println!("{}", "OK".green());
        }
        Ok(_) => {
            println!("{}", "MISSING".yellow());
            issues.push("Cache directory doesn't exist. Fix: Run 'sk1llz update'".to_string());
        }
        Err(e) => {
            println!("{}", "ERROR".red());
            issues.push(format!("Cannot determine cache directory: {}", e));
        }
    }

    // Check 2: Manifest freshness
    print!("  Checking skill index... ");
    match check_manifest_age() {
        Ok(days) if days < 7 => {
            println!("{} ({} days old)", "OK".green(), days);
        }
        Ok(days) => {
            println!("{} ({} days old)", "STALE".yellow(), days);
            issues.push("Skill index is stale. Fix: Run 'sk1llz update'".to_string());
        }
        Err(_) => {
            println!("{}", "MISSING".yellow());
            issues.push("No local skill index. Fix: Run 'sk1llz update'".to_string());
        }
    }

    // Check 3: Installation locations
    print!("  Checking installation directories... ");
    let (local, global) = get_skill_locations();
    if local.is_some() || global.exists() {
        println!("{}", "OK".green());
    } else {
        println!("{}", "NONE".yellow());
        issues.push("No skill directories found. Fix: Run 'sk1llz init'".to_string());
    }

    // Check 4: Network connectivity
    print!("  Checking network... ");
    match reqwest::blocking::get(get_manifest_url()) {
        Ok(r) if r.status().is_success() => {
            println!("{}", "OK".green());
        }
        _ => {
            println!("{}", "FAILED".red());
            issues
                .push("Cannot reach skill repository. Check your internet connection.".to_string());
        }
    }

    // Summary
    println!();
    if issues.is_empty() {
        println!("{} All checks passed!", "✓".green().bold());
    } else {
        println!("{} {} issue(s) found:\n", "⚠".yellow().bold(), issues.len());
        for issue in issues {
            println!("  • {}", issue);
        }
    }

    Ok(())
}

fn check_manifest_age() -> Result<u64> {
    let path = get_manifest_path()?;
    let metadata = fs::metadata(&path)?;
    let modified = metadata.modified()?;
    let age = SystemTime::now().duration_since(modified)?;
    Ok(age.as_secs() / 86400)
}

fn find_similar_skills(query: &str, skills: &[Skill]) -> Vec<String> {
    let matcher = SkimMatcherV2::default();

    let mut scored: Vec<_> = skills
        .iter()
        .filter_map(|s| {
            let score = matcher.fuzzy_match(&s.name, query).unwrap_or(0);
            if score > 20 {
                Some((s.name.clone(), score))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().take(3).map(|(name, _)| name).collect()
}

fn cmd_completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "sk1llz", &mut io::stdout());
    Ok(())
}

// ─── Team Types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TeamSpec {
    name: String,
    description: String,
    skills: Vec<TeamMember>,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TeamMember {
    skill_id: String,
    role: String,
    rationale: String,
}

#[derive(Debug, Deserialize)]
struct LlmTeamResponse {
    team_name: String,
    summary: String,
    members: Vec<LlmTeamMember>,
}

#[derive(Debug, Deserialize)]
struct LlmTeamMember {
    skill_id: String,
    role: String,
    rationale: String,
}

// ─── LLM Integration ──────────────────────────────────────────────────────

fn get_anthropic_key() -> Option<String> {
    std::env::var("ANTHROPIC_API_KEY").ok()
}

fn get_openai_key() -> Option<String> {
    std::env::var("OPENAI_API_KEY").ok()
}

fn build_skill_catalog(manifest: &Manifest) -> String {
    let mut catalog = String::new();
    for skill in &manifest.skills {
        let sub = skill
            .subcategory
            .as_deref()
            .map(|s| format!("/{}", s))
            .unwrap_or_default();
        catalog.push_str(&format!(
            "- id: {} | category: {}{} | {}\n",
            skill.id, skill.category, sub, skill.description
        ));
    }
    catalog
}

fn build_team_prompt(description: &str, catalog: &str) -> String {
    format!(
        r#"You are an elite engineering team architect. Given a project description, select the PERFECT team of coding skills from the catalog below.

PROJECT:
{description}

AVAILABLE SKILLS:
{catalog}

Select 3-8 skills that form the ideal team for this project. For each skill, explain:
1. Their specific role on this project
2. Why they are essential (not just nice-to-have)

Think like you're assembling an Avengers-level engineering team. Every member must earn their spot.

Respond with ONLY valid JSON in this exact format (no markdown, no code fences):
{{
  "team_name": "short epic team name",
  "summary": "one sentence describing this team's combined superpower",
  "members": [
    {{
      "skill_id": "exact-skill-id-from-catalog",
      "role": "their role on this project (e.g. 'Systems Architect', 'Performance Lead')",
      "rationale": "why this skill is essential for this specific project"
    }}
  ]
}}"#
    )
}

fn build_analyze_prompt(analysis: &str, catalog: &str) -> String {
    format!(
        r#"You are an elite engineering team architect. I've analyzed a codebase and found the following characteristics. Recommend the perfect team of coding skills.

PROJECT ANALYSIS:
{analysis}

AVAILABLE SKILLS:
{catalog}

Select 3-8 skills that would most benefit this codebase. Prioritize skills that match:
1. The languages and frameworks detected
2. The architectural patterns observed
3. Gaps where expert guidance would elevate the code quality

Respond with ONLY valid JSON in this exact format (no markdown, no code fences):
{{
  "team_name": "short epic team name",
  "summary": "one sentence describing this team's combined superpower",
  "members": [
    {{
      "skill_id": "exact-skill-id-from-catalog",
      "role": "their role on this project",
      "rationale": "why this skill benefits this specific codebase"
    }}
  ]
}}"#
    )
}

fn call_anthropic(prompt: &str) -> Result<String> {
    let api_key = get_anthropic_key().context(
        "ANTHROPIC_API_KEY not set. Set it with:\n  export ANTHROPIC_API_KEY=sk-ant-...\n\nOr set OPENAI_API_KEY for OpenAI.",
    )?;

    let body = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 2048,
        "messages": [{
            "role": "user",
            "content": prompt
        }]
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .context("Failed to call Anthropic API")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        anyhow::bail!("Anthropic API error ({}): {}", status, text);
    }

    let resp: serde_json::Value = response.json()?;
    let text = resp["content"][0]["text"]
        .as_str()
        .context("Unexpected Anthropic response format")?;

    Ok(text.to_string())
}

fn call_openai(prompt: &str) -> Result<String> {
    let api_key = get_openai_key().context("OPENAI_API_KEY not set")?;

    let body = serde_json::json!({
        "model": "gpt-4o",
        "messages": [{
            "role": "user",
            "content": prompt
        }],
        "max_tokens": 2048
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .context("Failed to call OpenAI API")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        anyhow::bail!("OpenAI API error ({}): {}", status, text);
    }

    let resp: serde_json::Value = response.json()?;
    let text = resp["choices"][0]["message"]["content"]
        .as_str()
        .context("Unexpected OpenAI response format")?;

    Ok(text.to_string())
}

fn call_llm(prompt: &str) -> Result<String> {
    if get_anthropic_key().is_some() {
        call_anthropic(prompt)
    } else if get_openai_key().is_some() {
        call_openai(prompt)
    } else {
        anyhow::bail!(
            "No AI API key found. Set one of:\n  \
             export ANTHROPIC_API_KEY=sk-ant-...\n  \
             export OPENAI_API_KEY=sk-..."
        )
    }
}

fn parse_llm_team(raw: &str) -> Result<LlmTeamResponse> {
    // Strip potential markdown code fences
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str(cleaned).context("Failed to parse AI team recommendation as JSON")
}

// ─── Project Analysis ─────────────────────────────────────────────────────

fn analyze_project(path: &Path) -> Result<String> {
    let mut ext_counts: HashMap<String, usize> = HashMap::new();
    let mut total_files = 0usize;
    let mut config_files: Vec<String> = Vec::new();
    let mut frameworks: Vec<String> = Vec::new();

    let known_configs = [
        ("Cargo.toml", "Rust"),
        ("go.mod", "Go"),
        ("package.json", "Node.js/JavaScript"),
        ("tsconfig.json", "TypeScript"),
        ("pyproject.toml", "Python"),
        ("requirements.txt", "Python"),
        ("Gemfile", "Ruby"),
        ("pom.xml", "Java/Maven"),
        ("build.gradle", "Java/Gradle"),
        ("CMakeLists.txt", "C/C++ CMake"),
        ("Makefile", "Make"),
        ("docker-compose.yml", "Docker"),
        ("Dockerfile", "Docker"),
        (".github/workflows", "GitHub Actions CI"),
        ("terraform", "Terraform/IaC"),
        ("k8s", "Kubernetes"),
        ("helm", "Helm"),
    ];

    // Walk directory (max depth 4 to avoid huge repos)
    fn walk(dir: &Path, depth: u8, ext_counts: &mut HashMap<String, usize>, total: &mut usize) {
        if depth > 4 {
            return;
        }
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip hidden dirs, node_modules, target, vendor
            if name.starts_with('.')
                || name == "node_modules"
                || name == "target"
                || name == "vendor"
                || name == "__pycache__"
            {
                continue;
            }
            if path.is_dir() {
                walk(&path, depth + 1, ext_counts, total);
            } else if path.is_file() {
                *total += 1;
                if let Some(ext) = path.extension() {
                    *ext_counts
                        .entry(ext.to_string_lossy().to_string())
                        .or_insert(0) += 1;
                }
            }
        }
    }

    walk(path, 0, &mut ext_counts, &mut total_files);

    // Detect config files and frameworks
    for (config, framework) in &known_configs {
        if path.join(config).exists() {
            config_files.push(config.to_string());
            if !frameworks.contains(&framework.to_string()) {
                frameworks.push(framework.to_string());
            }
        }
    }

    // Sort extensions by count
    let mut ext_sorted: Vec<_> = ext_counts.into_iter().collect();
    ext_sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let mut analysis = String::new();
    analysis.push_str(&format!("Total files: {}\n", total_files));
    analysis.push_str(&format!("Frameworks detected: {}\n", frameworks.join(", ")));
    analysis.push_str(&format!("Config files: {}\n", config_files.join(", ")));
    analysis.push_str("File extensions (by count):\n");
    for (ext, count) in ext_sorted.iter().take(15) {
        analysis.push_str(&format!("  .{}: {}\n", ext, count));
    }

    Ok(analysis)
}

// ─── Team Commands ────────────────────────────────────────────────────────

fn get_teams_dir() -> Result<PathBuf> {
    let dir = get_cache_dir()?.join("teams");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn cmd_team_assemble(description: &str, auto_install: bool, global: bool) -> Result<()> {
    let manifest = load_manifest()?;
    let catalog = build_skill_catalog(&manifest);
    let prompt = build_team_prompt(description, &catalog);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Consulting the AI oracle...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let raw = call_llm(&prompt)?;
    pb.finish_and_clear();

    let team = parse_llm_team(&raw)?;

    // Validate skill IDs against manifest
    let valid_ids: Vec<String> = manifest.skills.iter().map(|s| s.id.clone()).collect();

    println!("\n{}", "━".repeat(60).cyan());
    println!(
        "  {} {}",
        "⚡".bold(),
        team.team_name.bold().cyan().underline()
    );
    println!("  {}", team.summary.dimmed());
    println!("{}\n", "━".repeat(60).cyan());

    let mut installable: Vec<String> = Vec::new();

    for (i, member) in team.members.iter().enumerate() {
        let exists = valid_ids.contains(&member.skill_id);
        let status = if exists {
            installable.push(member.skill_id.clone());
            "✓".green().bold()
        } else {
            "?".yellow().bold()
        };

        println!(
            "  {} {} {}",
            status,
            format!("{}.", i + 1).dimmed(),
            member.skill_id.bold().green()
        );
        println!("    {} {}", "Role:".bold(), member.role);
        println!("    {} {}", "Why:".bold(), member.rationale.dimmed());
        println!();
    }

    println!(
        "{} {} skills recommended, {} available to install\n",
        "Summary:".bold(),
        team.members.len().to_string().cyan(),
        installable.len().to_string().green()
    );

    if auto_install && !installable.is_empty() {
        println!("{}", "Installing team...".bold().cyan());
        for skill_id in &installable {
            if let Some(skill) = manifest.skills.iter().find(|s| s.id == *skill_id) {
                match cmd_install(&skill.name, None, global) {
                    Ok(()) => {}
                    Err(e) => eprintln!("  {} {}: {}", "ERR".red().bold(), skill_id, e),
                }
            }
        }
        println!(
            "\n{} Team {} assembled and installed!",
            "✓".green().bold(),
            team.team_name.cyan()
        );
    } else if !installable.is_empty() {
        println!("{} Install this team with:", "Hint:".blue().bold());
        for id in &installable {
            println!("  {}", format!("sk1llz install {}", id).cyan());
        }
        println!("\n  Or re-run with {} to auto-install.", "--install".cyan());
    }

    Ok(())
}

fn cmd_team_analyze(path: &Path, auto_install: bool, global: bool) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Scanning project...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let analysis = analyze_project(path)?;
    pb.set_message("Project scanned. Consulting the AI oracle...");

    let manifest = load_manifest()?;
    let catalog = build_skill_catalog(&manifest);
    let prompt = build_analyze_prompt(&analysis, &catalog);

    let raw = call_llm(&prompt)?;
    pb.finish_and_clear();

    println!("\n{}", "Project Analysis".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    for line in analysis.lines() {
        println!("  {}", line.dimmed());
    }
    println!();

    let team = parse_llm_team(&raw)?;

    let valid_ids: Vec<String> = manifest.skills.iter().map(|s| s.id.clone()).collect();

    println!("{}", "━".repeat(60).cyan());
    println!(
        "  {} {}",
        "⚡".bold(),
        team.team_name.bold().cyan().underline()
    );
    println!("  {}", team.summary.dimmed());
    println!("{}\n", "━".repeat(60).cyan());

    let mut installable: Vec<String> = Vec::new();

    for (i, member) in team.members.iter().enumerate() {
        let exists = valid_ids.contains(&member.skill_id);
        let status = if exists {
            installable.push(member.skill_id.clone());
            "✓".green().bold()
        } else {
            "?".yellow().bold()
        };

        println!(
            "  {} {} {}",
            status,
            format!("{}.", i + 1).dimmed(),
            member.skill_id.bold().green()
        );
        println!("    {} {}", "Role:".bold(), member.role);
        println!("    {} {}", "Why:".bold(), member.rationale.dimmed());
        println!();
    }

    println!(
        "{} {} skills recommended, {} available to install\n",
        "Summary:".bold(),
        team.members.len().to_string().cyan(),
        installable.len().to_string().green()
    );

    if auto_install && !installable.is_empty() {
        println!("{}", "Installing team...".bold().cyan());
        for skill_id in &installable {
            if let Some(skill) = manifest.skills.iter().find(|s| s.id == *skill_id) {
                match cmd_install(&skill.name, None, global) {
                    Ok(()) => {}
                    Err(e) => eprintln!("  {} {}: {}", "ERR".red().bold(), skill_id, e),
                }
            }
        }
        println!(
            "\n{} Team {} assembled and installed!",
            "✓".green().bold(),
            team.team_name.cyan()
        );
    } else if !installable.is_empty() {
        println!("{} Install this team with:", "Hint:".blue().bold());
        for id in &installable {
            println!("  {}", format!("sk1llz install {}", id).cyan());
        }
        println!("\n  Or re-run with {} to auto-install.", "--install".cyan());
    }

    Ok(())
}

fn cmd_team_save(name: &str, description: Option<String>) -> Result<()> {
    let (local, global) = get_skill_locations();
    let mut skills_found: Vec<String> = Vec::new();

    // Collect installed skills from both locations
    for dir in [local.as_ref(), Some(&global)].into_iter().flatten() {
        if dir.exists() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name != ".gitkeep" && !skills_found.contains(&name) {
                            skills_found.push(name);
                        }
                    }
                }
            }
        }
    }

    if skills_found.is_empty() {
        println!(
            "{} No installed skills found. Install some first with {}",
            "Error:".red().bold(),
            "sk1llz install <skill>".cyan()
        );
        return Ok(());
    }

    let team = TeamSpec {
        name: name.to_string(),
        description: description.unwrap_or_else(|| format!("Team: {}", name)),
        skills: skills_found
            .iter()
            .map(|s| TeamMember {
                skill_id: s.clone(),
                role: "team member".to_string(),
                rationale: "installed skill".to_string(),
            })
            .collect(),
        created_at: chrono_free_now(),
    };

    let teams_dir = get_teams_dir()?;
    let file = teams_dir.join(format!("{}.json", name));
    let json = serde_json::to_string_pretty(&team)?;
    fs::write(&file, json)?;

    println!(
        "\n{} Saved team '{}' with {} skills to {}",
        "✓".green().bold(),
        name.cyan(),
        team.skills.len().to_string().green(),
        file.display().to_string().dimmed()
    );

    for member in &team.skills {
        println!("  {} {}", "•".green(), member.skill_id);
    }

    Ok(())
}

fn cmd_team_list() -> Result<()> {
    let teams_dir = get_teams_dir()?;
    let mut teams: Vec<TeamSpec> = Vec::new();

    if let Ok(entries) = fs::read_dir(&teams_dir) {
        for entry in entries.flatten() {
            if entry
                .path()
                .extension()
                .map(|e| e == "json")
                .unwrap_or(false)
            {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(team) = serde_json::from_str::<TeamSpec>(&content) {
                        teams.push(team);
                    }
                }
            }
        }
    }

    if teams.is_empty() {
        println!("{}", "No saved teams found.".yellow());
        println!("\n{} Create one with:", "Hint:".blue().bold());
        println!(
            "  {}",
            "sk1llz team assemble \"project description\"".cyan()
        );
        println!("  {}", "sk1llz team save my-team".cyan());
        return Ok(());
    }

    println!("\n{}", "Saved Teams".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());

    for team in &teams {
        println!(
            "\n  {} {} {}",
            "⚡".bold(),
            team.name.bold().green(),
            format!("({} skills)", team.skills.len()).dimmed()
        );
        println!("    {}", team.description.dimmed());
    }

    println!(
        "\n{} View details with {}",
        "Hint:".blue().bold(),
        "sk1llz team show <name>".cyan()
    );

    Ok(())
}

fn cmd_team_show(name: &str) -> Result<()> {
    let teams_dir = get_teams_dir()?;
    let file = teams_dir.join(format!("{}.json", name));

    if !file.exists() {
        println!(
            "{} Team '{}' not found.",
            "Error:".red().bold(),
            name.yellow()
        );
        println!(
            "{} Use {} to see saved teams.",
            "Hint:".blue().bold(),
            "sk1llz team list".cyan()
        );
        return Ok(());
    }

    let content = fs::read_to_string(&file)?;
    let team: TeamSpec = serde_json::from_str(&content)?;

    println!("\n{}", "━".repeat(60).cyan());
    println!("  {} {}", "⚡".bold(), team.name.bold().cyan().underline());
    println!("  {}", team.description.dimmed());
    println!("  {}", format!("Created: {}", team.created_at).dimmed());
    println!("{}\n", "━".repeat(60).cyan());

    for (i, member) in team.skills.iter().enumerate() {
        println!(
            "  {} {} {}",
            "✓".green().bold(),
            format!("{}.", i + 1).dimmed(),
            member.skill_id.bold().green()
        );
        if member.role != "team member" {
            println!("    {} {}", "Role:".bold(), member.role);
        }
        if member.rationale != "installed skill" {
            println!("    {} {}", "Why:".bold(), member.rationale.dimmed());
        }
    }

    println!(
        "\n{} Install with: {}",
        "Hint:".blue().bold(),
        format!("sk1llz team install {}", name).cyan()
    );

    Ok(())
}

fn cmd_team_install(name: &str, global: bool) -> Result<()> {
    let teams_dir = get_teams_dir()?;
    let file = teams_dir.join(format!("{}.json", name));

    if !file.exists() {
        println!(
            "{} Team '{}' not found.",
            "Error:".red().bold(),
            name.yellow()
        );
        return Ok(());
    }

    let content = fs::read_to_string(&file)?;
    let team: TeamSpec = serde_json::from_str(&content)?;
    let manifest = load_manifest()?;

    println!(
        "\n{} Installing team '{}' ({} skills)...\n",
        "⚡".bold(),
        team.name.cyan(),
        team.skills.len()
    );

    let mut installed = 0;
    let mut skipped = 0;

    for member in &team.skills {
        if let Some(skill) = manifest
            .skills
            .iter()
            .find(|s| s.id == member.skill_id || s.name == member.skill_id)
        {
            match cmd_install(&skill.name, None, global) {
                Ok(()) => installed += 1,
                Err(e) => {
                    eprintln!("  {} {}: {}", "ERR".red().bold(), member.skill_id, e);
                    skipped += 1;
                }
            }
        } else {
            println!(
                "  {} {} not found in catalog, skipping",
                "SKIP".yellow().bold(),
                member.skill_id
            );
            skipped += 1;
        }
    }

    println!(
        "\n{} Team '{}': {} installed, {} skipped",
        "✓".green().bold(),
        team.name.cyan(),
        installed.to_string().green(),
        skipped.to_string().yellow()
    );

    Ok(())
}

fn chrono_free_now() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { category } => cmd_list(category, cli.format),
        Commands::Search { query } => cmd_search(&query, cli.format),
        Commands::Info { name } => cmd_info(&name, cli.format),
        Commands::Install {
            name,
            target,
            global,
        } => cmd_install(&name, target, global),
        Commands::Where => cmd_where(),
        Commands::Update => cmd_update(),
        Commands::Init => cmd_init(),
        Commands::Uninstall { name, yes } => cmd_uninstall(&name, yes),
        Commands::Doctor => cmd_doctor(),
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Team(team_cmd) => match team_cmd {
            TeamCommand::Assemble {
                description,
                install,
                global,
            } => cmd_team_assemble(&description, install, global),
            TeamCommand::Analyze {
                path,
                install,
                global,
            } => cmd_team_analyze(&path, install, global),
            TeamCommand::Save { name, description } => cmd_team_save(&name, description),
            TeamCommand::List => cmd_team_list(),
            TeamCommand::Install { name, global } => cmd_team_install(&name, global),
            TeamCommand::Show { name } => cmd_team_show(&name),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello", 5), "hello");
        // UTF-8 safety: should not panic on multi-byte characters
        assert_eq!(truncate("Fran Méndez designs APIs", 10), "Fran Mé...");
    }

    #[test]
    fn test_find_similar_skills() {
        let skills = vec![
            Skill {
                id: "test-skill-1".to_string(),
                name: "rust-expert".to_string(),
                description: "Expert level Rust".to_string(),
                category: "languages".to_string(),
                subcategory: None,
                path: "path/to/skill".to_string(),
                files: vec![],
                tags: vec!["rust".to_string()],
            },
            Skill {
                id: "test-skill-2".to_string(),
                name: "python-expert".to_string(),
                description: "Expert level Python".to_string(),
                category: "languages".to_string(),
                subcategory: None,
                path: "path/to/skill2".to_string(),
                files: vec![],
                tags: vec!["python".to_string()],
            },
        ];

        let results = find_similar_skills("rust", &skills);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "rust-expert");

        let results = find_similar_skills("expert", &skills);
        assert_eq!(results.len(), 2);
    }
}
