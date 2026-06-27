use std::sync::{Arc, Mutex};

use crate::{
    config::Config,
    display::{Display, WebPreview},
    document::Document,
    fonts::Fonts,
    render,
    storage::Storage,
};

pub struct AppState {
    pub storage: Storage,
    pub config: Mutex<Config>,
    pub document: Mutex<Document>,
    pub display: Arc<dyn Display>,
    pub web_preview: Arc<WebPreview>,
    pub fonts: Arc<Fonts>,
}

impl AppState {
    pub fn render_and_show(&self) -> anyhow::Result<()> {
        let doc = self.document.lock().unwrap().clone();
        let cfg = self.config.lock().unwrap().clone();
        let png = render::render(&doc, &cfg, &self.fonts, &self.storage)?;
        self.display.show(&png)?;
        Ok(())
    }
}
