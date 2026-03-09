use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

// ─── Zero-Width Character Alphabet ───────────────────────────────────────────
// Invisible Unicode characters used to encode binary data within markdown.
// These render as nothing in browsers, editors, and terminals.
mod zwc {
    pub const BIT_ZERO: char = '\u{200B}'; // Zero Width Space
    pub const BIT_ONE: char = '\u{200C}'; // Zero Width Non-Joiner
    pub const BYTE_SEP: char = '\u{200D}'; // Zero Width Joiner
    pub const FRAME: char = '\u{2060}'; // Word Joiner
}

// ─── Protocol Constants ──────────────────────────────────────────────────────
const MAGIC: [u8; 4] = *b"SK1L";
const VERSION: u8 = 0x01;
const PAYLOAD_LEN: usize = 23; // 4 magic + 1 ver + 8 origin + 4 ts + 4 path + 2 cksum
const ORIGIN_IDENTITY: &[u8] = b"copyleftdev/sk1llz";
const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", ".github", "media"];

// ─── CLI ─────────────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(name = "sk1llz-dna")]
#[command(version)]
#[command(about = "Steganographic zero-width character fingerprinting for sk1llz markdown")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inject fingerprints into all unstamped markdown files
    Inject {
        /// Root directory to scan
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Show what would change without modifying files
        #[arg(long)]
        dry_run: bool,
    },
    /// Verify fingerprints in all markdown files (exit 1 if any missing/invalid)
    Verify {
        /// Root directory to scan
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Strip fingerprints from all markdown files (debugging only)
    Strip {
        /// Root directory to scan
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Decode and display the fingerprint from a single file
    Decode {
        /// Path to the markdown file
        file: PathBuf,
    },
}

// ─── Data Structures ─────────────────────────────────────────────────────────
#[derive(Debug)]
struct Fingerprint {
    version: u8,
    origin: [u8; 8],
    timestamp: u32,
    path_hash: [u8; 4],
}

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let origin_hex: String = self.origin.iter().map(|b| format!("{:02x}", b)).collect();
        let path_hex: String = self
            .path_hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        let ts = chrono_free_timestamp(self.timestamp);
        write!(
            f,
            "v{} origin={} ts={} ({}) path={}",
            self.version, origin_hex, self.timestamp, ts, path_hex
        )
    }
}

