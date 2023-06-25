use anyhow::Result;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
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

pub async fn chat(prompt: &str, contents: &str) -> Result<ChatCompletion> {
    let question = format!("{}\nContext: {}\nBe concise.", prompt, contents);

    // message is where we would append the context so we can have a proper converstaion
    return ChatCompletion::builder("gpt-3.5-turbo", vec![
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(question),
            name: Some("shuttle".to_string()),
            function_call: None,
        }
    ])
        .temperature(0.0)
        .user("shuttle")
        .create()
        .await
        .map_err(|_| { EmbeddingError {}.into() });
}

