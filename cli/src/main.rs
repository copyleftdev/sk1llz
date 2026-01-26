use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use colored::Colorize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::SystemTime;

const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/copyleftdev/sk1llz/master/skills.json";
const RAW_BASE_URL: &str = "https://raw.githubusercontent.com/copyleftdev/sk1llz/master";

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

    let response = reqwest::blocking::get(MANIFEST_URL)
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

        let url = format!("{}/{}/{}", RAW_BASE_URL, skill.path, file);
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
    match reqwest::blocking::get(MANIFEST_URL) {
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
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
    }
}
