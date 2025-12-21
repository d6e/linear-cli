use colored::Colorize;
use serde::Serialize;
use tabled::{Table, Tabled, settings::Style};

/// Global output format setting
static mut OUTPUT_JSON: bool = false;

pub fn set_json_output(json: bool) {
    unsafe {
        OUTPUT_JSON = json;
    }
}

pub fn is_json_output() -> bool {
    unsafe { OUTPUT_JSON }
}

/// Print a table or JSON depending on output mode
pub fn print_table<T, R, F>(items: &[T], to_row: F)
where
    T: Serialize,
    R: Tabled,
    F: Fn(&T) -> R,
{
    if is_json_output() {
        println!("{}", serde_json::to_string_pretty(items).unwrap_or_default());
    } else {
        let rows: Vec<R> = items.iter().map(|item| to_row(item)).collect();
        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{table}");
    }
}

/// Print a single item or JSON depending on output mode
pub fn print_item<T: Serialize>(item: &T, display: impl FnOnce(&T)) {
    if is_json_output() {
        println!("{}", serde_json::to_string_pretty(item).unwrap_or_default());
    } else {
        display(item);
    }
}

/// Print a message (skipped in JSON mode, or prints simple object)
pub fn print_message(message: &str) {
    if is_json_output() {
        println!(r#"{{"message": "{}"}}"#, message.replace('"', "\\\""));
    } else {
        println!("{message}");
    }
}

/// Format priority with color
pub fn priority_colored(priority: i32) -> String {
    let label = priority_label(priority);
    match priority {
        1 => label.red().bold().to_string(),
        2 => label.yellow().bold().to_string(),
        3 => label.blue().to_string(),
        4 => label.bright_black().to_string(),
        _ => label,
    }
}

/// Get priority label without color
pub fn priority_label(priority: i32) -> String {
    match priority {
        0 => "None".to_string(),
        1 => "Urgent".to_string(),
        2 => "High".to_string(),
        3 => "Medium".to_string(),
        4 => "Low".to_string(),
        _ => format!("P{priority}"),
    }
}

/// Format status with color based on state type
pub fn status_colored(status: &str, color: Option<&str>) -> String {
    if let Some(hex) = color {
        // Parse hex color and apply
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
    } else if lower.contains("blocked") || lower.contains("canceled") || lower.contains("cancelled") {
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
        // Fallback: just extract date portion
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

/// Truncate a string with ellipsis
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
