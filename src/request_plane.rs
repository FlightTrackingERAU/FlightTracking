use std::sync::{Arc, Mutex};
use tokio::{runtime::Runtime, time::Instant};

use opensky_api::errors::Error;

use crate::{Airline, PlaneType};

/// The body of a Plane
///
/// Right Now we only care about Long and Lat;
/// It will maybe be bigger depending on things we may like
/// The planes to do.
#[derive(Clone)]
pub struct Plane {
    pub longitude: f32,
    pub latitude: f32,
    pub track: f32,
    pub airline: Airline,
    pub plane_type: PlaneType,
    pub callsign: String,
}
impl Plane {
    ///Constructor on to make a new Plane
    pub fn new(
        longitude: f32,
        latitude: f32,
        track: f32,
        callsign: String,
        airline: Airline,
        plane_type: PlaneType,
    ) -> Self {
        Plane {
            longitude,
            latitude,
            track,
            airline,
            plane_type,
            callsign,
        }
    }
}

pub struct PlaneBody {
    pub planes: Vec<Plane>,
    pub airline: Airline,
    pub plane_type: PlaneType,
}

impl PlaneBody {
    pub fn new(planes: Vec<Plane>, airline: Airline, plane_type: PlaneType) -> Self {
        PlaneBody {
            planes,
            airline,
            plane_type,
        }
    }
    pub fn empty_body() -> Self {
        PlaneBody {
            planes: Vec::new(),
            airline: Airline::Other,
            plane_type: PlaneType::Unknown,
        }
    }
}

///Structure to save te Plane data we request
///We put it into an Arc and Mutex to make it easier to read.
pub struct PlaneRequester {
    planes_storage: Arc<Mutex<Arc<Vec<PlaneBody>>>>,
}

impl PlaneRequester {
    ///Constructor on how to request the plane data.
    pub fn new(runtime: &Runtime) -> Self {
        let planes_storage = Arc::new(Mutex::new(Arc::new(Vec::new())));

        runtime.spawn(plane_data_loop(planes_storage.clone()));

        PlaneRequester { planes_storage }
    }

    ///Returns a clone of the Mutex list of planes.
    pub fn planes_storage(&self) -> Arc<Vec<PlaneBody>> {
        let guard = self.planes_storage.lock().unwrap();
        guard.clone()
    }
}

/// Loop to get plane data.
/// Some math had to be done for the sleeping time.
///
/// The OpenSky Api gets data every 5-6 seconds,
/// the function must also follow that running time.
///
async fn plane_data_loop(list_of_planes: Arc<Mutex<Arc<Vec<PlaneBody>>>>) {
    loop {
        let start = Instant::now();

        match request_plane_data().await {
            Ok(plane_data) => {
                let mut guard = list_of_planes.lock().unwrap();
                *guard = Arc::new(plane_data);
            }
            Err(error) => {
                println!("Error at getting plane data: {:?}", error)
            }
        }

        if let Ok(plane_data) = request_plane_data().await {
            let mut guard = list_of_planes.lock().unwrap();
            *guard = Arc::new(plane_data);
        };

        let end = Instant::now();

        let time_interval = tokio::time::Duration::from_secs(5);
        let seconds = end - start;

        let sleep_time = if seconds <= tokio::time::Duration::from_secs(5) {
            time_interval - seconds
        } else {
            tokio::time::Duration::from_secs(0)
        };

        tokio::time::sleep(sleep_time).await;
    }
}

/// In here we call the OpenSky Api to get the data from planes.
///
/// Request the plane data and makes it into a Vec.
async fn request_plane_data() -> Result<Vec<PlaneBody>, Error> {
    let open_sky = opensky_api::OpenSkyApi::new();

    let state_request = open_sky.get_states();
    let mut list_of_planes: Vec<PlaneBody> = Vec::new();

    let mut spirit_planes: PlaneBody = PlaneBody::empty_body();
    let mut american_al_planes: PlaneBody = PlaneBody::empty_body();
    let mut southwest_planes: PlaneBody = PlaneBody::empty_body();
    let mut united_al_planes: PlaneBody = PlaneBody::empty_body();
    let mut other_planes: PlaneBody = PlaneBody::empty_body();

    let open_sky = state_request.send().await?;
    for state in open_sky.states {
        let longitude = state.longitude;
        let latitude = state.latitude;
        let track = (-state.true_track.unwrap_or(0.0) + 90.0) * (std::f32::consts::PI / 180.0);

        if !state.on_ground {
            if let Some(longitude) = longitude {
                let latitude = latitude.unwrap();

                let (airline, callsign, plane_type) = if let Some(airline) = state.callsign {
                    if airline.len() > 3 {
                        match &airline[0..3] {
                            "NKS" => (Airline::Spirit, airline, PlaneType::Commercial),
                            "AAL" => (Airline::American, airline, PlaneType::Commercial),
                            "SWA" => (Airline::Southwest, airline, PlaneType::Commercial),
                            "UAL" => (Airline::United, airline, PlaneType::Commercial),
                            "DAL" => (Airline::Delta, airline, PlaneType::Commercial),
                            _ => (Airline::Other, airline, PlaneType::Unknown),
                        }
                    } else {
                        (Airline::Other, airline, PlaneType::Unknown)
                    }
                } else {
                    (Airline::Other, String::from("Unknown"), PlaneType::Unknown)
                };

                let plane = Plane {
                    longitude,
                    latitude,
                    track,
                    airline,
                    plane_type,
                    callsign,
                };

                match airline {
                    Airline::Spirit => {
                        spirit_planes.planes.push(plane);
                        spirit_planes.airline = airline;
                        spirit_planes.plane_type = plane_type;
                    }
                    Airline::American => {
                        american_al_planes.planes.push(plane);
                        american_al_planes.airline = airline;
                        american_al_planes.plane_type = plane_type;
                    }
                    Airline::Southwest => {
                        southwest_planes.planes.push(plane);
                        southwest_planes.airline = airline;
                        southwest_planes.plane_type = plane_type;
                    }
                    Airline::United => {
                        united_al_planes.planes.push(plane);
                        united_al_planes.airline = airline;
                        united_al_planes.plane_type = plane_type;
                    }
                    _ => {
                        other_planes.planes.push(plane);
                        other_planes.airline = airline;
                        other_planes.plane_type = plane_type;
                    }
                }
            }
        }
    }

    list_of_planes.push(spirit_planes);
    list_of_planes.push(american_al_planes);
    list_of_planes.push(southwest_planes);
    list_of_planes.push(united_al_planes);
    list_of_planes.push(other_planes);

    Ok(list_of_planes)
}
