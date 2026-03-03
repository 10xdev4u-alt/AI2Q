use crate::PrivacyGuard;
use async_trait::async_trait;
use regex::Regex;

pub struct SimplePrivacyGuard {
    email_regex: Regex,
    card_regex: Regex,
}

impl SimplePrivacyGuard {
    pub fn new() -> Self {
        Self {
            email_regex: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            card_regex: Regex::new(r"\b(?:\d[ -]*?){13,16}\b").unwrap(),
        }
    }
}

#[async_trait]
impl PrivacyGuard for SimplePrivacyGuard {
    async fn scrub_prompt(&self, prompt: &str) -> anyhow::Result<String> {
        let mut scrubbed = prompt.to_string();
        scrubbed = self.email_regex.replace_all(&scrubbed, "[EMAIL]").to_string();
        scrubbed = self.card_regex.replace_all(&scrubbed, "[CARD]").to_string();
        Ok(scrubbed)
    }

    async fn mask_results(&self, data: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // Recursively mask strings in JSON
        let mut masked_data = data.clone();
        self.mask_value(&mut masked_data);
        Ok(masked_data)
    }
}

impl SimplePrivacyGuard {
    fn mask_value(&self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(s) => {
                *s = self.email_regex.replace_all(s, "[EMAIL]").to_string();
                *s = self.card_regex.replace_all(s, "[CARD]").to_string();
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    self.mask_value(v);
                }
            }
            serde_json::Value::Object(map) => {
                for v in map.values_mut() {
                    self.mask_value(v);
                }
            }
            _ => {}
        }
    }
}
