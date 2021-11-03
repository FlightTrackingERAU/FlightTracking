use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use super::{Backend, ReadinessStatus, TileError, TileId};
use async_trait::async_trait;

fn get_tile_path(folder_name: &str, extension: &str, tile: TileId) -> String {
    format!(
        "./{}/{}/{}/{}.{}",
        folder_name, tile.zoom, tile.x, tile.y, extension
    )
}

#[derive(Copy, Clone)]
pub struct DiskCacheData {
    pub folder_name: &'static str,
    pub image_extension: &'static str,
    pub invalidate_time: Duration,
}

impl DiskCacheData {
    pub async fn cache_tile(&self, tile: TileId, bytes: &[u8]) -> Result<(), std::io::Error> {
        let str_path = get_tile_path(self.folder_name, self.image_extension, tile);
        let path = Path::new(str_path.as_str());
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Some(err) = tokio::fs::create_dir_all(parent).await.err() {
                    println!("Failed to create dir: {} for cache: {:?}", str_path, err);
                }
            }
        }

        tokio::fs::write(path, bytes).await
    }
}

pub struct DiskCache(DiskCacheData);

impl DiskCache {
    pub fn new(data: DiskCacheData) -> Self {
        //Try to create dir. If it fails, we don't care
        let _ = std::fs::create_dir_all(format!("./{}", data.folder_name));
        Self(data)
    }
}

#[async_trait]
impl Backend for DiskCache {
    async fn request_inner(&self, tile: TileId) -> Result<Option<Vec<u8>>, TileError> {
        let path = get_tile_path(self.0.folder_name, self.0.image_extension, tile);
        match std::fs::metadata(&path) {
            Ok(metadata) => {
                if let Ok(last_modified) = metadata.modified() {
                    if let Ok(age) = SystemTime::now().duration_since(last_modified) {
                        if age > self.0.invalidate_time {
                            println!("{:?} - {:?} too old", tile, age);
                        }
                    }
                }

                Ok(Some(tokio::fs::read(path).await?))
            }
            Err(_) => Ok(None),
        }
    }

    async fn readiness(&self, tile: TileId) -> ReadinessStatus {
        let path = get_tile_path(self.0.folder_name, self.0.image_extension, tile);
        match std::fs::metadata(&path) {
            Ok(_) => ReadinessStatus::Available,
            Err(_) => ReadinessStatus::NotAvailable,
        }
    }

    fn name(&self) -> &'static str {
        "Disk"
    }

    fn tile_size(&self) -> Option<u32> {
        //Traverse directory tree, and return length of first image
        fn inner(mut dir_path: PathBuf) -> Result<u32, std::io::Error> {
            let it1 = std::fs::read_dir(&dir_path)?;
            for entry in it1.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        if let Ok(bytes) = std::fs::read(&path) {
                            if let Ok(image) = image::load_from_memory(&bytes[..]) {
                                let rgb = image.to_rgb();
                                let str_path = path.to_string_lossy();
                                if rgb.width() != rgb.height() {
                                    panic!("Image in cache: {}, is not square", str_path);
                                }
                                println!(
                                    "Using image {} as model cache size: {}",
                                    str_path,
                                    rgb.width()
                                );
                                return Ok(rgb.width());
                            }
                        }
                    } else if metadata.is_dir() {
                        dir_path.push(entry.file_name());
                        //Recurse directory structure
                        if let Ok(size) = inner(dir_path.clone()) {
                            return Ok(size);
                        }
                        dir_path.pop();
                    }
                }
            }
            Err(std::io::Error::last_os_error())
        }

        inner(PathBuf::from(self.0.folder_name)).ok()
    }
}