fn chrono_free_timestamp(epoch: u32) -> String {
    // Simple epoch-to-date without pulling in chrono
    let secs = epoch as u64;
    let days = secs / 86400;
    // Approximate year/month/day from days since 1970-01-01
    let mut y = 1970u64;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let months_days: Vec<u64> = if is_leap(y) {
        vec![31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u64;
    for md in &months_days {
        if remaining < *md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

// ─── Crypto Helpers ──────────────────────────────────────────────────────────
fn origin_hash() -> [u8; 8] {
    let hash = Sha256::digest(ORIGIN_IDENTITY);
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}

fn path_hash(relative_path: &str) -> [u8; 4] {
    let normalized = relative_path.replace('\\', "/");
    let hash = Sha256::digest(normalized.as_bytes());
    let mut out = [0u8; 4];
    out.copy_from_slice(&hash[..4]);
    out
}

fn compute_checksum(data: &[u8]) -> [u8; 2] {
    let sum: u16 = data.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16));
    sum.to_be_bytes()
}

// ─── Payload Construction ────────────────────────────────────────────────────
fn build_payload(relative_path: &str) -> Vec<u8> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs() as u32;

    let mut payload = Vec::with_capacity(PAYLOAD_LEN);
    payload.extend_from_slice(&MAGIC);
    payload.push(VERSION);
    payload.extend_from_slice(&origin_hash());
    payload.extend_from_slice(&now.to_be_bytes());
    payload.extend_from_slice(&path_hash(relative_path));

    let cksum = compute_checksum(&payload);
    payload.extend_from_slice(&cksum);

    debug_assert_eq!(payload.len(), PAYLOAD_LEN);
    payload
}

fn verify_payload(payload: &[u8]) -> Result<Fingerprint> {
    if payload.len() != PAYLOAD_LEN {
        bail!(
            "payload length {} != expected {}",
            payload.len(),
            PAYLOAD_LEN
        );
    }
    if payload[0..4] != MAGIC {
        bail!("invalid magic bytes");
    }
    let version = payload[4];
    if version != VERSION {
        bail!("unknown version {}", version);
    }

    let mut origin = [0u8; 8];
    origin.copy_from_slice(&payload[5..13]);
    if origin != origin_hash() {
        bail!("origin mismatch — content may originate from a different source");
    }

    let timestamp = u32::from_be_bytes([payload[13], payload[14], payload[15], payload[16]]);

    let mut ph = [0u8; 4];
    ph.copy_from_slice(&payload[17..21]);

    let stored = u16::from_be_bytes([payload[21], payload[22]]);
    let computed = u16::from_be_bytes(compute_checksum(&payload[..21]));
    if stored != computed {
        bail!(
            "checksum mismatch: stored={:#06x} computed={:#06x}",
            stored,
            computed
        );
    }

    Ok(Fingerprint {
        version,
        origin,
        timestamp,
        path_hash: ph,
    })
}

// ─── ZWC Encoding / Decoding ─────────────────────────────────────────────────
fn frame_marker() -> String {
    format!("{}{}{}", zwc::FRAME, zwc::BYTE_SEP, zwc::FRAME)
}

fn encode_byte(b: u8) -> String {
    (0..8)
        .rev()
        .map(|i| {
            if (b >> i) & 1 == 1 {
                zwc::BIT_ONE
            } else {
                zwc::BIT_ZERO
            }
        })
        .collect()
}

fn encode_fingerprint(payload: &[u8]) -> String {
    let marker = frame_marker();
    let mut out = String::with_capacity(512);
    out.push_str(&marker);

    for (i, &byte) in payload.iter().enumerate() {
        out.push_str(&encode_byte(byte));
        if i < payload.len() - 1 {
            out.push(zwc::BYTE_SEP);
        }
    }

    out.push_str(&marker);
    out
}

fn decode_fingerprint(content: &str) -> Option<Vec<u8>> {
    let marker = frame_marker();
    let start = content.find(&marker)?;
    let after = start + marker.len();
    let end = content[after..].find(&marker)?;
    let encoded = &content[after..after + end];

    let mut bytes = Vec::with_capacity(PAYLOAD_LEN);
    let mut current: u8 = 0;
    let mut bits: u8 = 0;

    for ch in encoded.chars() {
        match ch {
            c if c == zwc::BIT_ZERO => {
                current <<= 1;
                bits += 1;
            }
            c if c == zwc::BIT_ONE => {
                current = (current << 1) | 1;
                bits += 1;
            }
            c if c == zwc::BYTE_SEP => continue,
            _ => return None,
        }
        if bits == 8 {
            bytes.push(current);
            current = 0;
            bits = 0;
        }
    }

    if bytes.is_empty() {
        None
    } else {
        Some(bytes)
    }
}

fn has_fingerprint(content: &str) -> bool {
    let marker = frame_marker();
    if let Some(start) = content.find(&marker) {
        let after = start + marker.len();
        content[after..].contains(&marker)
    } else {
        false
    }
}

fn strip_fingerprint(content: &str) -> Option<String> {
    let marker = frame_marker();
    let start = content.find(&marker)?;
    let after = start + marker.len();
    let end = content[after..].find(&marker)?;
    let total_end = after + end + marker.len();

    let mut result = String::with_capacity(content.len());
    result.push_str(&content[..start]);
    result.push_str(&content[total_end..]);
    Some(result)
}

// ─── Content Injection ───────────────────────────────────────────────────────
fn inject_into_content(content: &str, fingerprint: &str) -> String {
    // Target: end of first markdown heading line (after any YAML frontmatter)
    let in_frontmatter = content.starts_with("---");
    let mut past_frontmatter = !in_frontmatter;
    let mut frontmatter_end_seen = false;

    // Find target line index
    let target_idx = content
        .lines()
        .enumerate()
        .find(|(i, line)| {
            if in_frontmatter && !past_frontmatter {
                if *i > 0 && line.starts_with("---") {
                    past_frontmatter = true;
                    frontmatter_end_seen = true;
                }
                return false;
            }
            line.starts_with('#')
        })
        .map(|(i, _)| i);

    // Fallback: first non-empty line after frontmatter
    let target_idx = target_idx.or_else(|| {
        let mut past = !in_frontmatter;
        content
            .lines()
            .enumerate()
            .find(|(i, line)| {
                if in_frontmatter && !past {
                    if *i > 0 && line.starts_with("---") {
                        past = true;
                    }
                    return false;
                }
                !line.trim().is_empty()
            })
            .map(|(i, _)| i)
    });

    match target_idx {
        Some(idx) => {
            let mut result = String::with_capacity(content.len() + fingerprint.len());
            for (i, line) in content.lines().enumerate() {
                result.push_str(line);
                if i == idx {
                    result.push_str(fingerprint);
                }
                result.push('\n');
            }
            // Preserve original trailing-newline behavior
            if !content.ends_with('\n') {
                result.pop();
            }
            result
        }
        None => {
            // Empty or whitespace-only file — prepend fingerprint
            format!("{}\n{}", fingerprint, content)
        }
    }
}

// ─── File Discovery ──────────────────────────────────────────────────────────
fn find_markdown_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.contains(&name.as_ref())
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().map(|ext| ext == "md").unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}

