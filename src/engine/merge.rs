// src/engine/merge.rs
use crate::error::Result;

/// Strategy for merging results from parallel execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Concatenate all results with newlines
    Concat,
    /// Vote on most common result
    Vote,
    /// Use LLM to intelligently merge results
    LlmMerge,
}

/// Merge multiple agent outputs using the specified strategy
pub async fn merge_results(
    results: Vec<String>,
    strategy: MergeStrategy,
) -> Result<String> {
    match strategy {
        MergeStrategy::Concat => concat_results(results),
        MergeStrategy::Vote => vote_results(results),
        MergeStrategy::LlmMerge => llm_merge_results(results).await,
    }
}

fn concat_results(results: Vec<String>) -> Result<String> {
    Ok(results.join("\n"))
}

fn vote_results(results: Vec<String>) -> Result<String> {
    todo!("Implement vote strategy")
}

async fn llm_merge_results(results: Vec<String>) -> Result<String> {
    todo!("Implement LLM merge strategy")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concat_strategy() {
        let results = vec![
            "Result from agent1".to_string(),
            "Result from agent2".to_string(),
            "Result from agent3".to_string(),
        ];

        let merged = merge_results(results, MergeStrategy::Concat)
            .await
            .unwrap();

        assert!(merged.contains("Result from agent1"));
        assert!(merged.contains("Result from agent2"));
        assert!(merged.contains("Result from agent3"));
        assert_eq!(merged.lines().count(), 3);
    }
}
