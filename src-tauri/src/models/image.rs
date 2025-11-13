use serde::Serialize;


#[derive(Debug, Clone, Serialize)]
pub struct Image {
    pub id: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImageEntry {
    pub image: Image, 
    pub absolute_path: String,
    pub folder_path: String,
    pub filename: String,
}







