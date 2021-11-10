use serde::Deserialize;

/// Represents an Airport that will be deserialized
#[derive(Debug, Deserialize)]
pub struct Airport {
    pub id: u32,
    pub ident: String,
    pub airport_type: String,
    pub name: String,
    pub latitude: f32,
    pub longitude: f32,
    pub elevation: i32,
    pub continent: String,
    pub country_name: String,
    pub iso_country: String,
    pub region_name: String,
    pub iso_region: String,
    pub local_region: String,
    pub municipality: String,
    pub scheduled_service: bool,
    pub gps_code: String,
    pub iata_code: String,
    pub local_code: String,
}

/// Deserializes a Vec<Airport> from a &[u8] using serde Postcard
pub fn airports_from_bytes(bytes: &[u8]) -> Result<Vec<Airport>, Box<bincode::ErrorKind>> {
    // Deserialize all of the airports
    let airports: Vec<Airport> = bincode::deserialize(bytes)?;

    let airports = airports
        .into_iter()
        .filter(|airport| match airport.airport_type.as_str() {
            "medium_airport" | "large_airport" => true,
            _ => false,
        })
        .collect();

    Ok(airports)
}

/// Useful functions for rendering airports on the map
pub mod airport_renderer {
    use conrod_core::{Positionable, Sizeable, UiCell, Widget};
    use num::Float;

    use crate::Airport;

    /// Draws all of the airports onto the map. Should be run before plane rendering, but after the
    /// map tiles are rendered
    pub fn draw(
        airports: &[Airport],
        view: &crate::map::TileView,
        _display: &glium::Display,
        ids: &mut crate::Ids,
        image_id: crate::ImageId,
        ui: &mut UiCell,
    ) {
        let viewport = view.get_world_viewport(ui.win_w, ui.win_h);

        let zoom = view.get_zoom();

        ids.airports
            .resize(airports.len(), &mut ui.widget_id_generator());

        let lat_top = crate::util::latitude_from_y(viewport.top_left.y.rem_euclid(1.0)) as f32;
        let lat_bottom =
            crate::util::latitude_from_y(viewport.bottom_right.y.rem_euclid(1.0)) as f32;
        let long_left = crate::util::longitude_from_x(viewport.top_left.x.rem_euclid(1.0)) as f32;
        let long_right =
            crate::util::longitude_from_x(viewport.bottom_right.x.rem_euclid(1.0)) as f32;

        for (i, airport) in airports.iter().enumerate() {
            if (airport.latitude > lat_bottom && airport.latitude < lat_top)
                && (airport.longitude > long_left && airport.longitude < long_right)
            {
                // Render airports
                let world_x = crate::util::x_from_longitude(airport.longitude as f64);
                let world_y = crate::util::y_from_latitude(airport.latitude as f64);

                let pixel_x = crate::world_x_to_pixel_x(world_x, &viewport, ui.win_w);
                let pixel_y = crate::world_y_to_pixel_y(world_y, &viewport, ui.win_h);

                let size = 1.5.powf(zoom) / 100.0;
                conrod_core::widget::Image::new(image_id.normal)
                    .x_y(pixel_x, pixel_y)
                    .w_h(size, size)
                    .set(ids.airports[i], ui);
            }
        }
    }
}
