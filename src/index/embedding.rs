//! 嵌入模型服务

use async_trait::async_trait;

use crate::error::Result;

#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    async fn encode(&self, text: &str) -> Result<Vec<f32>>;
    async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
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
}

pub async fn create_embedding_model(
    _model_name: &str,
    _dimension: usize,
) -> Result<Box<dyn EmbeddingModel>> {
    let model = SimpleEmbeddingModel::new(_dimension);
    Ok(Box::new(model))
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