fn relative_path(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

// ─── Commands ────────────────────────────────────────────────────────────────
fn cmd_inject(root: &Path, dry_run: bool) -> Result<()> {
    let root = root.canonicalize().context("cannot resolve root path")?;
    let files = find_markdown_files(&root);

    let mut stamped = 0u32;
    let mut skipped = 0u32;
    let mut errors = 0u32;

    for file in &files {
        let rel = relative_path(&root, file);
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  {} {} — {}", "ERR".red().bold(), rel.dimmed(), e);
                errors += 1;
                continue;
            }
        };

        if has_fingerprint(&content) {
            skipped += 1;
            continue;
        }

        let payload = build_payload(&rel);
        let fp_str = encode_fingerprint(&payload);
        let new_content = inject_into_content(&content, &fp_str);

        if dry_run {
            println!("  {} {}", "DRY".yellow().bold(), rel);
        } else {
            fs::write(file, &new_content).with_context(|| format!("failed to write {}", rel))?;
            println!("  {} {}", "INJ".green().bold(), rel);
        }
        stamped += 1;
    }

    println!();
    let mode = if dry_run { " (dry run)" } else { "" };
    println!(
        "{} {} stamped, {} already stamped, {} errors{}",
        "DNA".cyan().bold(),
        stamped.to_string().green(),
        skipped.to_string().dimmed(),
        if errors > 0 {
            errors.to_string().red().to_string()
        } else {
            errors.to_string().dimmed().to_string()
        },
        mode.dimmed()
    );

    Ok(())
}

fn cmd_verify(root: &Path) -> Result<()> {
    let root = root.canonicalize().context("cannot resolve root path")?;
    let files = find_markdown_files(&root);

    let mut valid = 0u32;
    let mut missing = 0u32;
    let mut invalid = 0u32;
    let mut failed_files: Vec<String> = Vec::new();

    for file in &files {
        let rel = relative_path(&root, file);
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  {} {} — {}", "ERR".red().bold(), rel.dimmed(), e);
                invalid += 1;
                failed_files.push(rel);
                continue;
            }
        };

        if !has_fingerprint(&content) {
            println!("  {} {}", "MISS".yellow().bold(), rel);
            missing += 1;
            failed_files.push(rel);
            continue;
        }

        match decode_fingerprint(&content) {
            Some(payload) => match verify_payload(&payload) {
                Ok(fp) => {
                    println!("  {} {} [{}]", "OK".green().bold(), rel, fp);
                    valid += 1;
                }
                Err(e) => {
                    println!("  {} {} — {}", "BAD".red().bold(), rel, e);
                    invalid += 1;
                    failed_files.push(rel);
                }
            },
            None => {
                println!("  {} {} — corrupt encoding", "BAD".red().bold(), rel);
                invalid += 1;
                failed_files.push(rel);
            }
        }
    }

    println!();
    println!(
        "{} {} valid, {} missing, {} invalid — {} total files",
        "DNA".cyan().bold(),
        valid.to_string().green(),
        if missing > 0 {
            missing.to_string().yellow().to_string()
        } else {
            missing.to_string().dimmed().to_string()
        },
        if invalid > 0 {
            invalid.to_string().red().to_string()
        } else {
            invalid.to_string().dimmed().to_string()
        },
        files.len()
    );

    if !failed_files.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

