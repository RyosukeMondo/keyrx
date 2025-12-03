//! Key name suggestion engine using Levenshtein distance.
//!
//! Provides similar key name suggestions for typos in user scripts.

use crate::drivers::keycodes::key_definitions;
use crate::validation::config::ValidationConfig;

/// Suggests similar valid key names for an invalid key name.
///
/// Uses Levenshtein distance to find the closest matches from all
/// known key names and aliases.
///
/// # Arguments
/// * `invalid` - The invalid key name to find suggestions for
/// * `config` - Validation config with max_suggestions and similarity_threshold
///
/// # Returns
/// Up to `config.max_suggestions` similar key names within `config.similarity_threshold` distance
pub fn suggest_similar_keys(invalid: &str, config: &ValidationConfig) -> Vec<String> {
    let invalid_upper = invalid.to_uppercase();
    let definitions = key_definitions();

    // First, check if the input is already a valid key (exact match)
    for def in &definitions {
        if def.name.to_uppercase() == invalid_upper {
            return Vec::new(); // Exact match to primary name
        }
        for alias in def.aliases {
            if *alias == invalid_upper {
                return Vec::new(); // Exact match to alias
            }
        }
    }

    // Collect similar names: primary names + aliases
    let mut candidates: Vec<(&str, usize)> = Vec::new();

    for def in &definitions {
        // Check primary name
        let dist = strsim::levenshtein(&invalid_upper, &def.name.to_uppercase());
        if dist <= config.similarity_threshold {
            candidates.push((def.name, dist));
        }

        // Check aliases
        for alias in def.aliases {
            let dist = strsim::levenshtein(&invalid_upper, alias);
            if dist <= config.similarity_threshold {
                // Use the primary name for suggestions, not the alias
                candidates.push((def.name, dist));
            }
        }
    }

    // Sort by distance (closest first), then by name for stability
    candidates.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    candidates
        .into_iter()
        .filter(|(name, _)| seen.insert(*name))
        .take(config.max_suggestions)
        .map(|(name, _)| name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> ValidationConfig {
        ValidationConfig::default()
    }

    #[test]
    fn escpe_suggests_escape() {
        let suggestions = suggest_similar_keys("Escpe", &default_config());
        assert!(suggestions.contains(&"Escape".to_string()));
    }

    #[test]
    fn leftcrtl_suggests_leftctrl() {
        let suggestions = suggest_similar_keys("LeftCrtl", &default_config());
        assert!(suggestions.contains(&"LeftCtrl".to_string()));
    }

    #[test]
    fn capslok_suggests_capslock() {
        let suggestions = suggest_similar_keys("Capslok", &default_config());
        assert!(suggestions.contains(&"CapsLock".to_string()));
    }

    #[test]
    fn respects_max_suggestions() {
        let mut config = default_config();
        config.max_suggestions = 2;
        config.similarity_threshold = 5; // Higher threshold to get more matches

        let suggestions = suggest_similar_keys("F", &config);
        assert!(suggestions.len() <= 2);
    }

    #[test]
    fn respects_similarity_threshold() {
        let mut config = default_config();
        config.similarity_threshold = 1; // Very strict

        // "Escapee" has distance 1 from "Escape" (one extra 'e')
        let suggestions = suggest_similar_keys("Escapee", &config);
        assert!(suggestions.contains(&"Escape".to_string()));

        // "Xyz123" should have no suggestions with threshold 1
        let suggestions = suggest_similar_keys("Xyz123", &config);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn no_suggestions_for_completely_wrong() {
        let suggestions = suggest_similar_keys("xyz123456789", &default_config());
        assert!(suggestions.is_empty());
    }

    #[test]
    fn exact_match_returns_empty() {
        // If the key is valid, distance is 0, so no suggestions
        let suggestions = suggest_similar_keys("Escape", &default_config());
        assert!(suggestions.is_empty());
    }

    #[test]
    fn case_insensitive_matching() {
        let suggestions = suggest_similar_keys("escpe", &default_config());
        assert!(suggestions.contains(&"Escape".to_string()));

        let suggestions = suggest_similar_keys("ESCPE", &default_config());
        assert!(suggestions.contains(&"Escape".to_string()));
    }
}
