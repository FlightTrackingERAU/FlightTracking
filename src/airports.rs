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
    let airports = bincode::deserialize(bytes)?;

    Ok(airports)
}

/// Useful functions for rendering airports on the map
pub mod airport_renderer {
    use conrod_core::UiCell;

    use crate::Airport;

    /// Draws all of the airports onto the map. Should be run before plane rendering, but after the
    /// map tiles are rendered
    pub fn draw(
        airports: &Vec<Airport>,
        view: &crate::map::TileView,
        _display: &glium::Display,
        ids: &mut crate::Ids,
        ui: &mut UiCell,
    ) {
        let _viewport = view.get_world_viewport(ui.win_w, ui.win_h);

        ids.airports
            .resize(airports.len(), &mut ui.widget_id_generator());

        for _airport in airports {
            // Render airports
        }
    }
}