fn cmd_strip(root: &Path) -> Result<()> {
    let root = root.canonicalize().context("cannot resolve root path")?;
    let files = find_markdown_files(&root);
    let mut stripped = 0u32;

    for file in &files {
        let rel = relative_path(&root, file);
        let content = fs::read_to_string(file).with_context(|| format!("cannot read {}", rel))?;

        if !has_fingerprint(&content) {
            continue;
        }

        if let Some(clean) = strip_fingerprint(&content) {
            fs::write(file, &clean).with_context(|| format!("cannot write {}", rel))?;
            println!("  {} {}", "DEL".yellow().bold(), rel);
            stripped += 1;
        }
    }

    println!();
    println!(
        "{} {} fingerprints stripped",
        "DNA".cyan().bold(),
        stripped.to_string().yellow()
    );
    Ok(())
}

fn cmd_decode(file: &Path) -> Result<()> {
    let content =
        fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))?;

    if !has_fingerprint(&content) {
        println!(
            "{} No fingerprint found in {}",
            "DNA".cyan().bold(),
            file.display().to_string().yellow()
        );
        std::process::exit(1);
    }

    let payload =
        decode_fingerprint(&content).context("fingerprint present but encoding is corrupt")?;

    let hex: String = payload.iter().map(|b| format!("{:02x}", b)).collect();
    println!("{} {}", "Raw".bold(), hex.dimmed());

    match verify_payload(&payload) {
        Ok(fp) => {
            let origin_hex: String = fp.origin.iter().map(|b| format!("{:02x}", b)).collect();
            let path_hex: String = fp.path_hash.iter().map(|b| format!("{:02x}", b)).collect();
            println!("{} {}", "Version".bold(), fp.version);
            println!("{} {}", "Origin".bold(), origin_hex.green());
            println!(
                "{} {} ({})",
                "Stamped".bold(),
                fp.timestamp,
                chrono_free_timestamp(fp.timestamp)
            );
            println!("{} {}", "PathHash".bold(), path_hex);
            println!("{} {}", "Status".bold(), "VALID".green().bold());
        }
        Err(e) => {
            println!("{} {} — {}", "Status".bold(), "INVALID".red().bold(), e);
            std::process::exit(1);
        }
    }

    Ok(())
}

