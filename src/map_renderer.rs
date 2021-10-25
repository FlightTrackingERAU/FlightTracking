use conrod_core::{
    widget::{Image, Line, Rectangle, Text},
    Colorable, Positionable, Sizeable, UiCell, Widget,
};

use crate::tile_cache::{TileCache, TileId};

// The width and height of a tile in pixels
const TILE_SIZE: u32 = 256;

fn world_to_pixel_x(world_x: f64, ui: &mut UiCell, it: &crate::map::TileViewIterator) -> f64 {
    0.0
}

fn world_y_to_pixel_y(
    world_y: f64,
    viewport: &crate::map::WorldViewport,
    window_height: f64,
) -> f64 {
    let half_height = window_height / 2.0;
    crate::util::map(
        viewport.bottom_right.y,
        viewport.top_left.y,
        world_y,
        -half_height,
        half_height,
    )
}

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

    let it = view.tile_iter(tile_size, ui.win_w, ui.win_h);
    let size = it.tile_size;
    let offset = it.tile_offset;
    let zoom_level = it.tile_zoom;

    let tiles_vertically = it.tiles_vertically;

    let tiles: Vec<_> = it.clone().collect();

    ids.tiles.resize(tiles.len(), &mut ui.widget_id_generator());
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
        } else if cfg!(debug_assertions) {
            //Render debug tile information when run in debug mode

            let text = format!("[{}, {}] @ {}", tile.0, tile.1, zoom_level);
            Text::new(text.as_str())
                .xy_relative([0.0, 0.0])
                .color(conrod_core::color::WHITE)
                .font_size(12)
                .set(ids.square_text[i], ui);
        }
    }

    //Render latitude / longitude lines
    //Render 1 latitude line for approx every 200 vertical pixels
    let wanted_lat_lines = (ui.win_h / 200.0).ceil();
    let viewport = view.get_world_viewport(ui.win_w, ui.win_h);
    let lat_height = viewport.bottom_right.y - viewport.top_left.y;

    //Make lines always aligned with a muliple of 2 world coordinates, and always start at the same
    //point
    let lat_line_distance = crate::util::round_up_pow2(lat_height / wanted_lat_lines);
    let min_lat_world =
        crate::util::round_up(viewport.top_left.y, lat_line_distance) - lat_line_distance;

    let lat_lines = ((viewport.bottom_right.y - min_lat_world) / lat_line_distance).ceil() as usize;
    ids.latitude_lines
        .resize(lat_lines, &mut ui.widget_id_generator());

    for i in 0..lat_lines {
        let y = min_lat_world + i as f64 * lat_line_distance;
        let lat = crate::util::latitude_from_y(y.rem_euclid(1.0));
        let y_pixel = world_y_to_pixel_y(y, &viewport, ui.win_h);
        println!("{} lat at y {}, pixel {}", lat, y, y_pixel);
        Line::new([0.0, y_pixel], [ui.win_w, y_pixel])
            .color(conrod_core::color::WHITE)
            .set(ids.latitude_lines[i], ui);
    }
}
