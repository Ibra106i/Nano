use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileOperation {
    New,
    Open(PathBuf),
    Save(PathBuf),
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: Option<PathBuf>,
    pub is_modified: bool,
}

impl FileInfo {
    pub fn new() -> Self {
        FileInfo {
            path: None,
            is_modified: false,
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        FileInfo {
            path: Some(path),
            is_modified: false,
        }
    }

    pub fn filename(&self) -> String {
        match &self.path {
            Some(path) => path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string(),
            None => "Untitled".to_string(),
        }
    }

    pub fn full_path(&self) -> String {
        match &self.path {
            Some(path) => path.to_string_lossy().to_string(),
            None => "No file".to_string(),
        }
    }

    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }

    pub fn mark_saved(&mut self) {
        self.is_modified = false;
    }
}

pub fn open_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text Files", &["txt", "rs", "py", "js", "ts", "jsx", "tsx", "html", "css", "json", "md", "toml", "yaml", "yml", "xml"])
        .add_filter("All Files", &["*"])
        .pick_file()
}

pub fn save_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text Files", &["txt", "rs", "py", "js", "ts", "jsx", "tsx", "html", "css", "json", "md", "toml", "yaml", "yml", "xml"])
        .add_filter("All Files", &["*"])
        .save_file()
}

pub fn read_file(path: &PathBuf) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

pub fn write_file(path: &PathBuf, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
}
