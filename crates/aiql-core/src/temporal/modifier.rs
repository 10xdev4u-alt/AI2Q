use chrono::{DateTime, Utc};
use regex::Regex;

pub struct TemporalModifier;

impl TemporalModifier {
    pub fn modify_query(query: &str, target_time: DateTime<Utc>, timestamp_column: &str) -> String {
        let time_str = target_time.to_rfc3339();
        let where_clause = format!("{} <= '{}'", timestamp_column, time_str);

        let re_where = Regex::new(r"(?i)\bWHERE\b").unwrap();
        if re_where.is_match(query) {
            // Add to existing WHERE
            query.replacen("WHERE", &format!("WHERE {} AND", where_clause), 1)
        } else {
            // Add new WHERE before GROUP BY, ORDER BY, or LIMIT
            let re_tail = Regex::new(r"(?i)\b(GROUP|ORDER|LIMIT)\b").unwrap();
            if let Some(m) = re_tail.find(query) {
                let (head, tail) = query.split_at(m.start());
                format!("{} WHERE {} {}", head.trim(), where_clause, tail)
            } else {
                format!("{} WHERE {}", query.trim(), where_clause)
            }
        }
    }
}
