use anyhow::Result;
use openai::{
    chat::{ChatCompletion, ChatCompletionBuilder, ChatCompletionDelta, ChatCompletionMessage},
    embeddings::{Embedding, Embeddings},
};
use shuttle_secrets::SecretStore;
use tokio::sync::mpsc::Receiver;

use crate::{contents::File, errors::EmbeddingError, errors::SetupError};

type Conversation = Receiver<ChatCompletionDelta>;

pub fn setup(secrets: &SecretStore) -> Result<()> {
    let openai_key = secrets
        .get("OPEN_AI_KEY")
        .ok_or(SetupError("OPENAI Key not available"))?;
    openai::set_key(openai_key);
    return Ok(());
}

pub async fn embed_file(file: &File) -> Result<Embeddings> {
    let sentence_as_str: Vec<&str> = file.sentences.iter().map(|s| s.as_str()).collect();
    return Embeddings::create("text-embedding-ada-002", sentence_as_str, "shuttle")
        .await
        .map_err(|e| EmbeddingError {}.into());
}

pub async fn embed_sentence(prompt: &str) -> Result<Embedding> {
    return Embedding::create("text-embedding-ada-002", prompt, "shuttle")
        .await
        .map_err(|_| EmbeddingError {}.into());
}

pub async fn chat_stream(prompt: &str, contents: &str) -> Result<Conversation> {
    let content = format!("{}\n Context: {}\n Be concise", prompt, contents);

    return ChatCompletionBuilder::default()
        .model("gpt-3.5-turbo")
        .temperature(0.0)
        .user("shuttle")
        .messages(vec![ChatCompletionMessage {
            role: openai::chat::ChatCompletionMessageRole::User,
            content: Some(content),
            name: Some("shuttle".to_string()),
            function_call: None,
        }])
        .create_stream()
        .await
        .map_err(|_| EmbeddingError {}.into());
}

pub async fn _chat(prompt: &str, contents: &str) -> Result<ChatCompletion> {
    let content = format!("{}\n Context: {}\n Be concise", prompt, contents);

    return ChatCompletionBuilder::default()
        .model("gpt-3.5-turbo")
        .temperature(0.0)
        .user("shuttle")
        .messages(vec![ChatCompletionMessage {
            role: openai::chat::ChatCompletionMessageRole::User,
            content: Some(content),
            name: Some("shuttle".to_string()),
            function_call: None,
        }])
        .create()
        .await
        .map_err(|_| EmbeddingError {}.into());
}