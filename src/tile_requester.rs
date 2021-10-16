use std::sync::mpsc::{Receiver, SyncSender};

use maptiler_cloud::{ConstructedRequest, Maptiler, TileRequest};

use crate::tile_cache::{Tile, TileId};

const TILE_BUFFER_SIZE: usize = 1024;
const REQUEST_BUFFER_SIZE: usize = 1024;

pub struct TileRequester {
    tile_rx: tokio::sync::mpsc::Receiver<Tile>,
    request_tx: SyncSender<TileId>,
}

impl TileRequester {
    pub fn spawn<S>(api_key: S) -> Self
    where
        S: AsRef<str>,
    {
        // The channel with which the thread can send back the tiles that we requested
        let (tile_tx, tile_rx) = tokio::sync::mpsc::channel(TILE_BUFFER_SIZE);
        // This channel allows us to send and receive tile requests
        let (request_tx, request_rx) = std::sync::mpsc::sync_channel(REQUEST_BUFFER_SIZE);

        let api_key = api_key.as_ref().to_string();

        // Spawn our request loop thread
        std::thread::spawn(move || request_loop(api_key, tile_tx, request_rx));

        Self {
            tile_rx,
            request_tx,
        }
    }

    /// Sends a request to get a tile
    pub fn request(&mut self, tile_request: TileId) {
        // Send the tile id over the channel
        self.request_tx.send(tile_request).ok();
    }

    /// Returns an Option<Vec> that contains all tiles (if any) that have been succesfully requested
    /// since we have last checked
    pub fn new_tiles(&mut self) -> Option<Vec<Tile>> {
        let mut tiles = Vec::new();

        while let Ok(tile) = self.tile_rx.try_recv() {
            tiles.push(tile);
        }

        if tiles.len() == 0 {
            None
        } else {
            Some(tiles)
        }
    }
}

fn request_loop(
    api_key: String,
    tile_tx: tokio::sync::mpsc::Sender<Tile>,
    request_rx: Receiver<TileId>,
) {
    // Can customize the runtime parameters later
    // This uses expect(), because we are already in another thread, we would kind of already be in
    // trouble.
    let runtime = tokio::runtime::Runtime::new().expect("Unable to create Tokio runtime!");
    let maptiler = Maptiler::new(api_key);

    loop {
        if let Ok(tile_id) = request_rx.recv() {
            // This should panic to aid in debugability
            // Create the tile request
            let tile_request = TileRequest::new(
                maptiler_cloud::TileSet::Satellite,
                tile_id.x,
                tile_id.y,
                tile_id.zoom,
            )
            .expect(&format!("Invalid tile requested: {:?}", tile_id));

            // Create the request using the Maptiler
            let request = maptiler.create_request(tile_request);

            // Spawn the request function to be awaited
            runtime.spawn(request_tile(
                tokio::sync::mpsc::Sender::clone(&tile_tx),
                tile_id,
                request,
            ));
        }
    }
}

async fn request_tile(
    tile_tx: tokio::sync::mpsc::Sender<Tile>,
    id: TileId,
    request: ConstructedRequest,
) {
    if let Ok(tile_bytes) = request.execute().await {
        // Create an RGBA image from the JPEG bytes
        let image = image::load_from_memory(&tile_bytes).unwrap().into_rgba();

        let tile = Tile { id, image };

        tile_tx.send(tile).await.ok();
    }
}
