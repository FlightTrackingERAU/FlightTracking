use crate::{util, world_x_to_pixel_x, world_y_to_pixel_y, ImageId, PlaneRequester};
use conrod_core::{Colorable, Positionable, Sizeable, UiCell, Widget};

pub fn draw(
    plane_requester: &mut PlaneRequester,
    view: &crate::TileView,
    ids: &mut crate::Ids,
    image_id: ImageId,
    ui: &mut UiCell,
) {
    let planes = plane_requester.planes_storage();
    ids.planes
        .resize(planes.len(), &mut ui.widget_id_generator());

    let viewport = view.get_world_viewport(ui.win_w, ui.win_h);
    let lat_top = crate::util::latitude_from_y(viewport.top_left.y.rem_euclid(1.0)) as f32;
    let lat_bottom = crate::util::latitude_from_y(viewport.bottom_right.y.rem_euclid(1.0)) as f32;
    let long_left = crate::util::longitude_from_x(viewport.top_left.x.rem_euclid(1.0)) as f32;
    let long_right = crate::util::longitude_from_x(viewport.bottom_right.x.rem_euclid(1.0)) as f32;

    for (i, plane) in planes.iter().enumerate() {
        if (plane.latitude > lat_bottom && plane.latitude < lat_top)
            && (plane.longitude > long_left && plane.longitude < long_right)
        {
            let world_x = util::x_from_longitude(plane.longitude as f64);
            let world_y = util::y_from_latitude(plane.latitude as f64);

            let pixel_x = world_x_to_pixel_x(world_x, &viewport, ui.win_w);
            let pixel_y = world_y_to_pixel_y(world_y, &viewport, ui.win_h);

            conrod_core::widget::Image::new(image_id.normal)
                .x_y(pixel_x, pixel_y)
                .w_h(50.0, 50.0)
                .set(ids.planes[i], ui);
        }
    }
}
