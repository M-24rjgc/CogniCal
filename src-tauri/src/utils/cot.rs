use crate::error::AppResult;
use tracing::debug;

/// Chain-of-Thought summarizer for generating explanations
pub struct CoTSummarizer {
    // In a real implementation, this might include API clients for LLM services
    // For now, we'll implement a simple rule-based approach
}

impl CoTSummarizer {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate a summary explanation using Chain-of-Thought reasoning
    pub async fn summarize(&self, prompt: &str) -> AppResult<String> {
        debug!(target: "app::cot", "Generating CoT summary for prompt: {}", prompt);

        // For now, implement a simple rule-based summarizer
        // In a full implementation, this would call DeepSeek API or another LLM
        let summary = self.generate_rule_based_summary(prompt)?;

        debug!(target: "app::cot", "Generated summary: {}", summary);
        Ok(summary)
    }

    /// Synchronous summary helper for scenarios where async execution isn't available
    pub fn summarize_sync(&self, prompt: &str) -> AppResult<String> {
        self.generate_rule_based_summary(prompt)
    }

    /// Generate a rule-based summary when LLM is not available
    fn generate_rule_based_summary(&self, prompt: &str) -> AppResult<String> {
        // Simple keyword-based summarization
        let mut insights = Vec::new();

        if prompt.contains("completion") && prompt.contains("80") {
            insights.push("Strong task completion performance");
        } else if prompt.contains("completion") && prompt.contains("50") {
            insights.push("Task completion needs improvement");
        } else if prompt.contains("completion") && prompt.contains("85") {
            insights.push("Task completion remains a major strength");
        }

        if prompt.contains("completion")
            && !insights.iter().any(|entry| entry.contains("completion"))
        {
            insights.push("Task completion metrics detected; continue emphasizing follow-through");
        }

        if prompt.contains("focus") && prompt.contains("75") {
            insights.push("Good focus consistency");
        } else if prompt.contains("focus") && prompt.contains("50") {
            insights.push("Focus consistency could be better");
        } else if prompt.contains("focus") && prompt.contains("40") {
            insights.push("Focus consistency needs significant improvement");
        }

        if prompt.contains("focus") && !insights.iter().any(|entry| entry.contains("focus")) {
            insights.push("Focus metrics noted; consider strategies to maintain attention");
        }

        if prompt.contains("balance") && prompt.contains("70") {
            insights.push("Healthy work-life balance");
        } else if prompt.contains("balance") && prompt.contains("40") {
            insights.push("Work-life balance needs attention");
        }

        if prompt.contains("efficiency") && prompt.contains("85") {
            insights.push("High efficiency in task execution");
        } else if prompt.contains("efficiency") && prompt.contains("60") {
            insights.push("Efficiency can be improved");
        }

        // Look for composite score
        if let Some(start) = prompt.find("compositeScore") {
            if let Some(score_start) = prompt[start..].find(":") {
                let score_part = &prompt[start + score_start + 1..];
                let score_segment = score_part
                    .split(|c| c == ',' || c == '\n')
                    .next()
                    .unwrap_or(score_part)
                    .trim();

                if let Ok(score) = score_segment.parse::<f64>() {
                    if score >= 80.0 {
                        insights.push("Excellent overall productivity score");
                    } else if score >= 60.0 {
                        insights.push("Good productivity with room for growth");
                    } else {
                        insights.push("Focus on key areas to improve productivity");
                    }
                }
            }
        }

        if insights.is_empty() {
            insights.push(
                "Productivity analysis completed. Continue monitoring trends for deeper insights.",
            );
        }

        Ok(insights.join(". "))
    }

