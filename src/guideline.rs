// Guideline matching engine for behavioral rules
//
// This module implements the guideline matching system that determines
// which guideline should be activated based on user input.

use crate::context::Context;
use crate::error::Result;
use crate::types::{GuidelineId, ToolId};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use chrono::{DateTime, Utc};
use regex::{Regex, RegexSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, trace};

/// Behavioral guideline defining when to activate and what to do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guideline {
    pub id: GuidelineId,
    pub condition: GuidelineCondition,
    pub action: GuidelineAction,
    pub priority: i32,
    pub tools: Vec<ToolId>,
    pub parameters: HashMap<String, ParameterDef>,
    pub created_at: DateTime<Utc>,
}

/// Condition that triggers a guideline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuidelineCondition {
    /// Exact text match (case-insensitive substring)
    Literal(String),
    /// Regex pattern match
    Regex(String),
}

/// Action to take when guideline is activated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidelineAction {
    pub response_template: String,
    pub requires_llm: bool,
    pub parameters: Vec<String>,
}

/// Parameter definition for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// Result of matching a guideline against a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidelineMatch {
    pub guideline_id: GuidelineId,
    pub relevance_score: f32,
    pub semantic_score: f32,
    pub matched_condition: String,
    pub extracted_parameters: HashMap<String, serde_json::Value>,
    pub explanation: Option<String>,
}

/// Trait for guideline matching
#[async_trait::async_trait]
pub trait GuidelineMatcher: Send + Sync {
    /// Match a user message against all guidelines
    async fn match_guidelines(
        &self,
        message: &str,
        context: &Context,
    ) -> Result<Vec<GuidelineMatch>>;

    /// Select the best matching guideline (by priority and relevance)
    async fn select_best_match(&self, matches: Vec<GuidelineMatch>) -> Option<GuidelineMatch>;

    /// Add a guideline to the matcher
    async fn add_guideline(&mut self, guideline: Guideline) -> Result<GuidelineId>;

    /// Remove a guideline
    async fn remove_guideline(&mut self, id: &GuidelineId) -> Result<()>;

    /// Get all guidelines
    fn get_guidelines(&self) -> &[Guideline];
}

/// Default implementation of guideline matching using Aho-Corasick and regex
pub struct DefaultGuidelineMatcher {
    guidelines: Vec<Guideline>,
    aho_corasick: Option<AhoCorasick>,
    /// Maps pattern index to guideline indices (for handling duplicate patterns)
    literal_pattern_to_guidelines: HashMap<usize, Vec<usize>>,
    regex_set: Option<RegexSet>,
    /// Maps pattern index to guideline indices (for handling duplicate patterns)
    regex_pattern_to_guidelines: HashMap<usize, Vec<usize>>,
    individual_regexes: Vec<Regex>,
}

impl DefaultGuidelineMatcher {
    pub fn new() -> Self {
        Self {
            guidelines: Vec::new(),
            aho_corasick: None,
            literal_pattern_to_guidelines: HashMap::new(),
            regex_set: None,
            regex_pattern_to_guidelines: HashMap::new(),
            individual_regexes: Vec::new(),
        }
    }

    /// Rebuild pattern matchers after guidelines change
    fn rebuild_matchers(&mut self) {
        // Build Aho-Corasick automaton for literal conditions
        // Track which pattern maps to which guideline indices (handle duplicates)
        let mut literals: Vec<String> = Vec::new();
        let mut literal_to_guideline_map: HashMap<String, Vec<usize>> = HashMap::new();

        for (guideline_idx, guideline) in self.guidelines.iter().enumerate() {
            if let GuidelineCondition::Literal(s) = &guideline.condition {
                let lowercase = s.to_lowercase();
                literal_to_guideline_map
                    .entry(lowercase.clone())
                    .or_insert_with(Vec::new)
                    .push(guideline_idx);
            }
        }

        // Build unique pattern list and pattern->guidelines mapping
        self.literal_pattern_to_guidelines.clear();
        for (pattern, guideline_indices) in literal_to_guideline_map {
            let pattern_idx = literals.len();
            literals.push(pattern);
            self.literal_pattern_to_guidelines
                .insert(pattern_idx, guideline_indices);
        }

        if !literals.is_empty() {
            self.aho_corasick = Some(
                AhoCorasickBuilder::new()
                    .ascii_case_insensitive(true)
                    .build(&literals)
                    .expect("Failed to build Aho-Corasick automaton"),
            );
        } else {
            self.aho_corasick = None;
        }

        // Build regex set for regex conditions
        let mut patterns: Vec<String> = Vec::new();
        let mut regex_to_guideline_map: HashMap<String, Vec<usize>> = HashMap::new();

        for (guideline_idx, guideline) in self.guidelines.iter().enumerate() {
            if let GuidelineCondition::Regex(r) = &guideline.condition {
                regex_to_guideline_map
                    .entry(r.clone())
                    .or_insert_with(Vec::new)
                    .push(guideline_idx);
            }
        }

        // Build unique pattern list and pattern->guidelines mapping
        self.regex_pattern_to_guidelines.clear();
        for (pattern, guideline_indices) in regex_to_guideline_map {
            let pattern_idx = patterns.len();
            patterns.push(pattern);
            self.regex_pattern_to_guidelines
                .insert(pattern_idx, guideline_indices);
        }

        if !patterns.is_empty() {
            // Build individual regexes for extraction
            self.individual_regexes = patterns.iter().filter_map(|p| Regex::new(p).ok()).collect();

            // Build regex set for fast matching
            self.regex_set = RegexSet::new(&patterns).ok();
        } else {
            self.regex_set = None;
            self.individual_regexes.clear();
        }
    }

