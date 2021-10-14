use conrod_core::{
    widget::{Canvas, Rectangle, Text},
    Colorable, Positionable, UiCell, Widget,
};

pub fn draw(view: &crate::map::TileView, ids: &mut crate::Ids, ui: &mut UiCell) {
    let mut it = view.tile_iter(256, ui.win_w as u32, ui.win_h as u32);
    let size = it.tile_size;
    let offset = it.tile_offset;

    let tiles_vertically = it.tiles_vertically;
    let tiles_horizontally = it.tiles_horizontally;

    let tiles: Vec<_> = it.collect();

    Canvas::new().pad(0.0).set(ids.viewport, ui);

    ids.squares
        .resize(tiles.len(), &mut ui.widget_id_generator());
    ids.square_text
        .resize(tiles.len(), &mut ui.widget_id_generator());

    for (i, tile) in tiles.into_iter().enumerate() {
        let id = ids.squares[i];
        let tile_x = i / tiles_vertically as usize;
        let tile_y = i % tiles_vertically as usize;

        let x = offset.x + tile_x as f64 * size.x;
        let y = offset.y - (tile_y as f64 * size.y);
        Rectangle::outline(size.to_array()).x(x).y(y).set(id, ui);

        let text = format!("[{}, {}]", tile.0, tile.1);
        Text::new(text.as_str())
            .xy_relative([0.0, 0.0])
            .color(conrod_core::color::WHITE)
            .font_size(12)
            .set(ids.square_text[i], ui);
    }
}
