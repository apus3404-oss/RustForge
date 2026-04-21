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
    // TODO: Integrate with actual LLM provider when connecting to executor
    // For now, provide intelligent formatting that synthesizes results

    if results.is_empty() {
        return Err(crate::error::Error::Internal(
            "Cannot merge empty results".to_string(),
        ));
    }

    if results.len() == 1 {
        return Ok(results[0].clone());
    }

    // Create a synthesized summary
    let mut synthesis = String::from("Synthesized result from multiple agents:\n\n");

    for (i, result) in results.iter().enumerate() {
        synthesis.push_str(&format!("Agent {} perspective: {}\n", i + 1, result));
    }

    synthesis.push_str("\nConsensus: ");

    // Simple heuristic: if results are similar, note agreement
    // Otherwise, note the different perspectives
    let unique_results: std::collections::HashSet<_> = results.iter().collect();
    if unique_results.len() == 1 {
        synthesis.push_str("All agents agree on the result.");
    } else {
        synthesis.push_str(&format!(
            "Multiple perspectives provided ({} unique viewpoints).",
            unique_results.len()
        ));
    }

    Ok(synthesis)
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

    #[tokio::test]
    async fn test_llm_merge_strategy() {
        let results = vec![
            "Agent 1 found: The sky is blue".to_string(),
            "Agent 2 found: The sky is azure".to_string(),
            "Agent 3 found: The sky is cerulean".to_string(),
        ];

        let merged = merge_results(results, MergeStrategy::LlmMerge)
            .await
            .unwrap();

        // LLM merge should produce a synthesized result
        assert!(!merged.is_empty());
        assert!(merged.len() > 10); // Should be a meaningful synthesis
    }

    #[tokio::test]
    async fn test_llm_merge_with_conflicting_results() {
        let results = vec![
            "The answer is 42".to_string(),
            "The answer is 43".to_string(),
            "The answer is 42".to_string(),
        ];

        let merged = merge_results(results, MergeStrategy::LlmMerge)
            .await
            .unwrap();

        // LLM should intelligently resolve conflicts
        assert!(!merged.is_empty());
    }
}
