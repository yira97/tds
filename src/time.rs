use chrono::{DateTime, Datelike, Duration, Utc};

const ONE_HOUR_SEC: i64 = 3600;
const ONE_DAY_SEC: i64 = ONE_HOUR_SEC * 24;
const ONE_WEEK_SEC: i64 = ONE_DAY_SEC * 7;
const ONE_MONTH31_SEC: i64 = ONE_DAY_SEC * 31;

pub fn get_str_by_time(from: DateTime<Utc>, to: DateTime<Utc>) -> String {
    let (from, to) = if from > to {
        (to.timestamp(), from.timestamp())
    } else {
        (from.timestamp(), to.timestamp())
    };
    const LESS_HOUR: &str = "<1 h";
    let diff = to - from;
    if diff < ONE_HOUR_SEC {
        String::from(LESS_HOUR)
    } else if diff < ONE_DAY_SEC {
        format!("{} h", diff / ONE_HOUR_SEC)
    } else if diff < ONE_WEEK_SEC {
        format!("{} d", diff / ONE_DAY_SEC)
    } else {
        format!("{} w", diff / ONE_WEEK_SEC)
    }
}

// get_time_by_str("2 days") -> now().with_day(2)
pub fn get_time_by_str(s: &str) -> Option<DateTime<Utc>> {
    let after_digit = s.chars().position(|c| !c.is_digit(10));
    if let Some(pos) = after_digit {
        let (num, unit) = s.split_at(pos);
        if let Ok(num) = num.parse::<u32>() {
            let now = Utc::now();
            let unit = match unit.trim() {
                "h" | "hour" | "hours" => ONE_HOUR_SEC,
                "d" | "day" | "days" => ONE_DAY_SEC,
                "w" | "week" | "weeks" => ONE_WEEK_SEC,
                "M" | "month" | "months" => get_sec_in_month(now),
                "y" | "year" | "years" => get_sec_in_year(now),
                _ => 0,
            };
            if unit > 0 {
                return Some(now + Duration::seconds(num as i64 * unit));
            }
        }
    }
    None
}

fn get_sec_in_month(t: DateTime<Utc>) -> i64 {
    if let Some(d) = t.with_month(1) {
        let d = d.signed_duration_since(t).num_days();
        return d * ONE_DAY_SEC;
    }
    ONE_MONTH31_SEC
}

fn get_sec_in_year(t: DateTime<Utc>) -> i64 {
    if let Some(d) = t.with_year(1) {
        let d = d.signed_duration_since(t).num_days();
        return d * ONE_DAY_SEC;
    }
    ONE_MONTH31_SEC
}
