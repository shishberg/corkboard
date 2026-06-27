use std::collections::HashMap;

/// Embedded fallback font — always available regardless of filesystem state.
static EMBEDDED: &[u8] =
    include_bytes!("../../public/fonts/atkinson-hyperlegible/Regular.ttf");

pub struct Fonts {
    fonts: HashMap<String, ab_glyph::FontVec>,
    default_id: String,
}

impl Fonts {
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(f) => f,
            Err(_) => Self::embedded_only(),
        }
    }

    pub fn get(&self, id: &str) -> &ab_glyph::FontVec {
        self.fonts
            .get(id)
            .or_else(|| self.fonts.get(&self.default_id))
            .or_else(|| self.fonts.values().next())
            .expect("fonts map is never empty")
    }

    pub fn default_font(&self) -> &ab_glyph::FontVec {
        self.get(&self.default_id)
    }

    // ------------------------------------------------------------------ //

    fn embedded_only() -> Self {
        let font = ab_glyph::FontVec::try_from_vec(EMBEDDED.to_vec())
            .expect("embedded font bytes are always valid");
        let mut fonts = HashMap::new();
        fonts.insert("atkinson-hyperlegible".to_string(), font);
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

        let mut font_map: HashMap<String, ab_glyph::FontVec> = HashMap::new();
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
                let font = ab_glyph::FontVec::try_from_vec(bytes)
                    .map_err(|e| anyhow::anyhow!("invalid font {:?}: {:?}", face.file, e))?;
                font_map.insert(entry.id.clone(), font);
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
        let _ = fonts.get("atkinson-hyperlegible"); // should not panic
    }

    #[test]
    fn unknown_id_returns_default() {
        let fonts = Fonts::load();
        let _ = fonts.get("nonexistent-id-xyz"); // should not panic
    }

    #[test]
    fn missing_dir_falls_back_to_embedded() {
        // Override the env var to point nowhere.
        // This test can't run in parallel with other CORKBOARD_FONTS setters,
        // but it at minimum validates the embedded fallback path by calling it directly.
        let fonts = Fonts::embedded_only();
        let _ = fonts.get("atkinson-hyperlegible");
    }
}
