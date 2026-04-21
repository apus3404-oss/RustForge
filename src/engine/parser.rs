// src/engine/parser.rs
use crate::engine::types::WorkflowDefinition;
use crate::error::{Error, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct WorkflowParser;

impl WorkflowParser {
    /// Parse a workflow from a YAML file
    pub fn parse_file(path: &Path) -> Result<WorkflowDefinition> {
        let content = fs::read_to_string(path)?;
        Self::parse_str(&content)
    }

    /// Parse a workflow from a YAML string
    pub fn parse_str(yaml: &str) -> Result<WorkflowDefinition> {
        let workflow: WorkflowDefinition = serde_yaml::from_str(yaml)?;
        Self::validate(&workflow)?;
        Ok(workflow)
    }

    /// Validate workflow definition
    fn validate(workflow: &WorkflowDefinition) -> Result<()> {
        // Check for duplicate agent IDs
        Self::validate_unique_agent_ids(workflow)?;

        // Check for valid dependencies
        Self::validate_dependencies(workflow)?;

        Ok(())
    }

    /// Ensure all agent IDs are unique
    fn validate_unique_agent_ids(workflow: &WorkflowDefinition) -> Result<()> {
        let mut seen_ids = HashSet::new();
        let mut duplicates = Vec::new();

        for agent in &workflow.agents {
            if !seen_ids.insert(&agent.id) {
                duplicates.push(agent.id.clone());
            }
        }

        if !duplicates.is_empty() {
            return Err(Error::InvalidWorkflowDefinition {
                reason: format!("Duplicate agent IDs found: {:?}", duplicates),
            });
        }

        Ok(())
    }

    /// Ensure all dependencies reference valid agent IDs
    fn validate_dependencies(workflow: &WorkflowDefinition) -> Result<()> {
        let valid_ids: HashSet<&String> = workflow.agents.iter().map(|a| &a.id).collect();

        for agent in &workflow.agents {
            for dep in &agent.depends_on {
                if !valid_ids.contains(dep) {
                    return Err(Error::InvalidWorkflowDefinition {
                        reason: format!(
                            "Agent '{}' depends on non-existent agent '{}'",
                            agent.id, dep
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("workflows")
            .join(name)
    }

    #[test]
    fn test_parse_simple_workflow() {
        let yaml = r#"
name: "Simple Test"
mode: sequential
agents:
  - id: agent1
    type: TestAgent
    task: "Do something"
"#;
        let workflow = WorkflowParser::parse_str(yaml).unwrap();
        assert_eq!(workflow.name, "Simple Test");
        assert_eq!(workflow.agents.len(), 1);
        assert_eq!(workflow.agents[0].id, "agent1");
    }

    #[test]
    fn test_parse_workflow_from_file() {
        let path = fixture_path("simple.yaml");
        let workflow = WorkflowParser::parse_file(&path).unwrap();
        assert_eq!(workflow.name, "Simple Test");
        assert_eq!(workflow.agents.len(), 1);
    }

    #[test]
    fn test_parse_workflow_with_dependencies() {
        let path = fixture_path("with-dependencies.yaml");
        let workflow = WorkflowParser::parse_file(&path).unwrap();
        assert_eq!(workflow.agents.len(), 3);
        assert_eq!(workflow.agents[1].depends_on, vec!["agent1"]);
        assert_eq!(workflow.agents[2].depends_on, vec!["agent1", "agent2"]);
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = "invalid: yaml: content: [";
        let result = WorkflowParser::parse_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_duplicate_agent_ids() {
        let path = fixture_path("invalid-duplicate-ids.yaml");
        let result = WorkflowParser::parse_file(&path);
        assert!(result.is_err());

        if let Err(Error::InvalidWorkflowDefinition { reason }) = result {
            assert!(reason.contains("Duplicate agent IDs"));
            assert!(reason.contains("agent1"));
        } else {
            panic!("Expected InvalidWorkflowDefinition error");
        }
    }

    #[test]
    fn test_validate_invalid_dependency() {
        let path = fixture_path("invalid-dependency.yaml");
        let result = WorkflowParser::parse_file(&path);
        assert!(result.is_err());

        if let Err(Error::InvalidWorkflowDefinition { reason }) = result {
            assert!(reason.contains("depends on non-existent agent"));
            assert!(reason.contains("nonexistent_agent"));
        } else {
            panic!("Expected InvalidWorkflowDefinition error");
        }
    }

    #[test]
    fn test_parse_missing_file() {
        let path = PathBuf::from("nonexistent.yaml");
        let result = WorkflowParser::parse_file(&path);
        assert!(result.is_err());
    }
}
