use std::path::PathBuf;

use crate::config::Config;
use crate::document::{Document, Element};

pub struct Storage {
    pub root: PathBuf,
}

impl Storage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Storage { root: root.into() }
    }

    fn doc_path(&self) -> PathBuf {
        self.root.join("document.json")
    }

    fn images_dir(&self) -> PathBuf {
        self.root.join("images")
    }

    pub fn image_path(&self, id: &str) -> PathBuf {
        self.images_dir().join(format!("{}.bin", id))
    }

    fn config_path(&self) -> PathBuf {
        self.root.join("config.json")
    }

    pub fn load_document(&self) -> anyhow::Result<Document> {
        let path = self.doc_path();
        if !path.exists() {
            return Ok(Document::default());
        }
        let data = std::fs::read_to_string(&path)?;
        let doc = serde_json::from_str(&data)?;
        Ok(doc)
    }

    pub fn save_document(&self, doc: &Document) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        let data = serde_json::to_string_pretty(doc)?;
        std::fs::write(self.doc_path(), data)?;
        Ok(())
    }

    pub fn save_image(&self, id: &str, bytes: &[u8]) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.images_dir())?;
        std::fs::write(self.image_path(id), bytes)?;
        Ok(())
    }

    pub fn load_image(&self, id: &str) -> Option<Vec<u8>> {
        std::fs::read(self.image_path(id)).ok()
    }

    pub fn list_image_ids(&self) -> Vec<String> {
        let dir = self.images_dir();
        if !dir.exists() {
            return vec![];
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return vec![];
        };
        entries
            .filter_map(|e| {
                let e = e.ok()?;
                let name = e.file_name();
                let s = name.to_string_lossy();
                if s.ends_with(".bin") {
                    Some(s.trim_end_matches(".bin").to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn gc_images(&self, doc: &Document) -> anyhow::Result<()> {
        // Collect all image IDs referenced in the document
        let mut referenced = std::collections::HashSet::new();
        for page in &doc.pages {
            for el in &page.elements {
                if let Element::Image(img) = el {
                    if let Some(id) = &img.src {
                        referenced.insert(id.clone());
                    }
                }
            }
        }

        let stored = self.list_image_ids();
        for id in stored {
            if !referenced.contains(&id) {
                let path = self.image_path(&id);
                if path.exists() {
                    std::fs::remove_file(&path)?;
                }
            }
        }
        Ok(())
    }

    pub fn load_config(&self) -> Config {
        Config::load(&self.config_path())
    }

    pub fn save_config(&self, cfg: &Config) -> anyhow::Result<()> {
        cfg.save(&self.config_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Colour, ImageEl, Page};

    #[test]
    fn save_load_document_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());

        let doc = Document::default();
        storage.save_document(&doc).unwrap();

        let loaded = storage.load_document().unwrap();
        assert_eq!(loaded.pages.len(), 1);
        assert_eq!(loaded.live_page_id, doc.live_page_id);
    }

    #[test]
    fn load_document_missing_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());
        let doc = storage.load_document().unwrap();
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn gc_images_removes_unreferenced() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path());

        // Save two images
        storage.save_image("img-keep", b"keep-data").unwrap();
        storage.save_image("img-delete", b"delete-data").unwrap();

        // Build a document that references only "img-keep"
        let page_id = "page-1".to_string();
        let doc = Document {
            orientation: crate::document::Orientation::Landscape,
            live_page_id: Some(page_id.clone()),
            pages: vec![Page {
                id: page_id,
                name: "Page 1".to_string(),
                elements: vec![Element::Image(ImageEl {
                    id: "el-1".to_string(),
                    x: 0.0,
                    y: 0.0,
                    w: 100.0,
                    h: 100.0,
                    colour: Colour::White,
                    src: Some("img-keep".to_string()),
                })],
            }],
        };

        storage.gc_images(&doc).unwrap();

        assert!(storage.image_path("img-keep").exists());
        assert!(!storage.image_path("img-delete").exists());
    }
}
