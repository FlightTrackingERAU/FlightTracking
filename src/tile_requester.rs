use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use maptiler_cloud::{Maptiler, TileRequest};
use tokio::runtime::Runtime;

use crate::tile_cache::{Tile, TileId};

pub struct TileRequester {
    tile_rx: UnboundedReceiver<Tile>,
    request_tx: UnboundedSender<TileId>,
}

impl TileRequester {
    pub fn new<S>(api_key: S, runtime: &Runtime) -> Self
    where
        S: AsRef<str>,
    {
        // The channel with which the thread can send back the tiles that we requested
        let (tile_tx, tile_rx) = tokio::sync::mpsc::unbounded_channel();
        // This channel allows us to send and receive tile requests
        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel();

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
    tile_tx: UnboundedSender<Tile>,
    mut request_rx: UnboundedReceiver<TileId>,
) {
    // Can customize the runtime parameters later
    // This uses expect(), because we are already in another thread, we would kind of already be in
    // trouble.
    let maptiler = Maptiler::new(api_key);

    loop {
        if let Some(tile_id) = request_rx.recv().await {
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

            // Spawn the request function. Will push the tile to tile_tx when the request completes
            let tile_tx = tile_tx.clone();
            tokio::spawn(async move {
                if let Ok(tile_bytes) = request.execute().await {
                    // Create an RGBA image from the JPEG bytes
                    let image = image::load_from_memory(&tile_bytes).unwrap().into_rgba();

                    let tile = Tile { id: tile_id, image };

                    tile_tx.send(tile).ok();
                }
            });
        }
    }
}