    /// Generate a more detailed CoT explanation with step-by-step reasoning
    pub async fn generate_detailed_explanation(
        &self,
        context: &serde_json::Value,
    ) -> AppResult<String> {
        let mut reasoning_steps = Vec::new();

        // Step 1: Analyze task completion
        if let Some(completion) = context
            .get("dimensions")
            .and_then(|d| d.get("completionRate"))
        {
            if let Some(rate) = completion.as_f64() {
                if rate >= 80.0 {
                    reasoning_steps.push("Step 1: Task completion rate is excellent (≥80%), indicating strong follow-through on commitments.");
                } else if rate >= 60.0 {
                    reasoning_steps.push("Step 1: Task completion rate is good (60-79%), showing consistent progress with room for improvement.");
                } else {
                    reasoning_steps.push("Step 1: Task completion rate needs attention (<60%), consider breaking down larger tasks or reviewing priorities.");
                }
            }
        }

        // Step 2: Evaluate timeliness
        if let Some(on_time) = context.get("dimensions").and_then(|d| d.get("onTimeRatio")) {
            if let Some(ratio) = on_time.as_f64() {
                if ratio >= 75.0 {
                    reasoning_steps.push("Step 2: On-time completion is strong (≥75%), demonstrating good time management and planning.");
                } else if ratio >= 50.0 {
                    reasoning_steps.push("Step 2: On-time completion is moderate (50-74%), review time estimates and deadline setting.");
                } else {
                    reasoning_steps.push("Step 2: On-time completion needs improvement (<50%), focus on more realistic time planning.");
                }
            }
        }

        // Step 3: Assess focus consistency
        if let Some(focus) = context
            .get("dimensions")
            .and_then(|d| d.get("focusConsistency"))
        {
            if let Some(consistency) = focus.as_f64() {
                if consistency >= 70.0 {
                    reasoning_steps.push("Step 3: Focus consistency is good (≥70%), maintaining steady attention during work sessions.");
                } else if consistency >= 50.0 {
                    reasoning_steps.push("Step 3: Focus consistency is moderate (50-69%), consider minimizing distractions and taking regular breaks.");
                } else {
                    reasoning_steps.push("Step 3: Focus consistency needs work (<50%), evaluate work environment and session length.");
                }
            }
        }

        // Step 4: Check work-life balance
        if let Some(balance) = context.get("dimensions").and_then(|d| d.get("restBalance")) {
            if let Some(ratio) = balance.as_f64() {
                if ratio >= 60.0 {
                    reasoning_steps.push("Step 4: Work-life balance is healthy (≥60%), maintaining sustainable work habits.");
                } else if ratio >= 40.0 {
                    reasoning_steps.push("Step 4: Work-life balance is moderate (40-59%), ensure adequate rest and personal time.");
                } else {
                    reasoning_steps.push("Step 4: Work-life balance needs attention (<40%), prioritize rest to prevent burnout.");
                }
            }
        }

        // Step 5: Overall assessment
        if let Some(composite) = context.get("compositeScore") {
            if let Some(score) = composite.as_f64() {
                if score >= 80.0 {
                    reasoning_steps.push("Conclusion: Overall productivity is excellent (≥80), demonstrating strong performance across all dimensions. Keep up the great work!");
                } else if score >= 60.0 {
                    reasoning_steps.push("Conclusion: Overall productivity is good (60-79), with solid foundations and specific areas for enhancement. Focus on the lower-scoring dimensions for improvement.");
                } else {
                    reasoning_steps.push("Conclusion: Overall productivity needs development (<60), but this provides a clear baseline for improvement. Address the identified areas systematically.");
                }
            }
        }

        Ok(reasoning_steps.join(" "))
    }
}

impl Default for CoTSummarizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_rule_based_summarization() {
        let cot = CoTSummarizer::new();

        let prompt = "Task completion is 85%, focus consistency is 40%, compositeScore: 75";
        let summary = cot.summarize(prompt).await.unwrap();

        assert!(summary.contains("completion"));
        assert!(summary.contains("focus"));
        assert!(summary.contains("productivity"));
    }

    #[tokio::test]
    async fn test_detailed_explanation() {
        let cot = CoTSummarizer::new();

        let context = json!({
            "dimensions": {
                "completionRate": 85.0,
                "onTimeRatio": 70.0,
                "focusConsistency": 60.0,
                "restBalance": 50.0,
                "efficiencyRating": 80.0
            },
            "compositeScore": 75.0
        });

        let explanation = cot.generate_detailed_explanation(&context).await.unwrap();

        assert!(explanation.contains("Step 1"));
        assert!(explanation.contains("Step 2"));
        assert!(explanation.contains("Conclusion"));
    }

    #[test]
    fn test_rule_based_summarization_sync() {
        let cot = CoTSummarizer::new();

        let prompt = "Task completion is 85%, focus consistency is 40%, compositeScore: 75";
        let summary = cot.summarize_sync(prompt).unwrap();

        assert!(summary.contains("completion"));
        assert!(summary.contains("focus"));
        assert!(summary.contains("productivity"));
    }
}
