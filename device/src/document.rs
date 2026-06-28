#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Landscape,
    Portrait,
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Landscape
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Colour {
    Black,
    White,
    Red,
    Yellow,
    Blue,
    Green,
}

impl Colour {
    pub fn rgb(&self) -> [u8; 3] {
        match self {
            Colour::Black => [0, 0, 0],
            Colour::White => [255, 255, 255],
            Colour::Red => [220, 40, 40],
            Colour::Yellow => [240, 200, 30],
            Colour::Blue => [40, 80, 200],
            Colour::Green => [40, 160, 70],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stroke {
    pub colour: Colour,
    pub size: f32,
    pub points: Vec<Point>,
}

/// Which layout the calendar widget shows.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CalendarVariant {
    Date,
    Today,
    /// 7-day agenda list (Today, Tomorrow, then weekday names). `week` is the
    /// old wire name, kept as an alias so older saved documents still load.
    #[serde(alias = "week")]
    Agenda,
}

impl Default for CalendarVariant {
    fn default() -> Self {
        CalendarVariant::Today
    }
}

/// Horizontal text alignment.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    Center,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Left
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEl {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub colour: Colour,
    #[serde(default)]
    pub variant: CalendarVariant,
    #[serde(default)]
    pub feed_id: String,
    /// Font id for the calendar's text. Empty falls back to the default face.
    #[serde(default)]
    pub font: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageEl {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub colour: Colour,
    /// Image asset id — wire field is `src` to match the editor.
    pub src: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingEl {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub colour: Colour,
    /// Natural (local) width of the drawing canvas. Points are stored in this space.
    #[serde(default)]
    pub nat_w: f32,
    /// Natural (local) height of the drawing canvas.
    #[serde(default)]
    pub nat_h: f32,
    pub strokes: Vec<Stroke>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextEl {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub colour: Colour,
    pub text: String,
    #[serde(default)]
    pub font: String,
    #[serde(default)]
    pub align: TextAlign,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Element {
    Calendar(CalendarEl),
    Image(ImageEl),
    Drawing(DrawingEl),
    Text(TextEl),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Page {
    pub id: String,
    pub name: String,
    pub elements: Vec<Element>,
    /// Page background colour. Absent in documents that predate backgrounds;
    /// the renderer treats `None` as white.
    #[serde(default)]
    pub background: Option<Colour>,
    /// Per-page orientation. Absent in documents that predate per-page
    /// orientation; the document-level `orientation` is used as a fallback.
    #[serde(default)]
    pub orientation: Option<Orientation>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// Legacy document-level orientation. Newer documents set orientation
    /// per-page; this is kept only as a fallback for older saved documents.
    #[serde(default)]
    pub orientation: Option<Orientation>,
    pub pages: Vec<Page>,
    pub live_page_id: Option<String>,
}

impl Document {
    pub fn default() -> Self {
        let page_id = uuid::Uuid::new_v4().to_string();
        Document {
            orientation: None,
            pages: vec![Page {
                id: page_id.clone(),
                name: "Page 1".to_string(),
                elements: vec![],
                background: None,
                orientation: Some(Orientation::Landscape),
            }],
            live_page_id: Some(page_id),
        }
    }

    pub fn live_page(&self) -> Option<&Page> {
        let id = self.live_page_id.as_deref()?;
        self.pages.iter().find(|p| p.id == id)
    }

    /// Orientation of the page currently being displayed. Falls back to the
    /// legacy document-level orientation, then landscape.
    pub fn live_orientation(&self) -> Orientation {
        self.live_page()
            .and_then(|p| p.orientation.clone())
            .or_else(|| self.orientation.clone())
            .unwrap_or_default()
    }

    /// Pixel size of the live page's canvas.
    pub fn orientation_size(&self) -> (u32, u32) {
        match self.live_orientation() {
            Orientation::Landscape => (800, 480),
            Orientation::Portrait => (480, 800),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EDITOR_JSON: &str = r#"{
        "orientation": "landscape",
        "pages": [
            {
                "id": "page-1",
                "name": "Main",
                "elements": [
                    {
                        "type": "calendar",
                        "id": "el-cal-1",
                        "x": 10.0,
                        "y": 10.0,
                        "w": 200.0,
                        "h": 150.0,
                        "colour": "blue",
                        "variant": "week",
                        "feedId": "feed-1"
                    },
                    {
                        "type": "image",
                        "id": "el-img-1",
                        "x": 220.0,
                        "y": 10.0,
                        "w": 100.0,
                        "h": 100.0,
                        "colour": "white",
                        "src": "img-abc123"
                    },
                    {
                        "type": "drawing",
                        "id": "el-draw-1",
                        "x": 0.0,
                        "y": 0.0,
                        "w": 400.0,
                        "h": 300.0,
                        "colour": "black",
                        "natW": 400.0,
                        "natH": 300.0,
                        "strokes": [
                            {
                                "colour": "red",
                                "size": 3.0,
                                "points": [
                                    {"x": 5.0, "y": 5.0},
                                    {"x": 10.0, "y": 15.0}
                                ]
                            }
                        ]
                    },
                    {
                        "type": "text",
                        "id": "el-text-1",
                        "x": 50.0,
                        "y": 200.0,
                        "w": 300.0,
                        "h": 50.0,
                        "colour": "green",
                        "text": "Hello, World!",
                        "font": "atkinson-hyperlegible",
                        "align": "center"
                    }
                ]
            }
        ],
        "livePageId": "page-1",
        "selectedElId": "el-text-1",
        "activeTool": "select",
        "selectedPageId": "page-1"
    }"#;

    #[test]
    fn deserialize_full_editor_json() {
        let doc: Document = serde_json::from_str(EDITOR_JSON).unwrap();

        assert_eq!(doc.pages.len(), 1);
        assert_eq!(doc.live_page_id.as_deref(), Some("page-1"));

        let page = doc.live_page().expect("live page should be found");
        assert_eq!(page.id, "page-1");
        assert_eq!(page.elements.len(), 4);

        // Check all 4 element types deserialized correctly
        assert!(matches!(page.elements[0], Element::Calendar(_)));
        assert!(matches!(page.elements[1], Element::Image(_)));
        assert!(matches!(page.elements[2], Element::Drawing(_)));
        assert!(matches!(page.elements[3], Element::Text(_)));

        // Verify new fields on Image
        if let Element::Image(img) = &page.elements[1] {
            assert_eq!(img.src.as_deref(), Some("img-abc123"));
        }

        // Verify new fields on Drawing
        if let Element::Drawing(d) = &page.elements[2] {
            assert_eq!(d.nat_w, 400.0);
            assert_eq!(d.nat_h, 300.0);
        }

        // Verify new fields on Text
        if let Element::Text(t) = &page.elements[3] {
            assert_eq!(t.font, "atkinson-hyperlegible");
            assert!(matches!(t.align, TextAlign::Center));
        }

        // Verify Calendar variant — the fixture uses the legacy "week" value,
        // which now deserializes to the Agenda variant via its serde alias.
        if let Element::Calendar(c) = &page.elements[0] {
            assert!(matches!(c.variant, CalendarVariant::Agenda));
            assert_eq!(c.feed_id, "feed-1");
        }
    }

    #[test]
    fn image_src_field_deserializes() {
        let json = r#"{
            "type": "image",
            "id": "img-1",
            "x": 0.0, "y": 0.0, "w": 100.0, "h": 100.0,
            "colour": "white",
            "src": "my-image-id"
        }"#;
        let el: Element = serde_json::from_str(json).unwrap();
        if let Element::Image(img) = el {
            assert_eq!(img.src.as_deref(), Some("my-image-id"));
        } else {
            panic!("expected Image element");
        }
    }

    #[test]
    fn orientation_size_follows_live_page() {
        let mut doc = Document::default();
        doc.pages[0].orientation = Some(Orientation::Landscape);
        assert_eq!(doc.orientation_size(), (800, 480));

        doc.pages[0].orientation = Some(Orientation::Portrait);
        assert_eq!(doc.orientation_size(), (480, 800));
    }

    #[test]
    fn orientation_falls_back_to_document_level() {
        // Legacy document: orientation only at the document level, page has none.
        let json = r#"{
            "orientation": "portrait",
            "pages": [{ "id": "p1", "name": "P", "elements": [] }],
            "livePageId": "p1"
        }"#;
        let doc: Document = serde_json::from_str(json).unwrap();
        assert!(doc.pages[0].orientation.is_none());
        assert_eq!(doc.orientation_size(), (480, 800));
    }

    #[test]
    fn per_page_orientation_overrides_document_level() {
        let json = r#"{
            "orientation": "landscape",
            "pages": [{ "id": "p1", "name": "P", "elements": [], "orientation": "portrait" }],
            "livePageId": "p1"
        }"#;
        let doc: Document = serde_json::from_str(json).unwrap();
        assert_eq!(doc.orientation_size(), (480, 800));
    }

    #[test]
    fn live_page_none_when_id_missing() {
        let mut doc = Document::default();
        doc.live_page_id = None;
        assert!(doc.live_page().is_none());
    }

    #[test]
    fn colour_rgb_values() {
        assert_eq!(Colour::Black.rgb(), [0, 0, 0]);
        assert_eq!(Colour::White.rgb(), [255, 255, 255]);
        assert_eq!(Colour::Red.rgb(), [220, 40, 40]);
        assert_eq!(Colour::Yellow.rgb(), [240, 200, 30]);
        assert_eq!(Colour::Blue.rgb(), [40, 80, 200]);
        assert_eq!(Colour::Green.rgb(), [40, 160, 70]);
    }

    #[test]
    fn calendar_variant_defaults_to_today() {
        let json = r#"{
            "type": "calendar",
            "id": "c1",
            "x": 0.0, "y": 0.0, "w": 200.0, "h": 150.0,
            "colour": "black"
        }"#;
        let el: Element = serde_json::from_str(json).unwrap();
        if let Element::Calendar(c) = el {
            assert!(matches!(c.variant, CalendarVariant::Today));
        } else {
            panic!("expected Calendar element");
        }
    }

    #[test]
    fn calendar_variant_week_alias_maps_to_agenda() {
        // Older documents stored the agenda variant as "week"; it must still load.
        let json = r#"{
            "type": "calendar",
            "id": "c1",
            "x": 0.0, "y": 0.0, "w": 200.0, "h": 150.0,
            "colour": "black",
            "variant": "week"
        }"#;
        let el: Element = serde_json::from_str(json).unwrap();
        if let Element::Calendar(c) = el {
            assert!(matches!(c.variant, CalendarVariant::Agenda));
        } else {
            panic!("expected Calendar element");
        }
    }

    #[test]
    fn text_align_defaults_to_left() {
        let json = r#"{
            "type": "text",
            "id": "t1",
            "x": 0.0, "y": 0.0, "w": 200.0, "h": 50.0,
            "colour": "black",
            "text": "Hello"
        }"#;
        let el: Element = serde_json::from_str(json).unwrap();
        if let Element::Text(t) = el {
            assert!(matches!(t.align, TextAlign::Left));
            assert!(t.font.is_empty());
        } else {
            panic!("expected Text element");
        }
    }
}
