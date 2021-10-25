use conrod_core::{widget::Image, Positionable, Sizeable, UiCell, Widget};

use crate::tile_cache::{TileCache, TileId};

pub fn draw(
    tile_cache: &mut TileCache,
    view: &crate::map::TileView,
    display: &glium::Display,
    image_map: &mut conrod_core::image::Map<glium::Texture2d>,
    ids: &mut crate::Ids,
    ui: &mut UiCell,
) {
    //Or value is okay here because `tile_size()` only returns `None` if no tiles are cached, which
    //only happens the first few frames, therefore this value doesn't need to be accurate
    let tile_size = tile_cache.tile_size().unwrap_or(256) / 2;
    println!("Using size: {}", tile_size);

    let it = view.tile_iter(tile_size, ui.win_w as u32, ui.win_h as u32);
    let size = it.tile_size;
    let offset = it.tile_offset;
    let zoom_level = it.tile_zoom;

    let tiles_vertically = it.tiles_vertically;

    let tiles: Vec<_> = it.collect();

    // Canvas::new().pad(0.0).set(ids.viewport, ui);
    /*
    Canvas::new()
        .color(conrod_core::Color::Rgba(0.0, 0.0, 0.0, 1.0))
        .middle()
        .w(1280.0)
        .h(720.0)
        .set(ids.viewport, ui);
    */

    ids.tiles.resize(tiles.len(), &mut ui.widget_id_generator());
    ids.squares
        .resize(tiles.len(), &mut ui.widget_id_generator());
    ids.square_text
        .resize(tiles.len(), &mut ui.widget_id_generator());

    // The conrod coordinate system places 0, 0 in the center of the window. Up is the positive y
    // axis, and right is the positive x axis.
    // The units are in terms of screen pixels, so on a window with a size of 1000x500 the point
    // (500, 250) would be the top right corner
    for (i, tile) in tiles.into_iter().enumerate() {
        let tile_x = i / tiles_vertically as usize;
        let tile_y = i % tiles_vertically as usize;

        let half_width = ui.win_w / 2.0;
        let half_height = ui.win_h / 2.0;
        let x = offset.x + tile_x as f64 * size.x - half_width + size.x / 2.0;
        let y = offset.y - (tile_y as f64 * size.y) + half_height + size.y / 2.0;

        let tile_id = TileId::new(tile.0, tile.1, zoom_level);

        tile_cache.process(display, image_map);

        if let Some(tile) = tile_cache.get_tile(tile_id) {
            Image::new(tile)
                .x_y(x, y)
                .wh(size.to_array())
                .set(ids.tiles[i], ui);
        } /* else {
              Rectangle::outline(size.to_array())
                  .x_y(x, y)
                  .set(ids.squares[i], ui);

              let text = format!("[{}, {}] @ {}", tile.0, tile.1, zoom_level);
              Text::new(text.as_str())
                  .xy_relative([0.0, 0.0])
                  .color(conrod_core::color::WHITE)
                  .font_size(12)
                  .set(ids.square_text[i], ui);
          } */
    }
}
