use std::collections::HashMap;

/// Embedded fallback font — always available regardless of filesystem state.
static EMBEDDED: &[u8] =
    include_bytes!("../../public/fonts/atkinson-hyperlegible/Regular.ttf");

/// Loaded font files, keyed by id. We keep the raw bytes (not a parsed font):
/// the renderer builds FreeType faces from these per render pass, because
/// FreeType faces are `!Send`/`!Sync` and `Fonts` lives in shared state.
pub struct Fonts {
    fonts: HashMap<String, Vec<u8>>,
    default_id: String,
}

impl Fonts {
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(f) => f,
            Err(_) => Self::embedded_only(),
        }
    }

    pub fn default_id(&self) -> &str {
        &self.default_id
    }

    /// All (id, bytes) pairs — used to build the per-render face set.
    pub fn entries(&self) -> impl Iterator<Item = (&String, &Vec<u8>)> {
        self.fonts.iter()
    }

    // ------------------------------------------------------------------ //

    fn embedded_only() -> Self {
        let mut fonts = HashMap::new();
        fonts.insert("atkinson-hyperlegible".to_string(), EMBEDDED.to_vec());
        Fonts {
            fonts,
            default_id: "atkinson-hyperlegible".to_string(),
        }
    }

    fn try_load() -> anyhow::Result<Self> {
        let fonts_dir_str = std::env::var("CORKBOARD_FONTS")
            .unwrap_or_else(|_| "../public/fonts".to_string());
        let fonts_dir = std::path::Path::new(&fonts_dir_str);

        let manifest_text = std::fs::read_to_string(fonts_dir.join("manifest.json"))?;
        let manifest: Manifest = serde_json::from_str(&manifest_text)?;

        let mut font_map: HashMap<String, Vec<u8>> = HashMap::new();
        let mut default_id = "atkinson-hyperlegible".to_string();

        for entry in &manifest.fonts {
            if entry.default {
                default_id = entry.id.clone();
            }
            let face = entry
                .faces
                .iter()
                .find(|f| f.weight == 400 && f.style == "normal")
                .or_else(|| entry.faces.first());

            if let Some(face) = face {
                let bytes = std::fs::read(fonts_dir.join(&face.file))?;
                font_map.insert(entry.id.clone(), bytes);
            }
        }

        anyhow::ensure!(!font_map.is_empty(), "no fonts found in manifest");

        Ok(Fonts {
            fonts: font_map,
            default_id,
        })
    }
}

// JSON manifest shape ---------------------------------------------------

#[derive(serde::Deserialize)]
struct FontFace {
    weight: u32,
    style: String,
    file: String,
}

#[derive(serde::Deserialize)]
struct FontEntry {
    id: String,
    #[allow(dead_code)]
    name: String,
    #[serde(default)]
    default: bool,
    faces: Vec<FontFace>,
}

#[derive(serde::Deserialize)]
struct Manifest {
    fonts: Vec<FontEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_font_loads() {
        let fonts = Fonts::load();
        assert!(fonts.entries().count() >= 1);
        assert!(!fonts.default_id().is_empty());
    }

    #[test]
    fn embedded_fallback_has_the_default_font() {
        // Validate the embedded fallback path directly (can't toggle the env var
        // safely in parallel with other CORKBOARD_FONTS setters).
        let fonts = Fonts::embedded_only();
        assert_eq!(fonts.default_id(), "atkinson-hyperlegible");
        let (id, bytes) = fonts.entries().next().unwrap();
        assert_eq!(id, "atkinson-hyperlegible");
        assert!(!bytes.is_empty());
    }
}
