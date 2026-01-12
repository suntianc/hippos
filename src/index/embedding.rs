//! 嵌入模型服务

use async_trait::async_trait;
use reqwest;
use serde::Deserialize;

use crate::config::config::EmbeddingConfig;
use crate::error::Result;

#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    async fn encode(&self, text: &str) -> Result<Vec<f32>>;
    async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}

pub struct SimpleEmbeddingModel {
    embeddings: std::collections::HashMap<String, Vec<f32>>,
    dimension: usize,
}

impl SimpleEmbeddingModel {
    pub fn new(dimension: usize) -> Self {
        Self {
            embeddings: std::collections::HashMap::new(),
            dimension,
        }
    }

    pub fn add_word_embedding(&mut self, word: &str, embedding: &[f32]) {
        if embedding.len() == self.dimension {
            self.embeddings.insert(word.to_string(), embedding.to_vec());
        }
    }
}

#[async_trait]
impl EmbeddingModel for SimpleEmbeddingModel {
    async fn encode(&self, text: &str) -> Result<Vec<f32>> {
        let words: Vec<&str> = text.split_whitespace().collect();

        if words.is_empty() {
            return Ok(vec![0.0; self.dimension]);
        }

        let mut sum = vec![0.0; self.dimension];
        let mut count = 0;

        for word in words {
            if let Some(embedding) = self.embeddings.get(word) {
                for (i, val) in embedding.iter().enumerate() {
                    sum[i] += val;
                }
                count += 1;
            }
        }

        if count > 0 {
            for val in &mut sum {
                *val /= count as f32;
            }
        }

        Ok(sum)
    }

    async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::with_capacity(texts.len());

        for text in texts {
            let embedding = self.encode(text).await?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Ollama Embedding 模型客户端
pub struct OllamaEmbeddingModel {
    client: reqwest::Client,
    model_name: String,
    base_url: String,
    dimension: usize,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

impl OllamaEmbeddingModel {
    pub fn new(base_url: &str, model_name: &str, dimension: usize) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            model_name: model_name.to_string(),
            base_url: base_url.to_string(),
            dimension,
        })
    }

    async fn embed(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let response = self
            .client
            .post(format!("{}/api/embed", self.base_url))
            .json(&serde_json::json!({
                "model": self.model_name,
                "input": texts,
                "truncate": true
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::AppError::Embedding(format!(
                "Ollama embedding failed: {}",
                error_text
            )));
        }

        let embed_response: OllamaEmbedResponse = response.json().await?;
        Ok(embed_response.embeddings)
    }
}

#[async_trait]
impl EmbeddingModel for OllamaEmbeddingModel {
    async fn encode(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed(vec![text]).await?;
        Ok(embeddings
            .into_iter()
            .next()
            .unwrap_or_else(|| vec![0.0; self.dimension]))
    }

    async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Ollama 支持批量输入，但为了稳定性，分批处理
        let batch_size = 32;
        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(batch_size) {
            let chunk_vec: Vec<&str> = chunk.to_vec();
            let embeddings = self.embed(chunk_vec).await?;
            all_embeddings.extend(embeddings);
        }

        Ok(all_embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

pub async fn create_embedding_model(
    config: &EmbeddingConfig,
    dimension: usize,
) -> Result<Box<dyn EmbeddingModel>> {
    match config.backend.as_str() {
        "ollama" => {
            let model =
                OllamaEmbeddingModel::new(&config.ollama_url, &config.model_name, dimension)?;
            Ok(Box::new(model))
        }
        "simple" | _ => {
            let model = SimpleEmbeddingModel::new(dimension);
            Ok(Box::new(model))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_embedding_model() {
        let model = SimpleEmbeddingModel::new(384);
        let model: Box<dyn EmbeddingModel> = Box::new(model);

        let result = model.encode("hello world").await.unwrap();
        assert_eq!(result.len(), 384);
        assert_eq!(model.dimension(), 384);
    }

    #[tokio::test]
    async fn test_batch_encoding() {
        let model = SimpleEmbeddingModel::new(384);
        let model: Box<dyn EmbeddingModel> = Box::new(model);

        let texts = vec!["hello", "world", "test"];
        let results = model.encode_batch(&texts).await.unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].len(), 384);
        assert_eq!(results[1].len(), 384);
        assert_eq!(results[2].len(), 384);
    }
}
