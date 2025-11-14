use serde::{Deserialize, Serialize};
use serde_json::Serializer;
use slotmap::{Key, KeyData, SlotMap};
use crate::db::models::{AppImageDB, FolderData, FolderID, ImageData, ImageID};



/// ### 数据库
#[derive(Debug, Clone, Serialize, Deserialize)] 
pub struct ImageDB {
    images: SlotMap<ImageID, ImageData>,
    folders: SlotMap<FolderID, FolderData>,
}

impl ImageDB {
    pub fn new() -> AppImageDB {
        Self {
            images: SlotMap::with_key(),
            folders: SlotMap::with_key(),
        }.into()
    }

    pub fn add_folder(&mut self, folder_path: String) {
        let key = self.folders.insert_with_key( |key| {
            FolderData {
                id: key,
                folder_path: folder_path,
                images: Vec::new(),
            }
        });

        let folder = &self.folders[key];



    }

    /// ### 扫描文件夹
    fn scan_folder(&mut self, folder: &FolderData) {

    }
}

