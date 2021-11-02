use conrod_core::{
    widget::{Image, Line, Text},
    Colorable, Positionable, Sizeable, UiCell, Widget,
};

use crate::tile::{self, *};

fn world_x_to_pixel_x(
    world_x: f64,
    viewport: &crate::map::WorldViewport,
    window_width: f64,
) -> f64 {
    let half_width = window_width / 2.0;
    crate::util::map(
        viewport.top_left.x,
        viewport.bottom_right.x,
        world_x,
        -half_width,
        half_width,
    )
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

/// Returns how many degrees should between lines given the viewport range (in world coordinates), and the size
/// of the window, either width or height, depending on which dimension these lines are for
fn line_distance_for_viewport_degrees(world_range: f64, dimension_size: f64) -> f64 {
    // A neive approximation is ok here since we are only determining the distance between lines
    let range_degrees = world_range * 180.0;

    // Range in degrees, adjusted for screen size
    let mapped_range = range_degrees * 500.0 / dimension_size;
    const DISTANCE_SCALE: f64 = 2.0;

    // Define nice distance values between lines for large distances
    let mapping = [45.0, 15.0, 5.0, 2.0, 1.0];
    for distance in mapping {
        let min_range = distance * DISTANCE_SCALE;
        if mapped_range > min_range {
            return distance;
        }
    }
    let power = (mapped_range / DISTANCE_SCALE).log10();
    let part = power.rem_euclid(1.0);
    //We know the scale and where the number falls within the exponential range
    //so use math to find the correct spacing

    let int_power = power.ceil() as i32;

    if part >= 0.5 {
        0.5 * 10.0f64.powi(int_power)
    } else if part >= 0.2 {
        0.2 * 10.0f64.powi(int_power)
    } else {
        0.1 * 10.0f64.powi(int_power)
    }
}

fn world_width_from_longitude(lng: f64) -> f64 {
    // The world is 360 degrees around, and in world coordinates, 1.0 units around
    lng / 360.0
}

pub fn draw(
    tile_cache: &mut tile::PipelineMap,
    view: &crate::map::TileView,
    display: &glium::Display,
    image_map: &mut conrod_core::image::Map<glium::Texture2d>,
    ids: &mut crate::Ids,
    ui: &mut UiCell,
) {
    let _scope = crate::profile_scope("map_renderer::draw");
    //Or value is okay here because `tile_size()` only returns `None` if no tiles are cached, which
    //only happens the first few frames, therefore this value doesn't need to be accurate
    let tile_size = 128;

    let it = view.tile_iter(tile_size, ui.win_w, ui.win_h);
    let size = it.tile_size;
    let offset = it.tile_offset;
    let zoom_level = it.tile_zoom;

    let tiles_vertically = it.tiles_vertically;

    let tiles: Vec<_> = it.collect();
    {
        let mut guard = crate::MAP_PERF_DATA.lock();
        guard.tiles_rendered = tiles.len();
        guard.zoom = zoom_level;
    }

    ids.weather_tiles
        .resize(tiles.len(), &mut ui.widget_id_generator());
    ids.tiles.resize(tiles.len(), &mut ui.widget_id_generator());
    ids.square_text
        .resize(tiles.len(), &mut ui.widget_id_generator());

    let viewport = view.get_world_viewport(ui.win_w, ui.win_h);

    let mut cache_it = tile_cache.values_mut();
    let satellite = cache_it.next().unwrap();
    {
        let _p = crate::profile_scope("Satellite Tile Cache Update");
        satellite.update(&viewport, display, image_map);
    }

    let weather = cache_it.next().unwrap();
    {
        let _p = crate::profile_scope("Weather Tile Cache Update");
        weather.update(&viewport, display, image_map);
    }

    // The conrod coordinate system places 0, 0 in the center of the window. Up is the positive y
    // axis, and right is the positive x axis.
    // The units are in terms of screen pixels, so on a window with a size of 1000x500 the point
    // (500, 250) would be the top right corner
    let scope_render_tiles = crate::profile_scope("Render Tiles");
    for (i, tile) in tiles.iter().enumerate() {
        let tile_x = i / tiles_vertically as usize;
        let tile_y = i % tiles_vertically as usize;

        let half_width = ui.win_w / 2.0;
        let half_height = ui.win_h / 2.0;
        let x = offset.x + tile_x as f64 * size.x - half_width + size.x / 2.0;
        let y = offset.y - (tile_y as f64 * size.y) + half_height + size.y / 2.0;

        let tile_id = TileId::new(tile.0, tile.1, zoom_level);

        if let Some(tile) = satellite.get_tile(tile_id) {
            Image::new(tile)
                .x_y(x, y)
                .wh(size.to_array())
                .set(ids.tiles[i], ui);
        }

        if let Some(tile) = weather.get_tile(tile_id) {
            Image::new(tile)
                .x_y(x, y)
                .wh(size.to_array())
                .set(ids.weather_tiles[i], ui);
        }
    }
    scope_render_tiles.end();

    let scope_render_latitude = crate::profile_scope("Render Latitude");
    //Lines of latitude
    let lat_line_distance =
        line_distance_for_viewport_degrees(viewport.bottom_right.y - viewport.top_left.y, ui.win_h);

    let lat_top = crate::util::latitude_from_y(viewport.top_left.y.rem_euclid(1.0));
    let lat_bottom = crate::util::latitude_from_y(viewport.bottom_right.y.rem_euclid(1.0));
    let lat_start = crate::util::modulo_ceil(lat_top, lat_line_distance);

    let lat_lines = ((lat_top - lat_bottom) / lat_line_distance + 1.0).ceil() as usize;

    ids.latitude_lines
        .resize(lat_lines, &mut ui.widget_id_generator());
    ids.latitude_text
        .resize(lat_lines, &mut ui.widget_id_generator());

    let log10_line_distance = lat_line_distance.log10();
    let precision = if log10_line_distance < 0.0 {
        (-log10_line_distance.floor()) as usize
    } else {
        0usize
    };

    const LINE_ALPHA: f32 = 0.4;

    //Latitude decreases as world y increases
    for i in 0..lat_lines {
        let lat = lat_start - i as f64 * lat_line_distance;
        let world_y = crate::util::y_from_latitude(lat);
        let y_pixel = world_y_to_pixel_y(world_y, &viewport, ui.win_h);

        let half_width = ui.win_w / 2.0;
        Line::new([-half_width, y_pixel], [half_width, y_pixel])
            //Why does this call need to happen?
            .x_y(0.0, 0.0)
            .color(conrod_core::color::BLACK.alpha(LINE_ALPHA))
            .thickness(1.5)
            .set(ids.latitude_lines[i], ui);

        let text = if lat >= 0.0 {
            format!("{:.1$}°N", lat, precision)
        } else {
            format!("{:.1$}°S", -lat, precision)
        };
        Text::new(text.as_str())
            .top_right()
            .y(y_pixel)
            .color(conrod_core::color::WHITE)
            .font_size(12)
            .set(ids.latitude_text[i], ui);
    }
    scope_render_latitude.end();

    let scope_render_longitude = crate::profile_scope("Render Longitude");
    //Lines of longitude
    let lng_line_distance =
        line_distance_for_viewport_degrees(viewport.bottom_right.x - viewport.top_left.x, ui.win_w);

    let line_distance_world = world_width_from_longitude(lng_line_distance);
    let lng_start = crate::util::modulo_ceil(
        crate::util::longitude_from_x(viewport.top_left.x.rem_euclid(1.0)),
        lng_line_distance,
    );
    let x_start = crate::util::modulo_ceil(viewport.top_left.x, line_distance_world);

    let lng_lines = ((viewport.bottom_right.x - viewport.top_left.x) / line_distance_world + 1.0)
        .ceil() as usize;

    ids.longitude_lines
        .resize(lng_lines, &mut ui.widget_id_generator());
    ids.longitude_text
        .resize(lng_lines, &mut ui.widget_id_generator());

    let log10_line_distance = lng_line_distance.log10();
    let precision = if log10_line_distance < 0.0 {
        (-log10_line_distance.floor()) as usize
    } else {
        0usize
    };

    //Longitude increases as world x increases
    for i in 0..lng_lines {
        let lng = lng_start + i as f64 * lng_line_distance;
        let world_x = x_start + i as f64 * line_distance_world;
        let x_pixel = world_x_to_pixel_x(world_x, &viewport, ui.win_w);

        let half_height = ui.win_h / 2.0;
        Line::new([x_pixel, -half_height], [x_pixel, half_height])
            .x_y(0.0, 0.0)
            .color(conrod_core::color::BLACK.alpha(LINE_ALPHA))
            .thickness(1.5)
            .set(ids.longitude_lines[i], ui);

        let text = if lng >= 0.0 {
            format!("{:.1$}°E", lng, precision)
        } else {
            format!("{:.1$}°W", -lng, precision)
        };
        Text::new(text.as_str())
            .bottom_right()
            .x(x_pixel)
            .color(conrod_core::color::WHITE)
            .font_size(12)
            .set(ids.longitude_text[i], ui);
    }

    scope_render_longitude.end();
}
