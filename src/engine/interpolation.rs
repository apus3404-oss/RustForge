// src/engine/interpolation.rs
use crate::engine::types::ExecutionContext;
use crate::error::{Error, Result};
use regex::Regex;
use std::sync::OnceLock;

const MAX_SUGGESTION_DISTANCE: usize = 3;
const MAX_SUGGESTIONS: usize = 3;

pub struct VariableInterpolator<'a> {
    context: &'a ExecutionContext,
}

impl<'a> VariableInterpolator<'a> {
    pub fn new(context: &'a ExecutionContext) -> Self {
        Self { context }
    }

    pub fn interpolate(&self, template: &str) -> Result<String> {
        static VARIABLE_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = VARIABLE_REGEX.get_or_init(|| {
            Regex::new(r"\{([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)*)\}").unwrap()
        });

        let mut result = template.to_string();
        let mut missing_variables = Vec::new();

        // Collect all matches first to avoid borrow issues
        let matches: Vec<_> = regex
            .captures_iter(template)
            .map(|cap| (cap.get(0).unwrap().as_str(), cap.get(1).unwrap().as_str()))
            .collect();

        for (full_match, var_path) in matches {
            match self.resolve_variable(var_path) {
                Ok(value) => {
                    result = result.replace(full_match, &value);
                }
                Err(Error::VariableNotFound { variable, suggestions }) => {
                    missing_variables.push((variable, suggestions));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // Report all missing variables together
        if !missing_variables.is_empty() {
            if missing_variables.len() == 1 {
                let (variable, suggestions) = missing_variables.into_iter().next().unwrap();
                return Err(Error::VariableNotFound { variable, suggestions });
            } else {
                // Multiple missing variables - combine into one error
                let variables: Vec<String> = missing_variables.iter()
                    .map(|(v, _)| v.clone())
                    .collect();
                let all_suggestions: Vec<String> = missing_variables.into_iter()
                    .flat_map(|(_, s)| s)
                    .collect();

                return Err(Error::VariableNotFound {
                    variable: format!("Multiple variables not found: {}", variables.join(", ")),
                    suggestions: all_suggestions,
                });
            }
        }

        Ok(result)
    }

    fn resolve_variable(&self, path: &str) -> Result<String> {
        // Try direct lookup first
        if let Some(value) = self.context.get_value(path) {
            return Ok(value_to_string(value));
        }

        // Try nested path lookup (e.g., "agent1.output.data.count")
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() > 1 {
            // Try to find the root key
            for i in (1..=parts.len()).rev() {
                let key = parts[..i].join(".");
                if let Some(value) = self.context.get_value(&key) {
                    // Navigate the remaining path in the JSON value
                    let remaining_path = &parts[i..];
                    if let Some(nested_value) = navigate_json(value, remaining_path) {
                        return Ok(value_to_string(nested_value));
                    }
                }
            }
        }

        // Variable not found, generate suggestions
        let suggestions = self.find_similar_variables(path);
        Err(Error::VariableNotFound {
            variable: path.to_string(),
            suggestions,
        })
    }

    fn find_similar_variables(&self, target: &str) -> Vec<String> {
        let mut candidates: Vec<(String, usize)> = self
            .context
            .context_store
            .keys()
            .map(|key| {
                let distance = levenshtein_distance(target, key);
                (key.clone(), distance)
            })
            .filter(|(_, distance)| *distance <= MAX_SUGGESTION_DISTANCE)
            .collect();

        candidates.sort_by_key(|(_, distance)| *distance);
        candidates.into_iter().map(|(key, _)| key).take(MAX_SUGGESTIONS).collect()
    }
}

fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

fn navigate_json<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(value);
    }

    match value {
        serde_json::Value::Object(map) => {
            let next_value = map.get(path[0])?;
            navigate_json(next_value, &path[1..])
        }
        serde_json::Value::Array(arr) => {
            let index: usize = path[0].parse().ok()?;
            let next_value = arr.get(index)?;
            navigate_json(next_value, &path[1..])
        }
        _ => None,
    }
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_interpolate_simple_variable() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value("input.name", json!("Alice"));

        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator.interpolate("Hello {input.name}!").unwrap();
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_interpolate_multiple_variables() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value("input.name", json!("Alice"));
        context.set_value("input.age", json!(30));

        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator
            .interpolate("Name: {input.name}, Age: {input.age}")
            .unwrap();
        assert_eq!(result, "Name: Alice, Age: 30");
    }

