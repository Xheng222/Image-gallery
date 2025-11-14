use std::path::Path;

use crate::db::models::{FolderID, Image, ImageData, ImageID};


impl ImageData {
    pub fn new(folder_id: FolderID, image_id: ImageID, file_path: String) -> Self {
        let name_index = Path::new(&file_path)
            .file_name()
            .map_or(0, |file_name| {
                file_path.len() - file_name.len()
            });

        let image = Image {
            id: Some(image_id),
            width: 0,
            height: 0,
        };

        Self {
            folder_id,
            image,
            file_path: file_path,
            file_name_index: name_index,
        }
    }

    pub fn file_name(&self) -> &str {
        &self.file_path[self.file_name_index..]
    }
}




