use conrod_core::{
    widget::{Canvas, Rectangle, Text},
    Colorable, Positionable, UiCell, Widget,
};

pub fn draw(view: &crate::map::TileView, ids: &mut crate::Ids, ui: &mut UiCell) {
    let mut it = view.tile_iter(256, ui.win_w as u32, ui.win_h as u32);
    let size = it.tile_size;
    let offset = it.tile_offset;

    let tiles: Vec<_> = it.collect();

    Canvas::new().pad(0.0).set(ids.viewport, ui);

    ids.squares
        .resize(tiles.len(), &mut ui.widget_id_generator());
    ids.square_text
        .resize(tiles.len(), &mut ui.widget_id_generator());
    for (i, tile) in tiles.into_iter().enumerate() {
        let id = ids.squares[i];
        Rectangle::outline(size.to_array())
            .top_left_of(ids.viewport)
            .set(id, ui);

        let text = format!("TEST",);
        Text::new(text.as_str())
            .xy_relative([0.0, 0.0])
            .color(conrod_core::color::WHITE)
            .font_size(12)
            .set(ids.square_text[i], ui);
    }
}
