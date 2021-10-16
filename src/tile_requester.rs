use std::sync::mpsc::{Receiver, SyncSender};

use maptiler_cloud::{ConstructedRequest, Maptiler, TileRequest};
use tokio::runtime::Runtime;

use crate::tile_cache::{Tile, TileId};

const TILE_BUFFER_SIZE: usize = 1024;
const REQUEST_BUFFER_SIZE: usize = 1024;

pub struct TileRequester {
    tile_rx: tokio::sync::mpsc::Receiver<Tile>,
    request_tx: SyncSender<TileId>,
}

impl TileRequester {
    pub fn new<S>(api_key: S, runtime: &Runtime) -> Self
    where
        S: AsRef<str>,
    {
        // The channel with which the thread can send back the tiles that we requested
        let (tile_tx, tile_rx) = tokio::sync::mpsc::channel(TILE_BUFFER_SIZE);
        // This channel allows us to send and receive tile requests
        let (request_tx, request_rx) = std::sync::mpsc::sync_channel(REQUEST_BUFFER_SIZE);

        let api_key = api_key.as_ref().to_string();

        runtime.spawn(request_loop(api_key, tile_tx, request_rx));

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

    /// Returns Some(tile) if a tile is ready, or None otherwise.
    ///
    /// The caller should repeat calls to flush all ready tiles
    pub fn next_ready_tile(&mut self) -> Option<Tile> {
        self.tile_rx.try_recv().ok()
    }
}

async fn request_loop(
    api_key: String,
    tile_tx: tokio::sync::mpsc::Sender<Tile>,
    request_rx: Receiver<TileId>,
) {
    // Can customize the runtime parameters later
    // This uses expect(), because we are already in another thread, we would kind of already be in
    // trouble.
    let maptiler = Maptiler::new(api_key);

    loop {
        if let Ok(tile_id) = request_rx.recv() {
            // This should panic to aid in educability
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

            //TODO: Maybe look at priorities here and limit pending requests to a reasonable number when under load
            // Spawn the request function to be awaited
            tokio::spawn(request_tile(
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
