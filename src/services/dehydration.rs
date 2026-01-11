//! 脱水服务

use async_trait::async_trait;

use crate::error::Result;
use crate::models::turn::{DehydratedData, Turn};

#[async_trait]
pub trait DehydrationService: Send + Sync {
    async fn generate_summary(&self, content: &str) -> Result<DehydratedData>;
    async fn extract_keywords(&self, content: &str) -> Result<Vec<String>>;
    async fn extract_topics(&self, content: &str) -> Result<Vec<String>>;
}

pub struct SimpleDehydrationService {
    max_gist_length: usize,
    max_topics: usize,
    max_tags: usize,
}

impl SimpleDehydrationService {
    pub fn new(max_gist_length: usize, max_topics: usize, max_tags: usize) -> Self {
        Self {
            max_gist_length,
            max_topics,
            max_tags,
        }
    }

    fn clean_text(&self, text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn extract_basic_keywords(&self, text: &str) -> Vec<String> {
        let stop_words = std::collections::HashSet::from([
            "的", "了", "是", "在", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上",
            "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这",
            "that", "the", "is", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "be", "was", "were", "been",
        ]);

        let words: Vec<&str> = text
            .split_whitespace()
            .filter(|word| {
                word.len() >= 2
                    && !stop_words.contains(&word.to_lowercase().as_str())
                    && word
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            })
            .collect();

        let mut word_freq: std::collections::HashMap<&str, u32> = std::collections::HashMap::new();
        for word in &words {
            *word_freq.entry(word).or_insert(0) += 1;
        }

        let mut keywords: Vec<_> = word_freq.into_iter().collect();
        keywords.sort_by(|a, b| b.1.cmp(&a.1));

        keywords
            .into_iter()
            .take(self.max_tags)
            .map(|(word, _)| word.to_string())
            .collect()
    }

    fn classify_topics(&self, content: &str, keywords: &[String]) -> Vec<String> {
        let content_lower = content.to_lowercase();

        let topic_patterns = vec![
            (
                "编程",
                vec![
                    "code",
                    "function",
                    "class",
                    "api",
                    "programming",
                    "开发",
                    "代码",
                ],
            ),
            (
                "AI",
                vec!["ai", "model", "llm", "gpt", "machine learning", "人工智能"],
            ),
            (
                "数据库",
                vec!["database", "sql", "query", "db", "存储", "数据库"],
            ),
            (
                "Web",
                vec!["http", "web", "server", "client", "api", "前端", "后端"],
            ),
            (
                "系统",
                vec!["system", "os", "linux", "windows", "进程", "线程"],
            ),
        ];

        let mut topics = Vec::new();
        for (topic, patterns) in topic_patterns {
            for pattern in patterns {
                if content_lower.contains(pattern) {
                    topics.push(topic.to_string());
                    break;
                }
            }
        }

        if topics.is_empty() {
            topics.extend(keywords.iter().take(self.max_topics).cloned());
        }

        topics.into_iter().take(self.max_topics).collect()
    }
}

#[async_trait]
impl DehydrationService for SimpleDehydrationService {
    async fn generate_summary(&self, content: &str) -> Result<DehydratedData> {
        let cleaned = self.clean_text(content);

        let gist = if cleaned.len() > self.max_gist_length {
            cleaned
                .chars()
                .take(self.max_gist_length)
                .collect::<String>()
                + "..."
        } else {
            cleaned.clone()
        };

        let keywords = self.extract_basic_keywords(&cleaned);
        let topics = self.classify_topics(&cleaned, &keywords);

        Ok(DehydratedData {
            gist,
            topics: topics.clone(),
            tags: keywords,
            embedding: None,
            generated_at: chrono::Utc::now(),
            generator: Some("simple-dehydration".to_string()),
        })
    }

    async fn extract_keywords(&self, content: &str) -> Result<Vec<String>> {
        let cleaned = self.clean_text(content);
        Ok(self.extract_basic_keywords(&cleaned))
    }

    async fn extract_topics(&self, content: &str) -> Result<Vec<String>> {
        let cleaned = self.clean_text(content);
        let keywords = self.extract_basic_keywords(&cleaned);
        Ok(self.classify_topics(&cleaned, &keywords))
    }
}

pub fn create_dehydration_service(
    max_gist_length: usize,
    max_topics: usize,
    max_tags: usize,
) -> Box<dyn DehydrationService> {
    Box::new(SimpleDehydrationService::new(
        max_gist_length,
        max_topics,
        max_tags,
    ))
}

pub async fn dehydrate_turn(
    service: &dyn DehydrationService,
    turn: &mut Turn,
) -> Result<DehydratedData> {
    let summary = service.generate_summary(&turn.raw_content).await?;
    turn.dehydrated = Some(summary.clone());
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_summary() {
        let service = SimpleDehydrationService::new(100, 5, 10);

        let content = "这是一个测试内容，用于测试脱水服务。Hello world programming in Rust. \
                       This is a test for AI and machine learning applications.";
        let summary = service.generate_summary(content).await.unwrap();

        assert!(!summary.gist.is_empty());
        // The gist can be up to max_gist_length + 3 (for "...") or the full content if shorter
        assert!(summary.gist.len() <= 203);
    }

    #[tokio::test]
    async fn test_extract_keywords() {
        let service = SimpleDehydrationService::new(100, 5, 10);

        let content = "Rust programming language is great for systems programming. \
                       The async programming model in Rust is excellent.";
        let keywords = service.extract_keywords(content).await.unwrap();

        assert!(!keywords.is_empty());
    }

    #[tokio::test]
    async fn test_extract_topics() {
        let service = SimpleDehydrationService::new(100, 5, 10);

        let ai_content = "This is about machine learning and AI models like GPT. \
                          Neural networks are used for natural language processing.";
        let topics = service.extract_topics(ai_content).await.unwrap();

        assert!(topics.contains(&"AI".to_string()));
    }
}
