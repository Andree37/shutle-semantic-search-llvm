use std::path::PathBuf;
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use tower_http::services::ServeDir;

use crate::contents::File;
use crate::vector::VectorDB;

mod contents;
mod errors;
mod vector;
mod llm;


struct AppState {
    vector_db: VectorDB,
    files: Vec<File>,
}

#[derive(serde::Deserialize)]
struct Prompt {
    prompt: String,
}

async fn embed_documentation(vector_db: &mut VectorDB, files: &Vec<File>) -> anyhow::Result<()> {
    for file in files {
        let embeddings = llm::embed_file(&file).await?;
        println!("Embedding: {:?}", file.path);
        for embedding in embeddings.data {
            vector_db.upsert_embedding(embedding, file).await?;
        }
    }

    return Ok(());
}

async fn prompt(State(app_state): State<Arc<AppState>>, Json(prompt): Json<Prompt>) -> impl
IntoResponse {
    let prompt = prompt.prompt;
    let embedding = match llm::embed_sentence(&prompt).await {
        Ok(embedding) => embedding,
        Err(_) => return "No embedding possible",
    };
    let scored_point = match app_state.vector_db.search(embedding).await {
        Ok(scored_point) => scored_point,
        Err(_) => return "No search possible",
    };
    scored_point.payload.print();
    return "Found";
}


#[shuttle_runtime::main]
async fn axum(
    #[shuttle_static_folder::StaticFolder(folder = "static")] assets: PathBuf,
    #[shuttle_static_folder::StaticFolder(folder = "docs")] docs_folder: PathBuf,
    #[shuttle_static_folder::StaticFolder(folder = ".")] prefix: PathBuf,
    #[shuttle_secrets::Secrets] secrets: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let embed = false;

    let files = contents::load_files_from_dir(docs_folder, &prefix, "mdx")?;
    let mut vector_db = vector::VectorDB::new(&secrets)?;
    llm::setup(&secrets)?;

    println!("Setup done!");

    // We don't need to embed every time, so we can skip this step
    if embed {
        vector_db.reset_collection().await?;
        embed_documentation(&mut vector_db, &files).await?;
    }

    println!("Embeddings done!");

    let app_state = AppState {
        vector_db,
        files,
    };
    let app_state = Arc::new(app_state);

    let router = Router::new().route("/prompt", post(prompt))
        .nest_service("/", ServeDir::new(assets))
        .with_state(app_state);
    Ok(router.into())
}
