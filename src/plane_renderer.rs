use crate::{
    util, world_x_to_pixel_x, world_y_to_pixel_y, ImageId, Plane, PlaneRequester, WorldViewport,
};
use conrod_core::{Positionable, Sizeable, UiCell, Widget};
use num::Float;

///Draw the plane on conrod ui
pub fn draw(
    plane_requester: &mut PlaneRequester,
    airline: Airlines,
    view: &crate::TileView,
    ids: &mut crate::Ids,
    image_id: ImageId,
    ui: &mut UiCell,
) {
    //From PlaneRequester gets all the planes we get from the Mutex
    let planes = plane_requester.planes_storage();

    ids.planes
        .resize(planes.total_airlines(), &mut ui.widget_id_generator());

    let all_airlines = planes.all_airlines();

    //ViewPort of the world
    let viewport = view.get_world_viewport(ui.win_w, ui.win_h);

    //Takes the zoom value
    let zoom = view.get_zoom();
    //From the widget_ids we iter thru all the planes and
    //display into UI.
    let mut airlines = Vec::with_capacity(1);

    let all_airlines = match airline {
        Airlines::All => all_airlines,
        other => {
            let planes = match other {
                Airlines::AmericanAL => &planes.american_al,
                Airlines::Spirit => &planes.spirit,
                Airlines::SouthWest => &planes.southwest,
                Airlines::United => &planes.united_al,
                Airlines::Other => &planes.any_airline,
                _ => unreachable!(),
            };

            airlines.push(planes);

            airlines
        }
    };

    draw_plane_loop(&all_airlines, viewport, zoom, image_id, ids, ui);
}

///Function to draw planes
///according to airline given.
fn draw_plane_loop(
    airlines: &[&Vec<Plane>],
    viewport: WorldViewport,
    zoom: f64,
    image_id: ImageId,
    ids: &mut crate::Ids,
    ui: &mut UiCell,
) {
    let lat_top = crate::util::latitude_from_y(viewport.top_left.y.rem_euclid(1.0)) as f32;
    let lat_bottom = crate::util::latitude_from_y(viewport.bottom_right.y.rem_euclid(1.0)) as f32;
    let long_left = crate::util::longitude_from_x(viewport.top_left.x.rem_euclid(1.0)) as f32;
    let long_right = crate::util::longitude_from_x(viewport.bottom_right.x.rem_euclid(1.0)) as f32;

    for airline in airlines.iter() {
        for (i, plane) in airline.iter().enumerate() {
            if (plane.latitude > lat_bottom && plane.latitude < lat_top)
                && (plane.longitude > long_left && plane.longitude < long_right)
            {
                //Translates real world coordinates to
                //pixel coordinates.
                let world_x = util::x_from_longitude(plane.longitude as f64);
                let world_y = util::y_from_latitude(plane.latitude as f64);

                let pixel_x = world_x_to_pixel_x(world_x, &viewport, ui.win_w);
                let pixel_y = world_y_to_pixel_y(world_y, &viewport, ui.win_h);

                let size_of_plane = 1.5.powf(zoom) / 30.0;
                //Draw images of the planes.
                conrod_core::widget::Image::new(image_id.normal)
                    .x_y(pixel_x, pixel_y)
                    .w_h(size_of_plane, size_of_plane)
                    .set(ids.planes[i], ui);
            }
        }
    }
}

///Different type of airlines
#[derive(Copy, Clone)]
pub enum Airlines {
    AmericanAL,
    Spirit,
    SouthWest,
    United,
    All,
    Other,
}