    /// Match literal conditions using Aho-Corasick
    fn match_literal_conditions(&self, message: &str) -> Vec<(usize, &Guideline)> {
        let mut matches = Vec::new();

        if let Some(ref ac) = self.aho_corasick {
            let lowercase_message = message.to_lowercase();
            for mat in ac.find_iter(&lowercase_message) {
                let pattern_idx = mat.pattern().as_usize();

                // Get all guidelines that match this pattern
                if let Some(guideline_indices) =
                    self.literal_pattern_to_guidelines.get(&pattern_idx)
                {
                    for &guideline_idx in guideline_indices {
                        if let Some(guideline) = self.guidelines.get(guideline_idx) {
                            matches.push((guideline_idx, guideline));
                        }
                    }
                }
            }
        }

        matches
    }

    /// Match regex conditions using RegexSet
    fn match_regex_conditions(&self, message: &str) -> Vec<(usize, &Guideline)> {
        let mut matches = Vec::new();

        if let Some(ref regex_set) = self.regex_set {
            for pattern_idx in regex_set.matches(message).into_iter() {
                // Get all guidelines that match this pattern
                if let Some(guideline_indices) = self.regex_pattern_to_guidelines.get(&pattern_idx)
                {
                    for &guideline_idx in guideline_indices {
                        if let Some(guideline) = self.guidelines.get(guideline_idx) {
                            matches.push((guideline_idx, guideline));
                        }
                    }
                }
            }
        }

        matches
    }

    /// Extract parameters from message using regex
    fn extract_parameters(
        &self,
        message: &str,
        guideline: &Guideline,
    ) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();

        if let GuidelineCondition::Regex(pattern) = &guideline.condition {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(message) {
                    // Extract named and numbered captures
                    for (i, param_name) in guideline.action.parameters.iter().enumerate() {
                        if let Some(capture) = captures.get(i + 1) {
                            params.insert(
                                param_name.clone(),
                                serde_json::Value::String(capture.as_str().to_string()),
                            );
                        }
                    }
                }
            }
        }

        params
    }
}

impl Default for DefaultGuidelineMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GuidelineMatcher for DefaultGuidelineMatcher {
    async fn match_guidelines(
        &self,
        message: &str,
        _context: &Context,
    ) -> Result<Vec<GuidelineMatch>> {
        trace!(message = %message, "Starting guideline matching");
        let mut matches = Vec::new();

        // Match literal conditions
        let literal_matches = self.match_literal_conditions(message);
        debug!(
            literal_count = literal_matches.len(),
            "Literal conditions matched"
        );

        for (_idx, guideline) in literal_matches {
            trace!(
                guideline_id = %guideline.id,
                priority = guideline.priority,
                "Literal match found"
            );
            matches.push(GuidelineMatch {
                guideline_id: guideline.id,
                relevance_score: 1.0, // Exact match
                semantic_score: 0.0,
                matched_condition: format!("{:?}", guideline.condition),
                extracted_parameters: HashMap::new(),
                explanation: Some("Exact literal match".to_string()),
            });
        }

        // Match regex conditions
        let regex_matches = self.match_regex_conditions(message);
        debug!(
            regex_count = regex_matches.len(),
            "Regex conditions matched"
        );

        for (_idx, guideline) in regex_matches {
            let params = self.extract_parameters(message, guideline);
            trace!(
                guideline_id = %guideline.id,
                priority = guideline.priority,
                param_count = params.len(),
                "Regex match found"
            );
            matches.push(GuidelineMatch {
                guideline_id: guideline.id,
                relevance_score: 0.9, // Regex match
                semantic_score: 0.0,
                matched_condition: format!("{:?}", guideline.condition),
                extracted_parameters: params,
                explanation: Some("Regex pattern match".to_string()),
            });
        }

        info!(
            total_matches = matches.len(),
            message_length = message.len(),
            "Guideline matching complete"
        );

        Ok(matches)
    }

