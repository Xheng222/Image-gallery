use serde::{Deserialize, Serialize};
use slotmap::{KeyData, new_key_type};
use crate::db::db::ImageDB;

new_key_type! {
    pub struct FolderID;
}

new_key_type! {
    pub struct ImageID;
}

pub type AppImageDB = tokio::sync::RwLock<ImageDB>;

/// ### 给前端的 Image 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: Option<ImageID>,
    pub width: u32,
    pub height: u32,
}

/// ### 后端使用的 ImageData 信息
/// - file_path 是基于 Folder 的相对路径
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ImageData {
    pub folder_id: FolderID,
    pub file_path: String,
    pub file_name_index: usize,
    pub image: Image,
}

/// ### 文件夹数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct FolderData {
    pub id: FolderID,
    pub folder_path: String,
    pub images: Vec<ImageID>,
}

/// 发送给数据库的命令
#[derive(Debug)]
pub enum DbCommand {
    /// 添加一个文件夹（仅创建 FolderData）
    AddFolder(String), 
    
    /// 批量添加在一个文件夹中找到的文件
    AddFilesBatch {
        folder_path: FolderID, // 文件夹 ID
        files: Vec<ImageData>,  // 找到的一批文件
    },
    
    // ... 未来可以扩展, e.g., RemoveFolder, RemoveFile ...
}





