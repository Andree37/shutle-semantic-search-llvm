use std::{path::PathBuf, sync::Arc};

use axum::{extract::Json, extract::State, response::IntoResponse, Router, routing::post};
use axum_macros::debug_handler;
use futures::Stream;
use openai::chat::ChatCompletionDelta;
use serde::Deserialize;
use tokio::sync::mpsc::Receiver;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::services::ServeDir;

use contents::File;
use errors::PromptError;
use finder::Finder;
use vector::VectorDB;

mod contents;
mod errors;
mod finder;
mod llm;
mod vector;

async fn embed_documentation(vector_db: &mut VectorDB, files: &Vec<File>) -> anyhow::Result<()> {
    for file in files {
        let embeddings = match llm::embed_file(file).await {
            Ok(e) => e,
            Err(_) => { continue; }
        };
        println!("Embedding: {:?}", file.path);
        for embedding in embeddings.data {
            vector_db.upsert_embedding(embedding, file).await?;
        }
    }

    return Ok(());
}

#[derive(Deserialize)]
struct Prompt {
    prompt: String,
}

fn chat_completion_stream(
    chat_completion: Receiver<ChatCompletionDelta>,
) -> impl Stream<Item=String> {
    return ReceiverStream::new(chat_completion)
        .map(|completion| completion.choices)
        .map(|choices| {
            choices
                .into_iter()
                .map(|choice| choice.delta.content.unwrap_or("\n".to_string()))
                .collect()
        });
}

fn error_stream() -> impl Stream<Item=String> {
    futures::stream::once(async move { "Error with your prompt".to_string() })
}

async fn get_contents(
    prompt: &str,
    app_state: &AppState,
) -> anyhow::Result<Receiver<ChatCompletionDelta>> {
    let embedding = llm::embed_sentence(prompt).await?;
    let result = app_state.vector_db.search(embedding).await?;
    println!("Result: {:?}", result);
    let contents = app_state
        .files
        .get_contents(&result)
        .ok_or(PromptError {})?;
    return llm::chat_stream(prompt, contents.as_str()).await;
}

#[debug_handler]
async fn prompt(
    State(app_state): State<Arc<AppState>>,
    Json(prompt): Json<Prompt>,
) -> impl IntoResponse {
    let prompt = prompt.prompt;
    let chat_completion = get_contents(&prompt, &app_state).await;

    if let Ok(chat_completion) = chat_completion {
        return axum_streams::StreamBodyAs::text(chat_completion_stream(chat_completion));
    }

    return axum_streams::StreamBodyAs::text(error_stream());
}

struct AppState {
    files: Vec<File>,
    vector_db: VectorDB,
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_static_folder::StaticFolder(folder = "static")] static_folder: PathBuf,
    #[shuttle_static_folder::StaticFolder(folder = "docs-akash")] docs_folder: PathBuf,
    #[shuttle_static_folder::StaticFolder(folder = ".")] prefix: PathBuf,
    #[shuttle_secrets::Secrets] secrets: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let embedding = false;
    llm::setup(&secrets)?;
    let mut vector_db = VectorDB::new(&secrets)?;
    let files = contents::load_files_from_dir(docs_folder, ".mdx", &prefix)?;

    println!("Setup done");

    if embedding {
        vector_db.reset_collection().await?;
        embed_documentation(&mut vector_db, &files).await?;
        println!("Embedding done");
    }

    let app_state = AppState { files, vector_db };
    let app_state = Arc::new(app_state);

    let router = Router::new()
        .route("/prompt", post(prompt))
        .nest_service("/", ServeDir::new(static_folder))
        .with_state(app_state);
    return Ok(router.into());
}