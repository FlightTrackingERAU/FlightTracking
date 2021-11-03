use std::{
    path::PathBuf,
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

pub struct DiskCache {
    folder_name: String,
    image_extension: String,
    invalidate_time: Duration,
}

impl DiskCache {
    pub fn new(
        folder_name: &str,
        image_extension: &str,
        invalidate_time: Option<Duration>,
    ) -> Self {
        //Try to create dir. If it fails, we don't care
        let _ = std::fs::create_dir_all(format!("./{}", folder_name));
        Self {
            folder_name: folder_name.to_owned(),
            image_extension: image_extension.to_owned(),
            invalidate_time: invalidate_time.unwrap_or(Duration::MAX),
        }
    }

    pub async fn cache_tile(&self, tile: TileId, bytes: &[u8]) -> Result<(), std::io::Error> {
        let path = get_tile_path(
            self.folder_name.as_str(),
            self.image_extension.as_str(),
            tile,
        );

        tokio::fs::write(path, bytes).await
    }
}

#[async_trait]
impl Backend for DiskCache {
    async fn request_inner(&self, tile: TileId) -> Result<Option<Vec<u8>>, TileError> {
        let path = get_tile_path(
            self.folder_name.as_str(),
            self.image_extension.as_str(),
            tile,
        );
        match std::fs::metadata(&path) {
            Ok(metadata) => {
                if let Ok(last_modified) = metadata.modified() {
                    if let Ok(age) = SystemTime::now().duration_since(last_modified) {
                        if age > self.invalidate_time {
                            println!("{:?} - {:?} too old", tile, age);
                        } else {
                            println!("{:?} ok", tile);
                        }
                    }
                }

                Ok(Some(tokio::fs::read(path).await?))
            }
            Err(_) => Ok(None),
        }
    }

    async fn readiness(&self, tile: TileId) -> ReadinessStatus {
        let path = get_tile_path(
            self.folder_name.as_str(),
            self.image_extension.as_str(),
            tile,
        );
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

        inner(PathBuf::from(self.folder_name.as_str())).ok()
    }
}
