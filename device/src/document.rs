#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Landscape,
    Portrait,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEl {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub colour: Colour,
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
    pub image_id: Option<String>,
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
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub orientation: Orientation,
    pub pages: Vec<Page>,
    pub live_page_id: Option<String>,
}

impl Document {
    pub fn default() -> Self {
        let page_id = uuid::Uuid::new_v4().to_string();
        Document {
            orientation: Orientation::Landscape,
            pages: vec![Page {
                id: page_id.clone(),
                name: "Page 1".to_string(),
                elements: vec![],
            }],
            live_page_id: Some(page_id),
        }
    }

    pub fn live_page(&self) -> Option<&Page> {
        let id = self.live_page_id.as_deref()?;
        self.pages.iter().find(|p| p.id == id)
    }

    pub fn orientation_size(&self) -> (u32, u32) {
        match self.orientation {
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
                        "colour": "blue"
                    },
                    {
                        "type": "image",
                        "id": "el-img-1",
                        "x": 220.0,
                        "y": 10.0,
                        "w": 100.0,
                        "h": 100.0,
                        "colour": "white",
                        "imageId": "img-abc123"
                    },
                    {
                        "type": "drawing",
                        "id": "el-draw-1",
                        "x": 0.0,
                        "y": 0.0,
                        "w": 400.0,
                        "h": 300.0,
                        "colour": "black",
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
                        "text": "Hello, World!"
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
    }

    #[test]
    fn orientation_size() {
        let mut doc = Document::default();
        doc.orientation = Orientation::Landscape;
        assert_eq!(doc.orientation_size(), (800, 480));

        doc.orientation = Orientation::Portrait;
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
}
