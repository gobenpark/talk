// Guideline matching engine for behavioral rules
//
// This module implements the guideline matching system that determines
// which guideline should be activated based on user input.

use crate::error::Result;
use crate::types::{GuidelineId, ToolId};
use crate::context::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use regex::{Regex, RegexSet};

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
    /// Semantic similarity match
    Semantic {
        description: String,
        threshold: f32,
    },
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
    async fn select_best_match(
        &self,
        matches: Vec<GuidelineMatch>,
    ) -> Option<GuidelineMatch>;

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
    regex_set: Option<RegexSet>,
    individual_regexes: Vec<Regex>,
}

impl DefaultGuidelineMatcher {
    pub fn new() -> Self {
        Self {
            guidelines: Vec::new(),
            aho_corasick: None,
            regex_set: None,
            individual_regexes: Vec::new(),
        }
    }

    /// Rebuild pattern matchers after guidelines change
    fn rebuild_matchers(&mut self) {
        // Build Aho-Corasick automaton for literal conditions
        let literals: Vec<String> = self
            .guidelines
            .iter()
            .filter_map(|g| match &g.condition {
                GuidelineCondition::Literal(s) => Some(s.to_lowercase()),
                _ => None,
            })
            .collect();

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
        let patterns: Vec<String> = self
            .guidelines
            .iter()
            .filter_map(|g| match &g.condition {
                GuidelineCondition::Regex(r) => Some(r.clone()),
                _ => None,
            })
            .collect();

        if !patterns.is_empty() {
            // Build individual regexes for extraction
            self.individual_regexes = patterns
                .iter()
                .filter_map(|p| Regex::new(p).ok())
                .collect();

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
                // Map match index to guideline index
                let mut literal_idx = 0;
                for (guideline_idx, guideline) in self.guidelines.iter().enumerate() {
                    if let GuidelineCondition::Literal(_) = &guideline.condition {
                        if literal_idx == mat.pattern().as_usize() {
                            matches.push((guideline_idx, guideline));
                            break;
                        }
                        literal_idx += 1;
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
            for regex_idx in regex_set.matches(message).into_iter() {
                // Map regex index to guideline index
                let mut current_regex_idx = 0;
                for (guideline_idx, guideline) in self.guidelines.iter().enumerate() {
                    if let GuidelineCondition::Regex(_) = &guideline.condition {
                        if current_regex_idx == regex_idx {
                            matches.push((guideline_idx, guideline));
                            break;
                        }
                        current_regex_idx += 1;
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
        let mut matches = Vec::new();

        // Match literal conditions
        for (_idx, guideline) in self.match_literal_conditions(message) {
            matches.push(GuidelineMatch {
                guideline_id: guideline.id,
                relevance_score: 1.0, // Exact match
                matched_condition: format!("{:?}", guideline.condition),
                extracted_parameters: HashMap::new(),
                explanation: Some("Exact literal match".to_string()),
            });
        }

        // Match regex conditions
        for (_idx, guideline) in self.match_regex_conditions(message) {
            let params = self.extract_parameters(message, guideline);
            matches.push(GuidelineMatch {
                guideline_id: guideline.id,
                relevance_score: 0.9, // Regex match
                matched_condition: format!("{:?}", guideline.condition),
                extracted_parameters: params,
                explanation: Some("Regex pattern match".to_string()),
            });
        }

        // TODO: Semantic matching (requires embeddings)

        Ok(matches)
    }

    async fn select_best_match(
        &self,
        mut matches: Vec<GuidelineMatch>,
    ) -> Option<GuidelineMatch> {
        if matches.is_empty() {
            return None;
        }

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

        matches.into_iter().next()
    }

    async fn add_guideline(&mut self, guideline: Guideline) -> Result<GuidelineId> {
        let id = guideline.id;
        self.guidelines.push(guideline);
        self.rebuild_matchers();
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

        let context = Context {
            session_id: crate::types::SessionId::new(),
            messages: vec![],
            variables: HashMap::new(),
            journey_state: None,
            metadata: HashMap::new(),
        };

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

        let context = Context {
            session_id: crate::types::SessionId::new(),
            messages: vec![],
            variables: HashMap::new(),
            journey_state: None,
            metadata: HashMap::new(),
        };

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

        let low_priority = Guideline {
            id: GuidelineId::new(),
            condition: GuidelineCondition::Literal("test".to_string()),
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
            condition: GuidelineCondition::Literal("test".to_string()),
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

        let high_id = high_priority.id;

        matcher.add_guideline(low_priority).await.unwrap();
        matcher.add_guideline(high_priority).await.unwrap();

        let context = Context {
            session_id: crate::types::SessionId::new(),
            messages: vec![],
            variables: HashMap::new(),
            journey_state: None,
            metadata: HashMap::new(),
        };

        let matches = matcher.match_guidelines("test", &context).await.unwrap();
        let best = matcher.select_best_match(matches).await.unwrap();

        assert_eq!(best.guideline_id, high_id);
    }
}
