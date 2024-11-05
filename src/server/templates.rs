use askama_axum::Template;

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct Dashboard {
    pub event_count: usize,
    pub state_hash: String,
    pub peer_count: usize,
    pub version: String,
}
