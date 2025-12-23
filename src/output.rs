use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use colored::Colorize;
use serde::Serialize;
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::cli::OutputFormat;

/// Global output format setting (thread-safe)
/// 0 = Table, 1 = Json, 2 = Compact
static OUTPUT_FORMAT: AtomicU8 = AtomicU8::new(0);
static QUIET_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_format(format: OutputFormat) {
    let value = match format {
        OutputFormat::Table => 0,
        OutputFormat::Json => 1,
        OutputFormat::Compact => 2,
    };
    OUTPUT_FORMAT.store(value, Ordering::Relaxed);
}

pub fn get_format() -> OutputFormat {
    match OUTPUT_FORMAT.load(Ordering::Relaxed) {
        1 => OutputFormat::Json,
        2 => OutputFormat::Compact,
        _ => OutputFormat::Table,
    }
}

pub fn set_quiet(quiet: bool) {
    QUIET_MODE.store(quiet, Ordering::Relaxed);
}

pub fn is_quiet() -> bool {
    QUIET_MODE.load(Ordering::Relaxed)
}

pub fn is_json_output() -> bool {
    matches!(get_format(), OutputFormat::Json)
}

/// Print a table, JSON, or compact output depending on format
pub fn print_table<T, R>(items: &[T], to_row: impl Fn(&T) -> R, to_compact: impl Fn(&T) -> String)
where
    T: Serialize,
    R: Tabled,
{
    match get_format() {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(items)
                    .unwrap_or_else(|_| "<serialization error>".to_string())
            );
        }
        OutputFormat::Compact => {
            for item in items {
                println!("{}", to_compact(item));
            }
        }
        OutputFormat::Table => {
            let rows: Vec<R> = items.iter().map(|item| to_row(item)).collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{table}");
        }
    }
}

/// Print a single item as JSON or custom display
pub fn print_item<T: Serialize>(item: &T, display: impl FnOnce(&T)) {
    if is_json_output() {
        println!(
            "{}",
            serde_json::to_string_pretty(item)
                .unwrap_or_else(|_| "<serialization error>".to_string())
        );
    } else {
        display(item);
    }
}

/// Print a success message (respects quiet mode)
pub fn print_message(message: &str) {
    if is_quiet() {
        return;
    }
    if is_json_output() {
        println!(r#"{{"message": "{}"}}"#, message.replace('"', "\\\""));
    } else {
        println!("{message}");
    }
}

/// Format status with color based on state type
pub fn status_colored(status: &str, color: Option<&str>) -> String {
    if let Some(hex) = color {
        if let Ok((r, g, b)) = parse_hex_color(hex) {
            return status.truecolor(r, g, b).to_string();
        }
    }

    // Fallback colors based on status name
    let lower = status.to_lowercase();
    if lower.contains("done") || lower.contains("complete") || lower.contains("closed") {
        status.green().to_string()
    } else if lower.contains("progress") || lower.contains("started") {
        status.blue().to_string()
    } else if lower.contains("review") {
        status.magenta().to_string()
    } else if lower.contains("blocked") || lower.contains("canceled") || lower.contains("cancelled")
    {
        status.red().to_string()
    } else if lower.contains("backlog") || lower.contains("triage") {
        status.bright_black().to_string()
    } else {
        status.to_string()
    }
}

fn parse_hex_color(hex: &str) -> Result<(u8, u8, u8), ()> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(());
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
    Ok((r, g, b))
}

/// Format a date string nicely using chrono
pub fn format_date(iso: &str) -> String {
    use chrono::{DateTime, Local, Utc};

    if let Ok(dt) = iso.parse::<DateTime<Utc>>() {
        let local: DateTime<Local> = dt.into();
        local.format("%Y-%m-%d %H:%M").to_string()
    } else {
        iso.split('T').next().unwrap_or(iso).to_string()
    }
}

/// Format a date string as date only
pub fn format_date_only(iso: &str) -> String {
    use chrono::{DateTime, Utc};

    if let Ok(dt) = iso.parse::<DateTime<Utc>>() {
        dt.format("%Y-%m-%d").to_string()
    } else {
        iso.split('T').next().unwrap_or(iso).to_string()
    }
}

/// Format a relative time (e.g., "2 days ago")
pub fn format_relative(iso: &str) -> String {
    use chrono::{DateTime, Utc};

    if let Ok(dt) = iso.parse::<DateTime<Utc>>() {
        let now = Utc::now();
        let diff = now.signed_duration_since(dt);

        if diff.num_seconds() < 60 {
            "just now".to_string()
        } else if diff.num_minutes() < 60 {
            let mins = diff.num_minutes();
            format!("{} min{} ago", mins, if mins == 1 { "" } else { "s" })
        } else if diff.num_hours() < 24 {
            let hours = diff.num_hours();
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else if diff.num_days() < 30 {
            let days = diff.num_days();
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        } else {
            format_date_only(iso)
        }
    } else {
        iso.split('T').next().unwrap_or(iso).to_string()
    }
}

/// Truncate a string with ellipsis (unicode-safe)
pub fn truncate(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}
