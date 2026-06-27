use std::sync::{Arc, Mutex};

use crate::{
    config::Config,
    display::Display,
    document::Document,
    render,
    storage::Storage,
};

pub struct AppState {
    pub storage: Storage,
    pub config: Mutex<Config>,
    pub document: Mutex<Document>,
    pub display: Arc<dyn Display>,
}

impl AppState {
    pub fn render_and_show(&self) -> anyhow::Result<()> {
        let doc = self.document.lock().unwrap().clone();
        let cfg = self.config.lock().unwrap().clone();
        let png = render::render(&doc, &cfg)?;
        self.display.show(&png)?;
        Ok(())
    }
}
