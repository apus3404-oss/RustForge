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
    use std::collections::HashMap;

    if results.is_empty() {
        return Err(crate::error::Error::Internal(
            "Cannot vote on empty results".to_string(),
        ));
    }

    // Count occurrences of each result
    let mut counts: HashMap<String, usize> = HashMap::new();
    for result in &results {
        *counts.entry(result.clone()).or_insert(0) += 1;
    }

    // Find the result with maximum count
    let winner = counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(result, _)| result.clone())
        .unwrap(); // Safe because we checked results is not empty

    Ok(winner)
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

    #[tokio::test]
    async fn test_vote_strategy_with_clear_winner() {
        let results = vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option A".to_string(),
            "Option A".to_string(),
            "Option B".to_string(),
        ];

        let merged = merge_results(results, MergeStrategy::Vote)
            .await
            .unwrap();

        assert_eq!(merged, "Option A");
    }

    #[tokio::test]
    async fn test_vote_strategy_with_tie() {
        let results = vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option A".to_string(),
            "Option B".to_string(),
        ];

        let merged = merge_results(results, MergeStrategy::Vote)
            .await
            .unwrap();

        // With tie, return first most common option
        assert!(merged == "Option A" || merged == "Option B");
    }

    #[tokio::test]
    async fn test_vote_strategy_single_result() {
        let results = vec!["Only result".to_string()];

        let merged = merge_results(results, MergeStrategy::Vote)
            .await
            .unwrap();

        assert_eq!(merged, "Only result");
    }
}
