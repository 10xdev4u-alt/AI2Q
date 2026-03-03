use regex::Regex;

pub struct TenantScoper;

impl TenantScoper {
    pub fn apply_scope(query: &str, tenant_id: &str, tenant_column: &str) -> String {
        let scope_clause = format!("{} = '{}'", tenant_column, tenant_id);

        let re_where = Regex::new(r"(?i)\bWHERE\b").unwrap();
        if re_where.is_match(query) {
            query.replacen("WHERE", &format!("WHERE {} AND", scope_clause), 1)
        } else {
            let re_tail = Regex::new(r"(?i)\b(GROUP|ORDER|LIMIT)\b").unwrap();
            if let Some(m) = re_tail.find(query) {
                let (head, tail) = query.split_at(m.start());
                format!("{} WHERE {} {}", head.trim(), scope_clause, tail)
            } else {
                format!("{} WHERE {}", query.trim(), scope_clause)
            }
        }
    }
}
