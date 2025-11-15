use std::path::Path;

use crate::db::models::{FolderID, Image, ImageData, ImageID};


impl ImageData {
    pub fn new(folder_id: FolderID, file_path: &Path, width: u32, height: u32) -> Self {
        let name_index = file_path
            .file_name()
            .map_or(0, |file_name| {
                file_path.as_os_str().len() - file_name.len()
            });

        let image = Image {
            id: None,
            width: width,
            height: height,
        };

        Self {
            folder_id: folder_id,
            image: image,
            file_path: file_path.to_string_lossy().to_string(),
            file_name_index: name_index,
        }
    }

    pub fn file_name(&self) -> &str {
        &self.file_path[self.file_name_index..]
    }
}




