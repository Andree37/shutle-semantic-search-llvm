use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::errors::NotAvailableError;

enum FileState {
    None,
    CodeBlock,
    Sentence,
    Meta,
}

pub struct File {
    pub path: String,
    pub contents: String,
    pub sentences: Vec<String>,
}

impl File {
    pub fn new(path: String, contents: String) -> Self {
        Self {
            path,
            contents,
            sentences: Vec::new(),
        }
    }

    // TODO: we can also parse until there is a '.' for each sentence
    pub fn parse(&mut self) {
        let mut contents = Vec::new();
        let mut state = FileState::None;
        let mut sentence = String::new();

        for line in self.contents.lines() {
            match state {
                FileState::None => {
                    if line.starts_with("```") {
                        state = FileState::CodeBlock;
                        sentence = String::new();
                        sentence.push_str(line);
                        sentence.push('\n');
                    } else if line.starts_with("---") {
                        state = FileState::Meta;
                    } else if !line.starts_with('#') && !line.is_empty() {
                        state = FileState::Sentence;
                        sentence = String::new();
                        sentence.push_str(line);
                        sentence.push('\n');
                    }
                }
                FileState::CodeBlock => {
                    sentence.push_str(line);
                    if line.starts_with("```") {
                        contents.push(sentence);
                        sentence = String::new();
                        state = FileState::None;
                    }
                }
                FileState::Meta => {
                    if line.starts_with("---") {
                        state = FileState::None;
                    }
                }
                FileState::Sentence => {
                    if line.is_empty() {
                        state = FileState::None;
                        contents.push(sentence);
                        sentence = String::new();
                    } else {
                        sentence.push_str(line);
                        sentence.push('\n');
                    }
                }
            }
        }
        self.sentences = contents;
    }
}

trait HasFileExt {
    fn has_file_ext(&self, ext: &str) -> bool;
}

impl HasFileExt for PathBuf {
    fn has_file_ext(&self, ext: &str) -> bool {
        return if let Some(path) = self.to_str() {
            path.ends_with(ext)
        } else {
            false
        };
    }
}

pub fn load_files_from_dir(dir: PathBuf, prefix: &PathBuf, ending: &str) -> Result<Vec<File>> {
    let mut files = Vec::new();

    let dir = fs::read_dir(dir)?;
    for entry in dir {
        let path = entry?.path();
        if path.is_dir() {
            let mut sub_files = load_files_from_dir(path, prefix, ending)?;
            files.append(&mut sub_files);
        } else if path.is_file() && path.has_file_ext(ending) {
            let path = Path::new(&path).strip_prefix(prefix)?.to_owned();
            println!("Loading file: {:?}", path);
            let contents = fs::read_to_string(&path)?;
            let key = path.to_str().ok_or(NotAvailableError {})?;
            let mut file = File::new(key.to_string(), contents);
            file.parse();
            files.push(file);
        }
    }

    return Ok(files);
}