// ─── Main ────────────────────────────────────────────────────────────────────
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inject { path, dry_run } => cmd_inject(&path, dry_run),
        Commands::Verify { path } => cmd_verify(&path),
        Commands::Strip { path } => cmd_strip(&path),
        Commands::Decode { file } => cmd_decode(&file),
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_byte_roundtrip() {
        for b in 0..=255u8 {
            let encoded = encode_byte(b);
            let mut decoded: u8 = 0;
            let mut bits = 0u8;
            for ch in encoded.chars() {
                match ch {
                    c if c == zwc::BIT_ZERO => decoded <<= 1,
                    c if c == zwc::BIT_ONE => decoded = (decoded << 1) | 1,
                    _ => panic!("unexpected char in encoded byte"),
                }
                bits += 1;
            }
            assert_eq!(bits, 8);
            assert_eq!(decoded, b, "roundtrip failed for byte {}", b);
        }
    }

    #[test]
    fn test_encode_decode_fingerprint_roundtrip() {
        let payload = build_payload("test/file.md");
        assert_eq!(payload.len(), PAYLOAD_LEN);

        let encoded = encode_fingerprint(&payload);
        let decoded = decode_fingerprint(&encoded).expect("decode failed");
        assert_eq!(decoded, payload);
    }

    #[test]
    fn test_payload_verification() {
        let payload = build_payload("some/path.md");
        let fp = verify_payload(&payload).expect("verification failed");
        assert_eq!(fp.version, VERSION);
        assert_eq!(fp.origin, origin_hash());
        assert_eq!(fp.path_hash, path_hash("some/path.md"));
    }

    #[test]
    fn test_checksum_tamper_detection() {
        let mut payload = build_payload("test.md");
        // Flip a bit in the origin
        payload[5] ^= 0x01;
        assert!(verify_payload(&payload).is_err());
    }

    #[test]
    fn test_has_fingerprint() {
        let payload = build_payload("test.md");
        let fp = encode_fingerprint(&payload);

        assert!(has_fingerprint(&format!("# Hello{}\nworld", fp)));
        assert!(!has_fingerprint("# Hello\nworld"));
        assert!(!has_fingerprint(""));
    }

    #[test]
    fn test_strip_fingerprint() {
        let payload = build_payload("test.md");
        let fp = encode_fingerprint(&payload);
        let content = format!("# Hello{}\nworld\n", fp);

        let stripped = strip_fingerprint(&content).expect("strip failed");
        assert_eq!(stripped, "# Hello\nworld\n");
        assert!(!has_fingerprint(&stripped));
    }

    #[test]
    fn test_inject_heading() {
        let content = "# My Title\n\nSome content.\n";
        let fp = encode_fingerprint(&build_payload("test.md"));
        let injected = inject_into_content(content, &fp);

        assert!(has_fingerprint(&injected));
        // The heading line should contain the fingerprint
        let first_heading = injected.lines().find(|l| l.starts_with('#')).unwrap();
        assert!(first_heading.starts_with("# My Title"));
        assert!(has_fingerprint(first_heading));
    }

    #[test]
    fn test_inject_with_frontmatter() {
        let content = "---\nname: test\n---\n\n# Heading\n\nBody.\n";
        let fp = encode_fingerprint(&build_payload("test.md"));
        let injected = inject_into_content(content, &fp);

        assert!(has_fingerprint(&injected));
        // Frontmatter should be untouched
        assert!(injected.starts_with("---\nname: test\n---\n"));
    }

    #[test]
    fn test_inject_idempotent() {
        let content = "# Title\nBody\n";
        let fp = encode_fingerprint(&build_payload("t.md"));
        let first = inject_into_content(content, &fp);
        // Second inject should be caught by has_fingerprint in the command,
        // but verify the content is still valid
        assert!(has_fingerprint(&first));
        let decoded = decode_fingerprint(&first).unwrap();
        assert!(verify_payload(&decoded).is_ok());
    }

    #[test]
    fn test_frame_marker_uniqueness() {
        // Frame marker should not appear in normal markdown
        let marker = frame_marker();
        let normal_md = "# Hello World\n\nThis is **bold** and `code`.\n";
        assert!(!normal_md.contains(&marker));
    }

    #[test]
    fn test_origin_hash_deterministic() {
        let h1 = origin_hash();
        let h2 = origin_hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_path_hash_normalization() {
        // Windows and Unix paths should produce the same hash
        assert_eq!(
            path_hash("domains/testing/beck-tdd/SKILL.md"),
            path_hash("domains\\testing\\beck-tdd\\SKILL.md")
        );
    }

    #[test]
    fn test_invisible_in_content() {
        let payload = build_payload("test.md");
        let fp = encode_fingerprint(&payload);
        // All characters in the fingerprint should be zero-width
        for ch in fp.chars() {
            assert!(
                ch == zwc::BIT_ZERO
                    || ch == zwc::BIT_ONE
                    || ch == zwc::BYTE_SEP
                    || ch == zwc::FRAME,
                "visible character found: U+{:04X}",
                ch as u32
            );
        }
    }

    #[test]
    fn test_chrono_free_timestamp() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        let ts = chrono_free_timestamp(1704067200);
        assert_eq!(ts, "2024-01-01");
    }
}
