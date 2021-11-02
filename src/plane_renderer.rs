use crate::{util, world_x_to_pixel_x, world_y_to_pixel_y, PlaneRequester};
use conrod_core::{
    widget::{Image, Line, Text},
    Colorable, Positionable, Sizeable, UiCell, Widget,
};

pub fn draw(
    plane_requester: &mut PlaneRequester,
    view: &crate::TileView,
    display: &glium::Display,
    image_map: &mut conrod_core::image::Map<glium::Texture2d>,
    ids: &mut crate::Ids,
    ui: &mut UiCell,
) {
    let planes = plane_requester.planes_storage();
    ids.planes
        .resize(planes.len(), &mut ui.widget_id_generator());

    let viewport = view.get_world_viewport(ui.win_w, ui.win_h);
    for (i, plane) in planes.iter().enumerate() {
        let world_x = util::x_from_longitude(plane.longitude as f64);
        let world_y = util::y_from_latitude(plane.latitude as f64);

        let pixel_x = world_x_to_pixel_x(world_x, &viewport, ui.win_w);
        let pixel_y = world_y_to_pixel_y(world_y, &viewport, ui.win_h);

        let rect_fil = [50.0; 2];

        conrod_core::widget::Rectangle::fill(rect_fil)
            .x_y(pixel_x, pixel_y)
            .w_h(20.0, 20.0)
            .color(conrod_core::color::BLACK)
            .set(ids.planes[i], ui)
    }
}
