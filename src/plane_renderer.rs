use enum_map::*;

use crate::{
    util, world_x_to_pixel_x, world_y_to_pixel_y, ImageId, Plane, PlaneRequester, WorldViewport,
};
use conrod_core::{Positionable, Sizeable, UiCell, Widget};
use image::RgbaImage;
use num::Float;

///Draw the plane on conrod ui
pub fn draw(
    plane_requester: &mut PlaneRequester,
    airline: Airlines,
    view: &crate::TileView,
    ids: &mut crate::Ids,
    image_id: crate::ImageId,
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

pub fn make_color_planes(
    display: &glium::Display,
    image_map: &mut conrod_core::image::Map<glium::Texture2d>,
    color: image::Rgba<u8>,
) -> ImageId {
    let image_bytes = include_bytes!("../assets/images/airplane-image.png");
    let airplane_image = image::load_from_memory(image_bytes).unwrap().to_rgba();
    let mut new_image = RgbaImage::new(airplane_image.width(), airplane_image.height());
    for (x, y, pixel) in airplane_image.enumerate_pixels() {
        let mut pixel: [u8; 4] = pixel.0;

        pixel[0] = color[0];
        pixel[1] = color[1];
        pixel[2] = color[2];

        new_image.put_pixel(x, y, image::Rgba(pixel));
    }

    let image_dimensions = new_image.dimensions();
    let raw_image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&new_image.into_raw(), image_dimensions);
    let image = glium::texture::Texture2d::new(display, raw_image).unwrap();

    ImageId {
        normal: image_map.insert(image),
        hover: image_map.insert(crate::load_image(display, image_bytes)),
        press: image_map.insert(crate::load_image(display, image_bytes)),
    }
}

///Returns the image ids depending
///on what airline was chosen
pub fn airlines_ids(
    display: &glium::Display,
    image_map: &mut conrod_core::image::Map<glium::Texture2d>,
) -> EnumMap<Airlines, ImageId> {
    enum_map! {
        Airlines::AmericanAL => {
            let american_al_color = image::Rgba([3, 5, 135, 0]);
            make_color_planes(display, image_map, american_al_color)
        },
        Airlines::Spirit => {
            let spirit_color = image::Rgba([255,255,0,1]);
            make_color_planes(display, image_map,spirit_color)}

        Airlines::United => {
            let united_color = image::Rgba([146, 182, 240,1]);
            make_color_planes(display, image_map, united_color)
        }
        Airlines::SouthWest => {
            let southwest = image::Rgba([229, 29, 35,1]);
            make_color_planes(display, image_map, southwest)
        }

        Airlines::All | Airlines::Other => {
            let black = image::Rgba([0,0,0,1]);
            make_color_planes(display, image_map, black)
        }
    }
}

///Different type of airlines
#[derive(Copy, Clone, Debug, Enum)]
pub enum Airlines {
    AmericanAL,
    Spirit,
    SouthWest,
    United,
    All,
    Other,
}
