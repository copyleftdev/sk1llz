use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::Colorize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/copyleftdev/sk1llz/master/skills.json";
const RAW_BASE_URL: &str = "https://raw.githubusercontent.com/copyleftdev/sk1llz/master";

#[derive(Parser)]
#[command(name = "sk1llz")]
#[command(author = "copyleftdev")]
#[command(version)]
#[command(about = "A package manager for AI coding skills", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        /// Target directory (default: ~/.claude/skills)
        #[arg(short, long)]
        target: Option<PathBuf>,
    },
    /// Update the local skill index
    Update,
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

fn cmd_list(category: Option<String>) -> Result<()> {
    let manifest = load_manifest()?;

    let skills: Vec<_> = match &category {
        Some(cat) => manifest
            .skills
            .iter()
            .filter(|s| s.category.to_lowercase() == cat.to_lowercase())
            .collect(),
        None => manifest.skills.iter().collect(),
    };

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

fn cmd_search(query: &str) -> Result<()> {
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

fn cmd_info(name: &str) -> Result<()> {
    let manifest = load_manifest()?;

    let skill = manifest
        .skills
        .iter()
        .find(|s| {
            s.name.to_lowercase() == name.to_lowercase()
                || s.id.to_lowercase() == name.to_lowercase()
        })
        .context(format!("Skill '{}' not found", name))?;

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

fn cmd_install(name: &str, target: Option<PathBuf>) -> Result<()> {
    let manifest = load_manifest()?;

    let skill = manifest
        .skills
        .iter()
        .find(|s| {
            s.name.to_lowercase() == name.to_lowercase()
                || s.id.to_lowercase() == name.to_lowercase()
        })
        .context(format!("Skill '{}' not found", name))?;

    let target_dir = target.unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".claude")
            .join("skills")
            .join(&skill.name)
    });

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
        "\n{} Installed {} to {}",
        "✓".green().bold(),
        skill.name.cyan(),
        target_dir.display().to_string().green()
    );

    Ok(())
}

fn cmd_update() -> Result<()> {
    fetch_manifest()?;
    println!("{} Skill index updated.", "✓".green().bold());
    Ok(())
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
        Commands::List { category } => cmd_list(category),
        Commands::Search { query } => cmd_search(&query),
        Commands::Info { name } => cmd_info(&name),
        Commands::Install { name, target } => cmd_install(&name, target),
        Commands::Update => cmd_update(),
        Commands::Completions { shell } => cmd_completions(shell),
    }
}
