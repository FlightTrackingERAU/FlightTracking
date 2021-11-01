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
}

impl DiskCache {
    pub fn new(folder_name: &str, image_extension: &str) -> Self {
        //Try to create dir. If it fails, we don't care
        let _ = std::fs::create_dir_all(format!("./{}", folder_name));
        Self {
            folder_name: folder_name.to_owned(),
            image_extension: image_extension.to_owned(),
        }
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
            Ok(_) => Ok(Some(tokio::fs::read(path).await?)),
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
}
