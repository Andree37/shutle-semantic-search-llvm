use std::path::PathBuf;

use axum::{Router, routing::get};

mod contents;
mod errors;
mod vector;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_static_folder::StaticFolder(folder = "docs")] docs_folder: PathBuf,
    #[shuttle_static_folder::StaticFolder(folder = ".")] prefix: PathBuf,
    #[shuttle_secrets::Secrets] secrets: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world));

    let files = contents::load_files_from_dir(docs_folder, &prefix, "mdx")?;
    let mut vector_db = vector::VectorDB::new(&secrets)?;

    Ok(router.into())
}
