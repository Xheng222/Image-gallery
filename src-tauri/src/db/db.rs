use std::{collections::HashSet, path::Path, sync::{Arc, Mutex}, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::Serializer;
use slotmap::{Key, KeyData, SlotMap};
use tauri::{AppHandle, Manager, async_runtime::{Receiver, Sender, channel}, utils::config_v1::AppUrl};
use tokio::time::sleep;
use crate::{APP_HANDLE, db::{models::{AppImageDB, DbCommand, FolderData, FolderID, ImageData, ImageID}, utils::is_image_ext}};



/// ### 数据库
#[derive(Debug, Clone)] 
pub struct ImageDB {
    images: SlotMap<ImageID, ImageData>,
    folders: SlotMap<FolderID, FolderData>,
    db_command_tx: Sender<DbCommand>,
}

impl ImageDB {
    pub fn new() -> &'static AppImageDB {
        let app_handle = APP_HANDLE.get().unwrap();
        let (db_command_tx, db_command_rx) = channel::<DbCommand>(100);
        let image_db: AppImageDB = Self {
            images: SlotMap::with_key(),
            folders: SlotMap::with_key(),
            db_command_tx: db_command_tx.clone(),
        }.into();

        app_handle.manage(image_db);
        tauri::async_runtime::spawn(async move {
            ImageDB::db_background_task(db_command_rx).await;
        });

        app_handle.state::<AppImageDB>().inner()
    }


    /// ## Public Async
    /// ### 发送添加文件夹命令
    pub async fn send_add_folder_command(&self, folder_path: String) {
        let command = DbCommand::AddFolder(folder_path);
        self.send_command(command).await;
    }

    /// ## Private Async
    /// ### 发送数据库命令
    async fn send_command(&self, command: DbCommand) {
        let _ = self.db_command_tx.send(command).await;
    }


    /// ## Private
    /// ### 添加文件夹，启动异步扫描任务
    fn add_folder(&mut self, folder_path: String) {
        let key = self.folders.insert_with_key( |key| {
            FolderData {
                id: key,
                folder_path: folder_path.clone(),
                images: HashSet::new(),
            }
        });

        tauri::async_runtime::spawn_blocking(move ||{
            Self::scan_folder_async(key, folder_path);
        });
        
    }

    /// ## Private
    /// ### 后台线程扫描文件夹
    fn scan_folder_async(folder_id: FolderID, folder_path: String) {
        if let Some(app_handle) = APP_HANDLE.get() {
            let result_arc = Arc::new(Mutex::new(Vec::new()));
            let folder_path = Path::new(&folder_path);
            let folder_walker = ignore::WalkBuilder::new(folder_path.clone())
                .standard_filters(false)
                .build_parallel();
            let folder_path_arc = Arc::new(folder_path);

            folder_walker.run( || {
                let result_arc = Arc::clone(&result_arc);
                let folder_path = Arc::clone(&folder_path_arc);
                Box::new(move |result| {
                    if let Ok(entry) = result {
                        // COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if !entry.file_type().unwrap().is_file() { return ignore::WalkState::Continue; }
                        if !is_image_ext(entry.path()) { return ignore::WalkState::Continue; }

                        if let Ok((w, h)) = image::image_dimensions(&entry.path()) {
                            let image_data = ImageData::new(
                                folder_id, 
                                entry.path().strip_prefix(&*folder_path).unwrap(),
                                w, 
                                h
                            );

                            result_arc.lock().unwrap().push(image_data);
                        }
                    }
                    ignore::WalkState::Continue
                })
            });

            let image_db = app_handle.state::<AppImageDB>();
            let result = Arc::try_unwrap(result_arc).unwrap().into_inner().unwrap();
            let command = DbCommand::AddFilesBatch {
                folder_id: folder_id,
                images: result,
            };

            tauri::async_runtime::spawn(async move {
                image_db.write().await.send_command(command).await;
            });
        }


    }

    /// ### 数据库后台任务,异步读取 `DbCommand`，处理数据库操作，更新前端
    async fn db_background_task(mut db_command_receiver: Receiver<DbCommand>) {
        println!("db_background_task start...");

        let app_handle = APP_HANDLE.get().unwrap();
        let image_db = app_handle.state::<AppImageDB>();

        while let Some(command) = db_command_receiver.recv().await {
            match command {
                DbCommand::AddFolder(folder_path) => {
                    println!("db_command_receiver: add folder {:?}", folder_path);
                    {
                        let mut store = image_db.write().await;
                        store.add_folder(folder_path);
                    }

                }
                
                DbCommand::AddFilesBatch { folder_id, images } => {
                    println!("db_command_receiver: add files batch {} to {:?}", images.len(), folder_id);

                    {
                        let mut store = image_db.write().await;
                        let folder_exists = store.folders.contains_key(folder_id);
                        let mut id_vec = Vec::new();
                        if folder_exists {
                            // 批量处理所有文件
                            for mut image_data in images {
                                let image_id = store.images.insert_with_key(|key| {
                                    image_data.image.id = Some(key);
                                    image_data
                                });

                               id_vec.push(image_id);
                            }
                        }

                        if let Some(folder) = store.folders.get_mut(folder_id) {
                            for image_id in id_vec {
                                folder.images.insert(image_id);
                            }
                        }
                    }

                    println!("Now folders: {:?}", image_db.read().await.folders[folder_id]);
                    println!("Now images: {}", image_db.read().await.images.len());
                    
                    // (可选) 在这里向前端发送进度更新
                    // app.emit_to("main", "scan_progress", files.len());
                }
            }
        }
        
        println!("db_background_task start...");

    }

}

