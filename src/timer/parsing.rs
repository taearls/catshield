//! Duration parsing utilities for Cat Shield

use super::{MAX_TIMER_SECONDS, MIN_TIMER_SECONDS};

/// Parse duration string like "30m", "2h", "1h30m" into seconds
pub fn parse_duration(s: &str) -> Result<u64, String> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return Err("Duration cannot be empty".to_string());
    }

    let mut total_seconds: u64 = 0;
    let mut current_num = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else if c == 'h' {
            if current_num.is_empty() {
                return Err("Missing number before 'h'".to_string());
            }
            let hours: u64 = current_num
                .parse()
                .map_err(|_| format!("Invalid number: {}", current_num))?;
            total_seconds += hours * 3600;
            current_num.clear();
        } else if c == 'm' {
            if current_num.is_empty() {
                return Err("Missing number before 'm'".to_string());
            }
            let minutes: u64 = current_num
                .parse()
                .map_err(|_| format!("Invalid number: {}", current_num))?;
            total_seconds += minutes * 60;
            current_num.clear();
        } else if c == 's' {
            if current_num.is_empty() {
                return Err("Missing number before 's'".to_string());
            }
            let secs: u64 = current_num
                .parse()
                .map_err(|_| format!("Invalid number: {}", current_num))?;
            total_seconds += secs;
            current_num.clear();
        } else if !c.is_whitespace() {
            return Err(format!("Invalid character in duration: '{}'", c));
        }
    }

    // If there are remaining digits without a unit, assume minutes
    if !current_num.is_empty() {
        let minutes: u64 = current_num
            .parse()
            .map_err(|_| format!("Invalid number: {}", current_num))?;
        total_seconds += minutes * 60;
    }

    if total_seconds == 0 {
        return Err("Duration must be greater than zero".to_string());
    }

    if total_seconds < MIN_TIMER_SECONDS {
        return Err(format!(
            "Duration must be at least {} seconds (1 minute)",
            MIN_TIMER_SECONDS
        ));
    }

    if total_seconds > MAX_TIMER_SECONDS {
        return Err(format!(
            "Duration must not exceed {} seconds (24 hours)",
            MAX_TIMER_SECONDS
        ));
    }

    Ok(total_seconds)
}

/// Parse a timer string (e.g., "30m", "2h", "90s") into a numeric value and unit index.
/// Returns (value_string, unit_index) where unit_index is: 0=minutes, 1=hours, 2=seconds
pub fn parse_timer_value_and_unit(timer_str: &str) -> (String, isize) {
    let trimmed = timer_str.trim().to_lowercase();

    // Try to extract number and unit
    if let Some(pos) = trimmed.find(|c: char| c.is_alphabetic()) {
        let (num_part, unit_part) = trimmed.split_at(pos);
        let unit_index = match unit_part {
            "h" | "hr" | "hrs" | "hour" | "hours" => 1,
            "s" | "sec" | "secs" | "second" | "seconds" => 2,
            _ => 0, // Default to minutes for "m", "min", etc.
        };
        (num_part.to_string(), unit_index)
    } else {
        // Just a number, assume minutes
        (trimmed, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration("30m").unwrap(), 30 * 60);
        assert_eq!(parse_duration("1m").unwrap(), 60);
        assert_eq!(parse_duration("90m").unwrap(), 90 * 60);
    }

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(parse_duration("1h").unwrap(), 3600);
        assert_eq!(parse_duration("2h").unwrap(), 2 * 3600);
        assert_eq!(parse_duration("24h").unwrap(), 24 * 3600);
    }

    #[test]
    fn test_parse_duration_combined() {
        assert_eq!(parse_duration("1h30m").unwrap(), 3600 + 30 * 60);
        assert_eq!(parse_duration("2h45m").unwrap(), 2 * 3600 + 45 * 60);
    }

    #[test]
    fn test_parse_duration_with_spaces() {
        assert_eq!(parse_duration(" 30m ").unwrap(), 30 * 60);
        assert_eq!(parse_duration("1h 30m").unwrap(), 3600 + 30 * 60);
    }

    #[test]
    fn test_parse_duration_bare_number_as_minutes() {
        // A bare number without unit is treated as minutes
        assert_eq!(parse_duration("30").unwrap(), 30 * 60);
        assert_eq!(parse_duration("60").unwrap(), 60 * 60);
    }

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(parse_duration("90s").unwrap(), 90);
        assert_eq!(parse_duration("1m30s").unwrap(), 90);
    }

    #[test]
    fn test_parse_duration_errors() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("0m").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("30x").is_err());
        assert!(parse_duration("30s").is_err()); // Less than 1 minute
        assert!(parse_duration("25h").is_err()); // More than 24 hours
    }
}