    #[test]
    fn test_interpolate_agent_output() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value("agent1.output", json!("Task completed"));

        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator
            .interpolate("Result: {agent1.output}")
            .unwrap();
        assert_eq!(result, "Result: Task completed");
    }

    #[test]
    fn test_interpolate_nested_json() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value(
            "agent1.output",
            json!({
                "status": "success",
                "data": {
                    "count": 42
                }
            }),
        );

        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator
            .interpolate("Count: {agent1.output.data.count}")
            .unwrap();
        assert_eq!(result, "Count: 42");
    }

    #[test]
    fn test_interpolate_no_variables() {
        let context = ExecutionContext::new("test-workflow".to_string());
        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator.interpolate("No variables here").unwrap();
        assert_eq!(result, "No variables here");
    }

    #[test]
    fn test_missing_variable_with_suggestions() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value("input.username", json!("alice"));
        context.set_value("input.password", json!("secret"));

        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator.interpolate("Hello {input.usernme}!");

        assert!(result.is_err());
        match result {
            Err(Error::VariableNotFound {
                variable,
                suggestions,
            }) => {
                assert_eq!(variable, "input.usernme");
                assert!(suggestions.contains(&"input.username".to_string()));
            }
            _ => panic!("Expected VariableNotFound error"),
        }
    }

    #[test]
    fn test_missing_variable_no_suggestions() {
        let context = ExecutionContext::new("test-workflow".to_string());
        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator.interpolate("Hello {nonexistent.var}!");

        assert!(result.is_err());
        match result {
            Err(Error::VariableNotFound { variable, .. }) => {
                assert_eq!(variable, "nonexistent.var");
            }
            _ => panic!("Expected VariableNotFound error"),
        }
    }

    #[test]
    fn test_interpolate_with_special_characters() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value("input.message", json!("Hello, World!"));

        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator
            .interpolate("Message: {input.message} - Done.")
            .unwrap();
        assert_eq!(result, "Message: Hello, World! - Done.");
    }

    #[test]
    fn test_interpolate_boolean_and_null() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value("input.enabled", json!(true));
        context.set_value("input.disabled", json!(false));
        context.set_value("input.empty", json!(null));

        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator
            .interpolate("Enabled: {input.enabled}, Disabled: {input.disabled}, Empty: {input.empty}")
            .unwrap();
        assert_eq!(result, "Enabled: true, Disabled: false, Empty: null");
    }

    #[test]
    fn test_interpolate_array_indexing() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value(
            "agent1.output",
            json!({
                "items": ["first", "second", "third"],
                "nested": {
                    "list": [10, 20, 30]
                }
            }),
        );

        let interpolator = VariableInterpolator::new(&context);

        // Test array indexing
        let result = interpolator
            .interpolate("Item: {agent1.output.items.0}")
            .unwrap();
        assert_eq!(result, "Item: first");

        // Test nested array indexing
        let result = interpolator
            .interpolate("Value: {agent1.output.nested.list.1}")
            .unwrap();
        assert_eq!(result, "Value: 20");
    }

    #[test]
    fn test_interpolate_multiple_missing_variables() {
        let mut context = ExecutionContext::new("test-workflow".to_string());
        context.set_value("input.name", json!("Alice"));

        let interpolator = VariableInterpolator::new(&context);
        let result = interpolator.interpolate("Hello {input.name}, your age is {input.age} and city is {input.city}");

        assert!(result.is_err());
        match result {
            Err(Error::VariableNotFound { variable, .. }) => {
                assert!(variable.contains("Multiple variables not found"));
                assert!(variable.contains("input.age"));
                assert!(variable.contains("input.city"));
            }
            _ => panic!("Expected VariableNotFound error with multiple variables"),
        }
    }
}
