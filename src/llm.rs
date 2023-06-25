use anyhow::Result;
use openai::embeddings::{Embedding, Embeddings};
use shuttle_secrets::SecretStore;

use crate::contents::File;
use crate::errors::{EmbeddingError, SetupError};

pub fn setup(secrets: &SecretStore) -> Result<()> {
    let open_ai_key = secrets
        .get("OPEN_AI_KEY")
        .ok_or(SetupError("OPEN_AI_KEY not available"))?;

    openai::set_key(open_ai_key);
    return Ok(());
}

pub async fn embed_file(file: &File) -> Result<Embeddings> {
    let sentence_as_str: Vec<&str> = file.sentences.iter().map(|s| s.as_str()).collect();
    return Embeddings::create("text-embedding-ada-002", sentence_as_str, "shuttle")
        .await
        .map_err(|e| {
            println!("{:?}", e.to_string());
            EmbeddingError {}.into()
        });
}

pub async fn embed_sentence(prompt: &str) -> Result<Embedding> {
    return Embedding::create("text-embedding-ada-002", prompt, "shuttle")
        .await
        .map_err(|e| {
            println!("{:?}", e.to_string());
            EmbeddingError {}.into()
        });
}

