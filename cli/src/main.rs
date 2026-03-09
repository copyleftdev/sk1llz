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
    /// Assemble the ideal skill team for a project description
    Assemble {
        /// Project description (what you're building)
        description: String,
        /// Automatically install the recommended skills
        #[arg(long)]
        install: bool,
        /// Install skills globally
        #[arg(short, long)]
        global: bool,
        /// Use AI (LLM) instead of local NLP engine
        #[arg(long)]
        ai: bool,
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
        /// Use AI (LLM) instead of local NLP engine
        #[arg(long)]
        ai: bool,
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

// ─── Local NLP Engine ─────────────────────────────────────────────────────

const STOP_WORDS: &[&str] = &[
    "a",
    "an",
    "the",
    "in",
    "on",
    "at",
    "to",
    "for",
    "of",
    "with",
    "by",
    "from",
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "being",
    "have",
    "has",
    "had",
    "do",
    "does",
    "did",
    "will",
    "would",
    "could",
    "should",
    "may",
    "might",
    "shall",
    "can",
    "need",
    "must",
    "and",
    "or",
    "but",
    "if",
    "then",
    "else",
    "when",
    "up",
    "out",
    "so",
    "no",
    "not",
    "only",
    "very",
    "just",
    "that",
    "this",
    "it",
    "its",
    "i",
    "we",
    "you",
    "they",
    "he",
    "she",
    "my",
    "your",
    "our",
    "their",
    "what",
    "which",
    "who",
    "whom",
    "how",
    "all",
    "each",
    "every",
    "both",
    "few",
    "more",
    "most",
    "some",
    "any",
    "such",
    "than",
    "too",
    "also",
    "into",
    "over",
    "after",
    "before",
    "between",
    "under",
    "about",
    "using",
    "like",
    "want",
    "building",
    "build",
    "create",
    "creating",
    "make",
    "making",
    "write",
    "writing",
    "develop",
    "developing",
    "use",
];

/// Lightweight Porter-style suffix stripping
fn stem(word: &str) -> String {
    let w = word.to_lowercase();
    // Order matters: longest suffixes first
    for suffix in &[
        "ization", "isation", "fulness", "ousness", "iveness", "ational", "lessly", "ioning",
        "encing", "ancing", "enting", "ating", "ening", "ivity", "ement", "ments", "ness", "able",
        "ible", "ment", "tion", "sion", "ling", "ally", "ously", "ively", "ence", "ance", "ings",
        "ful", "ous", "ive", "ing", "ers", "ity", "ent", "ant", "ism", "ist", "ess", "ary", "ory",
        "ial", "ual", "ion", "ler", "ted", "ied", "les", "ies", "ize", "ise", "ate", "fy", "ly",
        "ed", "er", "es", "al",
    ] {
        if w.len() > suffix.len() + 2 && w.ends_with(suffix) {
            return w[..w.len() - suffix.len()].to_string();
        }
    }
    // Trailing 's' (but not 'ss')
    if w.len() > 3 && w.ends_with('s') && !w.ends_with("ss") {
        return w[..w.len() - 1].to_string();
    }
    w
}

/// Synonym / concept expansion table
/// Maps a term to related concepts that should also be searched
fn expand_synonyms(token: &str) -> Vec<String> {
    let expansions: &[(&str, &[&str])] = &[
        (
            "grep",
            &[
                "search", "regex", "pattern", "text", "matching", "filter", "find",
            ],
        ),
        (
            "ripgrep",
            &[
                "search",
                "regex",
                "pattern",
                "performance",
                "parallel",
                "rust",
                "cli",
                "fast",
            ],
        ),
        (
            "regex",
            &[
                "pattern",
                "matching",
                "automaton",
                "nfa",
                "dfa",
                "text",
                "search",
            ],
        ),
        (
            "search",
            &["find", "query", "index", "lookup", "filter", "match"],
        ),
        (
            "cli",
            &["command", "terminal", "shell", "interface", "tool"],
        ),
        ("terminal", &["cli", "shell", "console", "tui", "output"]),
        (
            "filesystem",
            &[
                "file",
                "directory",
                "path",
                "traverse",
                "walk",
                "io",
                "systems",
            ],
        ),
        (
            "traverse",
            &["walk", "scan", "iterate", "directory", "recursive"],
        ),
        (
            "parallel",
            &["concurrent", "thread", "async", "performance", "fast"],
        ),
        (
            "concurrent",
            &["parallel", "thread", "async", "lock", "atomic"],
        ),
        (
            "fast",
            &[
                "performance",
                "speed",
                "optimization",
                "latency",
                "efficient",
            ],
        ),
        (
            "performance",
            &[
                "fast",
                "speed",
                "optimization",
                "latency",
                "efficient",
                "benchmark",
            ],
        ),
        ("token", &["lexer", "parser", "text", "pattern", "string"]),
        ("parser", &["lexer", "token", "ast", "grammar", "syntax"]),
        (
            "rust",
            &["systems", "memory", "safety", "ownership", "performance"],
        ),
        (
            "database",
            &["sql", "query", "storage", "index", "transaction", "data"],
        ),
        (
            "distributed",
            &[
                "consensus",
                "replication",
                "network",
                "fault",
                "tolerance",
                "cluster",
            ],
        ),
        (
            "web",
            &["http", "api", "server", "client", "request", "response"],
        ),
        (
            "api",
            &["rest", "http", "endpoint", "interface", "service", "design"],
        ),
        (
            "security",
            &["auth", "crypto", "encryption", "vulnerability", "threat"],
        ),
        (
            "test",
            &[
                "testing",
                "quality",
                "assertion",
                "coverage",
                "verification",
            ],
        ),
        (
            "trading",
            &["finance", "market", "exchange", "latency", "quantitative"],
        ),
        (
            "network",
            &["socket", "protocol", "tcp", "udp", "latency", "packet"],
        ),
        (
            "compiler",
            &[
                "parser",
                "lexer",
                "ast",
                "codegen",
                "optimization",
                "language",
            ],
        ),
        (
            "cloud",
            &[
                "infrastructure",
                "deploy",
                "container",
                "kubernetes",
                "aws",
                "scale",
            ],
        ),
        (
            "machine",
            &["learning", "model", "training", "inference", "neural"],
        ),
        (
            "data",
            &["pipeline", "processing", "storage", "analysis", "stream"],
        ),
        (
            "microservice",
            &["distributed", "api", "service", "container", "event"],
        ),
        (
            "embedded",
            &["hardware", "firmware", "iot", "realtime", "bare", "metal"],
        ),
        (
            "operating",
            &["kernel", "systems", "process", "memory", "scheduler"],
        ),
        (
            "kernel",
            &["operating", "systems", "driver", "scheduler", "memory"],
        ),
        (
            "utf",
            &[
                "unicode",
                "encoding",
                "text",
                "string",
                "internationalization",
            ],
        ),
        (
            "unicode",
            &["utf", "encoding", "text", "string", "character"],
        ),
        (
            "index",
            &["search", "lookup", "query", "structure", "tree", "hash"],
        ),
        // Compound terms (after normalize_compounds)
        (
            "realtime",
            &[
                "streaming",
                "low_latency",
                "websocket",
                "event_driven",
                "sync",
                "interactive",
            ],
        ),
        (
            "websocket",
            &[
                "realtime",
                "streaming",
                "sync",
                "event_driven",
                "networking",
                "collaborative",
            ],
        ),
        (
            "crdt",
            &[
                "replication",
                "consistency",
                "collaborative",
                "distributed",
                "conflict",
                "sync",
                "convergent",
            ],
        ),
        (
            "collaborative",
            &[
                "crdt",
                "sync",
                "multiuser",
                "realtime",
                "sharing",
                "distributed",
            ],
        ),
        (
            "canvas",
            &[
                "rendering",
                "graphics",
                "drawing",
                "ui",
                "gpu",
                "2d",
                "vector",
            ],
        ),
        (
            "whiteboard",
            &[
                "canvas",
                "drawing",
                "collaborative",
                "ui",
                "interactive",
                "realtime",
            ],
        ),
        (
            "lowlatency",
            &[
                "performance",
                "fast",
                "networking",
                "optimization",
                "realtime",
            ],
        ),
        (
            "highperformance",
            &["performance", "fast", "optimization", "scale", "efficient"],
        ),
        (
            "eventdriven",
            &[
                "async",
                "streaming",
                "pubsub",
                "message",
                "reactive",
                "websocket",
            ],
        ),
        (
            "machinelearning",
            &[
                "neural",
                "training",
                "model",
                "inference",
                "data",
                "ai",
                "deep",
            ],
        ),
        (
            "typescript",
            &["javascript", "types", "web", "frontend", "node"],
        ),
        (
            "react",
            &[
                "components",
                "ui",
                "frontend",
                "hooks",
                "virtual_dom",
                "state",
                "declarative",
            ],
        ),
        (
            "docker",
            &[
                "container",
                "kubernetes",
                "deploy",
                "infrastructure",
                "devops",
            ],
        ),
        (
            "kubernetes",
            &[
                "container",
                "orchestration",
                "deploy",
                "cloud",
                "infrastructure",
                "scaling",
            ],
        ),
    ];

    let stemmed = stem(token);
    let mut results = Vec::new();
    for &(key, synonyms) in expansions {
        if token == key || stemmed == stem(key) {
            for &syn in synonyms {
                results.push(syn.to_string());
            }
        }
    }
    results
}

/// Domain concept detector: recognizes project archetypes and injects high-signal tags
fn detect_domain_concepts(tokens: &[String]) -> Vec<String> {
    let mut bonus_tokens = Vec::new();
    let joined = tokens.join(" ");

    // grep/search tool archetype
    if joined.contains("grep")
        || joined.contains("ripgrep")
        || joined.contains("search tool")
        || (joined.contains("search") && joined.contains("file"))
        || (joined.contains("find") && joined.contains("text"))
    {
        for t in &[
            "performance",
            "systems",
            "cli",
            "regex",
            "parallel",
            "text",
            "pattern",
            "low-level",
        ] {
            bonus_tokens.push(t.to_string());
        }
    }

    // web server archetype
    if joined.contains("web server") || joined.contains("http server") || joined.contains("web app")
    {
        for t in &["api", "rest", "http", "middleware", "routing", "security"] {
            bonus_tokens.push(t.to_string());
        }
    }

    // database archetype
    if joined.contains("database")
        || joined.contains("storage engine")
        || joined.contains("query engine")
    {
        for t in &[
            "transaction",
            "acid",
            "btree",
            "index",
            "concurrency",
            "recovery",
            "data",
        ] {
            bonus_tokens.push(t.to_string());
        }
    }

    // distributed system archetype
    if joined.contains("distributed")
        || joined.contains("microservice")
        || joined.contains("cluster")
    {
        for t in &[
            "consensus",
            "replication",
            "fault",
            "network",
            "eventual",
            "consistency",
        ] {
            bonus_tokens.push(t.to_string());
        }
    }

    // compiler/language archetype
    if joined.contains("compiler") || joined.contains("language") || joined.contains("interpreter")
    {
        for t in &[
            "parser",
            "lexer",
            "ast",
            "codegen",
            "type",
            "grammar",
            "optimization",
        ] {
            bonus_tokens.push(t.to_string());
        }
    }

    // OS/kernel archetype
    if joined.contains("operating system") || joined.contains("kernel") || joined.contains("os ") {
        for t in &[
            "systems",
            "memory",
            "scheduler",
            "driver",
            "process",
            "interrupt",
        ] {
            bonus_tokens.push(t.to_string());
        }
    }

    bonus_tokens
}

/// Normalize compound terms before tokenization
fn normalize_compounds(text: &str) -> String {
    let compounds: &[(&str, &str)] = &[
        ("real-time", "realtime"),
        ("real time", "realtime"),
        ("low-latency", "lowlatency"),
        ("low latency", "lowlatency"),
        ("high-performance", "highperformance"),
        ("high performance", "highperformance"),
        ("web socket", "websocket"),
        ("web-socket", "websocket"),
        ("type script", "typescript"),
        ("type-script", "typescript"),
        ("java script", "javascript"),
        ("java-script", "javascript"),
        ("event-driven", "eventdriven"),
        ("event driven", "eventdriven"),
        ("message-passing", "messagepassing"),
        ("message passing", "messagepassing"),
        ("lock-free", "lockfree"),
        ("lock free", "lockfree"),
        ("zero-copy", "zerocopy"),
        ("zero copy", "zerocopy"),
        ("multi-user", "multiuser"),
        ("multi user", "multiuser"),
        ("open-source", "opensource"),
        ("open source", "opensource"),
        ("machine-learning", "machinelearning"),
        ("machine learning", "machinelearning"),
        ("deep-learning", "deeplearning"),
        ("deep learning", "deeplearning"),
    ];

    let mut result = text.to_lowercase();
    for &(pattern, replacement) in compounds {
        result = result.replace(pattern, replacement);
    }
    result
}

/// Core tokenizer: split, lowercase, filter stop words, add stems
fn tokenize(text: &str) -> Vec<String> {
    let normalized = normalize_compounds(text);
    let raw: Vec<String> = normalized
        .split(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
        .filter(|w| w.len() > 1)
        .filter(|w| !STOP_WORDS.contains(w))
        .map(String::from)
        .collect();

    let mut tokens = Vec::new();
    for t in &raw {
        tokens.push(t.clone());
        let s = stem(t);
        if s != *t {
            tokens.push(s);
        }
    }
    tokens
}

/// Expanded tokens: synonyms + domain concepts (lower-signal, used for secondary scoring)
fn tokenize_expanded(text: &str) -> Vec<String> {
    let normalized = normalize_compounds(text);
    let raw: Vec<String> = normalized
        .split(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
        .filter(|w| w.len() > 1)
        .filter(|w| !STOP_WORDS.contains(w))
        .map(String::from)
        .collect();

    let mut expanded = Vec::new();
    for t in &raw {
        for syn in expand_synonyms(t) {
            expanded.push(syn.clone());
            let s = stem(&syn);
            if s != syn {
                expanded.push(s);
            }
        }
    }
    let domain = detect_domain_concepts(&raw);
    for t in &domain {
        expanded.push(t.clone());
        let s = stem(t);
        if s != *t {
            expanded.push(s);
        }
    }
    expanded
}

/// BM25 parameters
const BM25_K1: f64 = 1.5;
const BM25_B: f64 = 0.75;

struct SkillDocument {
    skill_idx: usize,
    tokens: Vec<String>,
    len: usize,
}

fn build_skill_documents(manifest: &Manifest) -> Vec<SkillDocument> {
    manifest
        .skills
        .iter()
        .enumerate()
        .map(|(idx, skill)| {
            let mut text = String::new();
            // Weight: id tokens appear 3x, tags 3x, description 1x, category 2x
            for _ in 0..3 {
                text.push(' ');
                text.push_str(&skill.id.replace('-', " "));
            }
            for _ in 0..3 {
                for tag in &skill.tags {
                    text.push(' ');
                    text.push_str(tag);
                }
            }
            for _ in 0..2 {
                text.push(' ');
                text.push_str(&skill.category);
                if let Some(sub) = &skill.subcategory {
                    text.push(' ');
                    text.push_str(sub);
                }
            }
            text.push(' ');
            text.push_str(&skill.description);
            // Also add the skill name with high weight
            for _ in 0..3 {
                text.push(' ');
                text.push_str(&skill.name);
            }

            let tokens = tokenize(&text);
            let len = tokens.len();
            SkillDocument {
                skill_idx: idx,
                tokens,
                len,
            }
        })
        .collect()
}

fn bm25_score(query_tokens: &[String], docs: &[SkillDocument]) -> Vec<(usize, f64)> {
    let n = docs.len() as f64;
    let avg_dl: f64 = docs.iter().map(|d| d.len as f64).sum::<f64>() / n;

    // Compute document frequency for each query term
    let mut df: HashMap<String, usize> = HashMap::new();
    for token in query_tokens {
        if df.contains_key(token) {
            continue;
        }
        let count = docs.iter().filter(|d| d.tokens.contains(token)).count();
        df.insert(token.clone(), count);
    }

    let mut scores: Vec<(usize, f64)> = docs
        .iter()
        .map(|doc| {
            let dl = doc.len as f64;
            let mut score = 0.0;

            for token in query_tokens {
                let tf = doc.tokens.iter().filter(|t| *t == token).count() as f64;
                let doc_freq = *df.get(token).unwrap_or(&0) as f64;

                if doc_freq == 0.0 || tf == 0.0 {
                    continue;
                }

                // IDF component (BM25 variant)
                let idf = ((n - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln();

                // TF component with length normalization
                let tf_norm = (tf * (BM25_K1 + 1.0))
                    / (tf + BM25_K1 * (1.0 - BM25_B + BM25_B * (dl / avg_dl)));

                score += idf * tf_norm;
            }

            (doc.skill_idx, score)
        })
        .filter(|(_, s)| *s > 0.0)
        .collect();

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores
}

/// Jaccard similarity between two tag sets
fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let set_a: std::collections::HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let set_b: std::collections::HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let intersection = set_a.intersection(&set_b).count() as f64;
    let union = set_a.union(&set_b).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

/// Detect which programming language(s) the query is about
fn detect_query_languages(query: &str) -> Vec<String> {
    let q = query.to_lowercase();
    let lang_signals: &[(&str, &[&str])] = &[
        ("rust", &["rust", "cargo", "crate", "ownership", "borrow"]),
        ("go", &["golang", " go ", "goroutine", "gopher"]),
        ("python", &["python", "pip", "django", "flask", "pytorch"]),
        (
            "javascript",
            &[
                "javascript",
                "node",
                "npm",
                "react",
                "vue",
                "angular",
                "deno",
                "bun",
            ],
        ),
        ("typescript", &["typescript"]),
        ("c", &[" c ", "c language", "c programming"]),
        ("cpp", &["c++", "cpp"]),
        ("java", &["java ", "jvm", "spring", "maven", "gradle"]),
        ("ruby", &["ruby", "rails", "gem"]),
        ("zig", &["zig "]),
    ];

    let mut detected = Vec::new();
    for &(lang, signals) in lang_signals {
        // Pad query for word-boundary-like matching
        let padded = format!(" {} ", q);
        if signals.iter().any(|s| padded.contains(s)) {
            detected.push(lang.to_string());
        }
    }
    detected
}

/// Apply language affinity: boost skills matching query language, penalize mismatched language skills
fn apply_language_affinity(
    scores: &mut [(usize, f64)],
    manifest: &Manifest,
    query_languages: &[String],
) {
    if query_languages.is_empty() {
        return;
    }

    for (idx, score) in scores.iter_mut() {
        let skill = &manifest.skills[*idx];

        // Check if this skill is in a language-specific category
        let skill_lang = if skill.category == "languages" {
            skill.subcategory.clone()
        } else {
            // Check tags for language signals
            let lang_tags = [
                "rust",
                "go",
                "python",
                "javascript",
                "typescript",
                "c",
                "cpp",
                "java",
                "ruby",
                "zig",
            ];
            skill
                .tags
                .iter()
                .find(|t| lang_tags.contains(&t.as_str()))
                .cloned()
        };

        if let Some(ref sl) = skill_lang {
            if query_languages.iter().any(|ql| ql == sl) {
                // Matching language: mild 20% boost (relevance still dominates)
                *score *= 1.2;
            } else if skill.category == "languages" {
                // Wrong language domain: heavy penalty
                *score *= 0.15;
            }
        }
    }

    // Re-sort after affinity adjustment
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
}

/// Select team with relevance-first strategy, diversity as tiebreaker
fn select_diverse_team(
    scores: &[(usize, f64)],
    manifest: &Manifest,
    max_size: usize,
    query: &str,
) -> Vec<(usize, f64)> {
    if scores.is_empty() {
        return vec![];
    }

    // Apply language affinity scoring
    let query_languages = detect_query_languages(query);
    let mut adjusted_scores = scores.to_vec();
    apply_language_affinity(&mut adjusted_scores, manifest, &query_languages);

    let top_score = adjusted_scores[0].1;
    // Adaptive relevance floor: start at 30%, relax down to 10% if too few candidates
    let min_team = max_size.min(4);
    let mut floor = top_score * 0.30;
    let mut candidates: Vec<(usize, f64)> = adjusted_scores
        .iter()
        .filter(|(_, s)| *s >= floor)
        .copied()
        .collect();
    if candidates.len() < min_team {
        floor = top_score * 0.15;
        candidates = adjusted_scores
            .iter()
            .filter(|(_, s)| *s >= floor)
            .copied()
            .collect();
    }
    if candidates.len() < min_team {
        floor = top_score * 0.10;
        candidates = adjusted_scores
            .iter()
            .filter(|(_, s)| *s >= floor)
            .copied()
            .collect();
    }

    let mut selected: Vec<(usize, f64)> = Vec::new();
    let mut categories_covered: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // Phase 1: Fill first with top scorers (relevance-first)
    // Take top ceil(max_size * 0.6) purely by score
    let relevance_slots = (max_size as f64 * 0.6).ceil() as usize;
    for &(idx, score) in &candidates {
        if selected.len() >= relevance_slots {
            break;
        }
        let skill = &manifest.skills[idx];
        let cat_key = if let Some(sub) = &skill.subcategory {
            format!("{}/{}", skill.category, sub)
        } else {
            skill.category.clone()
        };
        categories_covered.insert(cat_key);
        selected.push((idx, score));
    }

    // Phase 2: Fill remaining slots preferring uncovered categories (diversity as tiebreaker)
    for &(idx, score) in &candidates {
        if selected.len() >= max_size {
            break;
        }
        if selected.iter().any(|(i, _)| *i == idx) {
            continue;
        }
        let skill = &manifest.skills[idx];
        let cat_key = if let Some(sub) = &skill.subcategory {
            format!("{}/{}", skill.category, sub)
        } else {
            skill.category.clone()
        };
        if !categories_covered.contains(&cat_key) {
            categories_covered.insert(cat_key);
            selected.push((idx, score));
        }
    }

    // Phase 3: Fill any remaining slots with next-best scorers
    for &(idx, score) in &candidates {
        if selected.len() >= max_size {
            break;
        }
        if !selected.iter().any(|(i, _)| *i == idx) {
            selected.push((idx, score));
        }
    }

    // Phase 4: Affinity swap — check if a highly complementary skill should replace the weakest
    let mut affinity_candidates: Vec<(usize, f64)> = Vec::new();
    for &(sel_idx, _) in &selected {
        let sel_tags = &manifest.skills[sel_idx].tags;
        for &(cand_idx, cand_score) in &candidates {
            if selected.iter().any(|(i, _)| *i == cand_idx) {
                continue;
            }
            if affinity_candidates.iter().any(|(i, _)| *i == cand_idx) {
                continue;
            }
            let sim = jaccard_similarity(sel_tags, &manifest.skills[cand_idx].tags);
            if sim > 0.3 {
                affinity_candidates.push((cand_idx, cand_score * (1.0 + sim)));
            }
        }
    }

    affinity_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    if let Some(&(aff_idx, aff_score)) = affinity_candidates.first() {
        if let Some(min_pos) = selected
            .iter()
            .enumerate()
            .min_by(|a, b| {
                a.1 .1
                    .partial_cmp(&b.1 .1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(pos, _)| pos)
        {
            if selected.len() >= max_size && aff_score > selected[min_pos].1 {
                selected[min_pos] = (aff_idx, aff_score);
            }
        }
    }

    selected
}

/// Assign a role based on category and subcategory
fn assign_role(skill: &Skill) -> String {
    match (skill.category.as_str(), skill.subcategory.as_deref()) {
        ("languages", Some(lang)) => format!("{} Language Expert", capitalize(lang)),
        ("languages", None) => "Language Expert".to_string(),
        ("paradigms", Some("functional")) => "Functional Design Lead".to_string(),
        ("paradigms", Some("systems")) => "Systems Architecture Lead".to_string(),
        ("paradigms", Some("distributed")) => "Distributed Systems Architect".to_string(),
        ("paradigms", _) => "Paradigm Specialist".to_string(),
        ("domains", Some("testing")) => "Quality & Testing Lead".to_string(),
        ("domains", Some("systems-architecture")) => "Systems Architect".to_string(),
        ("domains", Some("databases")) => "Data & Storage Lead".to_string(),
        ("domains", Some("networking")) => "Networking Specialist".to_string(),
        ("domains", Some("security")) => "Security Lead".to_string(),
        ("domains", Some("api-design")) => "API Design Lead".to_string(),
        ("domains", Some("cli-design")) => "CLI/UX Lead".to_string(),
        ("domains", Some("problem-solving")) => "Problem-Solving Strategist".to_string(),
        ("domains", Some("trading")) => "Quantitative Strategist".to_string(),
        ("domains", Some("search")) => "Search & Retrieval Specialist".to_string(),
        ("domains", _) => "Domain Expert".to_string(),
        ("organizations", _) => "Organizational Practice Lead".to_string(),
        ("specialists", _) => "Specialist".to_string(),
        _ => "Team Member".to_string(),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}

/// Generate a team name from the query tokens
fn generate_team_name(query_tokens: &[String]) -> String {
    let adjectives = [
        "Iron", "Shadow", "Apex", "Storm", "Quantum", "Forge", "Nova", "Titan", "Cipher", "Pulse",
    ];
    let nouns = [
        "Legion",
        "Vanguard",
        "Council",
        "Order",
        "Assembly",
        "Coalition",
        "Syndicate",
        "Guild",
        "Brigade",
        "Collective",
    ];

    // Deterministic but varied: hash the query to pick adjective + noun
    let hash: usize = query_tokens
        .iter()
        .map(|t| t.bytes().map(|b| b as usize).sum::<usize>())
        .sum();
    let adj = adjectives[hash % adjectives.len()];
    let noun = nouns[(hash / adjectives.len()) % nouns.len()];
    format!("{} {}", adj, noun)
}

/// Generate a rationale for why this skill matches the query
fn generate_rationale(skill: &Skill, query_tokens: &[String]) -> String {
    let skill_tokens = tokenize(&format!(
        "{} {} {}",
        skill.id,
        skill.description,
        skill.tags.join(" ")
    ));
    let matching: Vec<&String> = query_tokens
        .iter()
        .filter(|qt| {
            skill_tokens
                .iter()
                .any(|st| st.contains(qt.as_str()) || qt.contains(st.as_str()))
        })
        .collect();

    if matching.is_empty() {
        format!(
            "Complements the team with {} expertise in {}",
            skill.category,
            skill.subcategory.as_deref().unwrap_or(&skill.name)
        )
    } else {
        format!(
            "Directly relevant — matches: {}. {}",
            matching
                .iter()
                .take(3)
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            truncate(&skill.description, 80)
        )
    }
}

/// Two-tier BM25: combine raw token scores (high weight) with expanded token scores (low weight)
fn two_tier_bm25(description: &str, docs: &[SkillDocument]) -> Vec<(usize, f64)> {
    let raw_tokens = tokenize(description);
    let expanded_tokens = tokenize_expanded(description);

    let raw_scores = bm25_score(&raw_tokens, docs);
    let expanded_scores = bm25_score(&expanded_tokens, docs);

    // Build a map of idx -> expanded score
    let exp_map: HashMap<usize, f64> = expanded_scores.into_iter().collect();

    // Combine: 70% raw + 30% expanded
    let mut combined: Vec<(usize, f64)> = raw_scores
        .iter()
        .map(|&(idx, raw_s)| {
            let exp_s = exp_map.get(&idx).copied().unwrap_or(0.0);
            (idx, raw_s * 0.7 + exp_s * 0.3)
        })
        .collect();

    // Also add skills that only matched on expanded tokens (not in raw results)
    let raw_idxs: std::collections::HashSet<usize> = raw_scores.iter().map(|(i, _)| *i).collect();
    for (&idx, &exp_s) in &exp_map {
        if !raw_idxs.contains(&idx) {
            combined.push((idx, exp_s * 0.3));
        }
    }

    combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    combined
}

/// The main local team assembly engine
fn local_assemble_team(description: &str, manifest: &Manifest) -> LlmTeamResponse {
    let query_tokens = tokenize(description);
    let docs = build_skill_documents(manifest);
    let scores = two_tier_bm25(description, &docs);
    let team = select_diverse_team(&scores, manifest, 6, description);

    let team_name = generate_team_name(&query_tokens);

    let members: Vec<LlmTeamMember> = team
        .iter()
        .map(|(idx, _score)| {
            let skill = &manifest.skills[*idx];
            LlmTeamMember {
                skill_id: skill.id.clone(),
                role: assign_role(skill),
                rationale: generate_rationale(skill, &query_tokens),
            }
        })
        .collect();

    let summary = if members.is_empty() {
        "No matching skills found for this description.".to_string()
    } else {
        let categories: Vec<String> = members
            .iter()
            .filter_map(|m| {
                manifest
                    .skills
                    .iter()
                    .find(|s| s.id == m.skill_id)
                    .map(|s| s.category.clone())
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        format!(
            "A {} skill team spanning {}",
            members.len(),
            categories.join(", ")
        )
    };

    LlmTeamResponse {
        team_name,
        summary,
        members,
    }
}

/// Local analysis-based team assembly
fn local_analyze_team(analysis: &str, manifest: &Manifest) -> LlmTeamResponse {
    // Extract key terms from the analysis output
    let mut query_parts: Vec<String> = Vec::new();

    for line in analysis.lines() {
        if line.starts_with("Frameworks detected:") {
            let frameworks = line.trim_start_matches("Frameworks detected:").trim();
            query_parts.push(frameworks.to_lowercase());
        }
        if line.trim().starts_with('.') {
            // File extension line like "  .rs: 42"
            let ext = line
                .trim()
                .split(':')
                .next()
                .unwrap_or("")
                .trim_start_matches('.');
            match ext {
                "rs" => query_parts.push("rust systems performance".to_string()),
                "go" => query_parts.push("go golang concurrency".to_string()),
                "py" => query_parts.push("python".to_string()),
                "js" | "jsx" => query_parts.push("javascript".to_string()),
                "ts" | "tsx" => query_parts.push("typescript javascript".to_string()),
                "c" | "h" => query_parts.push("c systems".to_string()),
                "cpp" | "cc" | "cxx" | "hpp" => query_parts.push("c++ cpp".to_string()),
                "rb" => query_parts.push("ruby".to_string()),
                "java" => query_parts.push("java".to_string()),
                "zig" => query_parts.push("zig systems".to_string()),
                "md" => query_parts.push("documentation".to_string()),
                "yml" | "yaml" => query_parts.push("configuration infrastructure".to_string()),
                "toml" => query_parts.push("rust configuration".to_string()),
                "sql" => query_parts.push("database sql".to_string()),
                "proto" => query_parts.push("api distributed".to_string()),
                _ => {}
            }
        }
    }

    let combined = query_parts.join(" ");
    local_assemble_team(&combined, manifest)
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

fn cmd_team_assemble(description: &str, auto_install: bool, global: bool, ai: bool) -> Result<()> {
    let manifest = load_manifest()?;

    let team = if ai {
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
        parse_llm_team(&raw)?
    } else {
        println!(
            "\n{} {}",
            "⚙".bold(),
            "Local NLP engine (BM25 + tag affinity + category diversity)".dimmed()
        );
        local_assemble_team(description, &manifest)
    };

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

fn cmd_team_analyze(path: &Path, auto_install: bool, global: bool, ai: bool) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Scanning project...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let analysis = analyze_project(path)?;
    let manifest = load_manifest()?;

    let team = if ai {
        pb.set_message("Project scanned. Consulting the AI oracle...");
        let catalog = build_skill_catalog(&manifest);
        let prompt = build_analyze_prompt(&analysis, &catalog);
        let raw = call_llm(&prompt)?;
        pb.finish_and_clear();
        parse_llm_team(&raw)?
    } else {
        pb.finish_and_clear();
        println!(
            "\n{} {}",
            "⚙".bold(),
            "Local NLP engine (BM25 + tag affinity + category diversity)".dimmed()
        );
        local_analyze_team(&analysis, &manifest)
    };

    println!("\n{}", "Project Analysis".bold().cyan());
    println!("{}", "─".repeat(40).dimmed());
    for line in analysis.lines() {
        println!("  {}", line.dimmed());
    }
    println!();

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
                ai,
            } => cmd_team_assemble(&description, install, global, ai),
            TeamCommand::Analyze {
                path,
                install,
                global,
                ai,
            } => cmd_team_analyze(&path, install, global, ai),
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
