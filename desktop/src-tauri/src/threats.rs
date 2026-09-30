use rand::seq::SliceRandom;
use serde::Deserialize;
use std::collections::HashMap;
use crate::models::Tier;

#[derive(Debug, Deserialize)]
pub struct Templates(pub HashMap<String, Vec<String>>);

pub fn default_templates() -> HashMap<String, Vec<String>> {
    serde_json::from_str(include_str!("../../../shared/threat_templates.json"))
        .expect("shared threat template bank should be valid JSON")
}

pub fn tier_key(tier: Tier) -> &'static str {
    match tier { Tier::Mild => "mild", Tier::Serious => "serious", Tier::Unhinged => "unhinged", Tier::Normal => "mild" }
}

pub fn choose(templates: &HashMap<String, Vec<String>>, tier: Tier, app: &str) -> Option<String> {
    let pool = templates.get(tier_key(tier))?;
    let selected = pool.choose(&mut rand::thread_rng())?.replace("{app}", app);
    Some(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_shared_templates_from_repo_bank() {
        let templates = default_templates();
        assert!(templates.contains_key("mild"));
        assert!(templates.contains_key("serious"));
        assert!(templates.contains_key("unhinged"));
    }

    #[test]
    fn chooses_message_with_app_name_substituted() {
        let templates = default_templates();
        let message = choose(&templates, Tier::Serious, "Chrome").expect("warning should exist");
        assert!(message.contains("Chrome"));
    }
}
