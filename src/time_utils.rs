use std::str::FromStr;

use chrono::{prelude::*, Duration};


pub fn create_timestamp() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339()
}

pub fn calculate_difference(start: &str, end: &str) -> Result<String, chrono::ParseError> {
    let start_time: DateTime<Utc> = DateTime::from_str(start)?;
    let end_time: DateTime<Utc> = DateTime::from_str(end)?;
    let duration: Duration = end_time - start_time;

    Ok(format_duration(duration))
}

fn format_duration(duration: Duration) -> String {
    let days: i64 = duration.num_days();
    let hours: i64 = duration.num_hours() % 24;
    let minutes: i64 = duration.num_minutes() % 60;
    let seconds: i64 = duration.num_seconds() % 60;

    format!("P{}DT{}H{}M{}S", days, hours, minutes, seconds)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_timestamp() {
        // Given: The function to create a timestamp

        // When: The function is called
        let timestamp = create_timestamp();

        // Then: The timestamp can be parsed back into a DateTime<Utc>
        let parsed = DateTime::parse_from_rfc3339(&timestamp);
        assert!(parsed.is_ok());
    }

    #[test]

    fn test_calculate_difference() {
        // Given: Two different timestamps
        let start = "2007-03-01T13:00:00Z";
        let end = "2007-03-02T14:30:00Z"; // 1 day, 1 hour, 30 minutes later

        // When: Calculating the difference between the two timestamps
        let result = calculate_difference(start, end);

        // Then: The result is the expected duration
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "P1DT1H30M0S");
    }

    #[test]
    fn test_calculate_difference_same_time() {
        // Given: Two identical timestamps
        let start = "2007-03-01T13:00:00Z";
        let end = "2007-03-01T13:00:00Z"; // Same time

        // When: Calculating the difference between the two timestamps
        let result = calculate_difference(start, end);

        // Then: The result is zero duration
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "P0DT0H0M0S");
    }

    #[test]

    fn test_calculate_difference_invalid_start() {
        // Given: An invalid start timestamp and a valid end timestamp
        let start = "invalid";
        let end = "2007-03-01T13:00:00Z";

        // When: Calculating the difference between the timestamps
        let result = calculate_difference(start, end);

        // Then: An error is returned
        assert!(result.is_err());
    }


    #[test]
    fn test_calculate_difference_invalid_end() {
        // Given: A valid start timestamp and an invalid end timestamp
        let start = "2007-03-01T13:00:00Z";
        let end = "invalid";

        // When: Calculating the difference between the timestamps
        let result = calculate_difference(start, end);

        // Then: An error is returned
        assert!(result.is_err());
    }
}
