use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct Activity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<Timestamps>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Assets>,
}

#[derive(Serialize, Clone)]
pub struct Timestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct Assets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}