    async fn select_best_match(&self, mut matches: Vec<GuidelineMatch>) -> Option<GuidelineMatch> {
        if matches.is_empty() {
            debug!("No matches to select from");
            return None;
        }

        debug!(
            candidate_count = matches.len(),
            "Selecting best match from candidates"
        );

        // Find corresponding guidelines and sort by priority and timestamp
        matches.sort_by(|a, b| {
            let guideline_a = self.guidelines.iter().find(|g| g.id == a.guideline_id);
            let guideline_b = self.guidelines.iter().find(|g| g.id == b.guideline_id);

            match (guideline_a, guideline_b) {
                (Some(ga), Some(gb)) => {
                    // First by priority (higher is better)
                    match gb.priority.cmp(&ga.priority) {
                        std::cmp::Ordering::Equal => {
                            // Then by timestamp (newer is better)
                            gb.created_at.cmp(&ga.created_at)
                        }
                        other => other,
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        let best = matches.into_iter().next();

        if let Some(ref selected) = best {
            if let Some(guideline) = self
                .guidelines
                .iter()
                .find(|g| g.id == selected.guideline_id)
            {
                info!(
                    selected_guideline_id = %selected.guideline_id,
                    priority = guideline.priority,
                    relevance_score = selected.relevance_score,
                    "Best match selected"
                );
            }
        }

        best
    }

    async fn add_guideline(&mut self, guideline: Guideline) -> Result<GuidelineId> {
        let id = guideline.id;
        info!(
            guideline_id = %id,
            condition = ?guideline.condition,
            priority = guideline.priority,
            "Adding guideline to matcher"
        );
        self.guidelines.push(guideline);
        self.rebuild_matchers();
        debug!(
            total_guidelines = self.guidelines.len(),
            "Guideline added and matchers rebuilt"
        );
        Ok(id)
    }

    async fn remove_guideline(&mut self, id: &GuidelineId) -> Result<()> {
        self.guidelines.retain(|g| &g.id != id);
        self.rebuild_matchers();
        Ok(())
    }

    fn get_guidelines(&self) -> &[Guideline] {
        &self.guidelines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_literal_matching() {
        let mut matcher = DefaultGuidelineMatcher::new();

        let guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("pricing".to_string()),
            action: GuidelineAction {
                response_template: "Pricing info".to_string(),
                requires_llm: false,
                parameters: vec![],
            },
            priority: 10,
            tools: vec![],
            parameters: HashMap::new(),
            created_at: Utc::now(),
        };

        matcher.add_guideline(guideline).await.unwrap();

        let context = Context::new();

        let matches = matcher
            .match_guidelines("What is your pricing?", &context)
            .await
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].relevance_score, 1.0);
    }

    #[tokio::test]
    async fn test_regex_matching() {
        let mut matcher = DefaultGuidelineMatcher::new();

        let guideline = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Regex(r"cancel.*subscription".to_string()),
            action: GuidelineAction {
                response_template: "Cancel info".to_string(),
                requires_llm: false,
                parameters: vec![],
            },
            priority: 10,
            tools: vec![],
            parameters: HashMap::new(),
            created_at: Utc::now(),
        };

        matcher.add_guideline(guideline).await.unwrap();

        let context = Context::new();

        let matches = matcher
            .match_guidelines("I want to cancel my subscription", &context)
            .await
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].relevance_score, 0.9);
    }

    #[tokio::test]
    async fn test_priority_resolution() {
        let mut matcher = DefaultGuidelineMatcher::new();

        // Use different but overlapping patterns to properly test priority
        let low_priority = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("pricing".to_string()),
            action: GuidelineAction {
                response_template: "Low".to_string(),
                requires_llm: false,
                parameters: vec![],
            },
            priority: 5,
            tools: vec![],
            parameters: HashMap::new(),
            created_at: Utc::now(),
        };

        let high_priority = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("pricing".to_string()),
            action: GuidelineAction {
                response_template: "High".to_string(),
                requires_llm: false,
                parameters: vec![],
            },
            priority: 20,
            tools: vec![],
            parameters: HashMap::new(),
            created_at: Utc::now(),
        };

        let low_id = low_priority.id;
        let high_id = high_priority.id;

        matcher.add_guideline(low_priority).await.unwrap();
        matcher.add_guideline(high_priority).await.unwrap();

        let context = Context::new();

        let matches = matcher
            .match_guidelines("What about pricing?", &context)
            .await
            .unwrap();

        // Should have 2 matches (both guidelines match "pricing")
        assert_eq!(matches.len(), 2, "Should match both guidelines");

        // One should be low priority, one should be high priority
        let low_match = matches.iter().find(|m| m.guideline_id == low_id);
        let high_match = matches.iter().find(|m| m.guideline_id == high_id);
        assert!(low_match.is_some(), "Low priority guideline should match");
        assert!(high_match.is_some(), "High priority guideline should match");

        // select_best_match should pick the high priority one
        let best = matcher.select_best_match(matches).await.unwrap();
        assert_eq!(
            best.guideline_id, high_id,
            "Should select high priority guideline"
        );
    }
}
