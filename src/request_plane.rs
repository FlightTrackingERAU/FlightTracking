use std::sync::{Arc, Mutex};
use tokio::{runtime::Runtime, time::Instant};

use opensky_api::errors::Error;
pub struct PlaneAirlines {
    //Code is NKS
    pub spirit: Vec<Plane>,
    //Code is AAL
    pub american_al: Vec<Plane>,

    //Code is SWA
    pub southwest: Vec<Plane>,

    //Code is UAL
    pub united_al: Vec<Plane>,

    pub any_airline: Vec<Plane>,
}

impl Default for PlaneAirlines {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaneAirlines {
    pub fn new() -> Self {
        PlaneAirlines {
            spirit: Vec::new(),
            american_al: Vec::new(),
            southwest: Vec::new(),
            united_al: Vec::new(),
            any_airline: Vec::new(),
        }
    }

    pub fn total_airlines(&self) -> usize {
        self.american_al.len()
            + self.spirit.len()
            + self.southwest.len()
            + self.united_al.len()
            + self.any_airline.len()
    }

    pub fn all_airlines(&self) -> Vec<&Vec<Plane>> {
        vec![
            &self.american_al,
            &self.spirit,
            &self.southwest,
            &self.united_al,
            &self.any_airline,
        ]
    }
}

///The body of a Plane
///
///Right Now we only care about Long and Lat;
///It will maybe be bigger depending on things we may like
///The planes to do.
#[derive(Clone)]
pub struct Plane {
    pub longitude: f32,
    pub latitude: f32,
}
impl Plane {
    ///Constructor on to make a new Plane
    pub fn new(longitude: f32, latitude: f32) -> Self {
        Plane {
            longitude,
            latitude,
        }
    }
}

///Structure to save te Plane data we request
///We put it into an Arc and Mutex to make it easier to read.
pub struct PlaneRequester {
    planes_storage: Arc<Mutex<Arc<PlaneAirlines>>>,
}

impl PlaneRequester {
    ///Constructor on how to request the plane data.
    pub fn new(runtime: &Runtime) -> Self {
        let planes_storage = Arc::new(Mutex::new(Arc::new(PlaneAirlines::new())));

        runtime.spawn(plane_data_loop(planes_storage.clone()));

        PlaneRequester { planes_storage }
    }

    ///Returns a clone of the Mutex list of planes.
    pub fn planes_storage(&self) -> Arc<PlaneAirlines> {
        let guard = self.planes_storage.lock().unwrap();
        guard.clone()
    }
}

///Loop to get plane data.
///Some math had to be done for the sleeping time.
///
///The OpenSky Api gets data every 5-6 seconds,
///the function must also follow that running time.
///
async fn plane_data_loop(list_of_planes: Arc<Mutex<Arc<PlaneAirlines>>>) {
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

///In here we call the OpenSky Api to get the data from planes.
///
///Request the plane data and makes it into a Vec.
async fn request_plane_data() -> Result<PlaneAirlines, Error> {
    let open_sky = opensky_api::OpenSkyApi::new();

    let state_request = open_sky.get_states();
    let mut plane_airlines = PlaneAirlines::new();

    let open_sky = state_request.send().await?;
    for state in open_sky.states {
        let longitude = state.longitude;
        let latitude = state.latitude;

        if !state.on_ground {
            if let Some(longitude) = longitude {
                let latitude = latitude.unwrap();

                let plane = Plane {
                    longitude,
                    latitude,
                };
                if let Some(airline) = state.callsign {
                    if airline.len() > 3 {
                        match &airline[0..3] {
                            "NKS" => plane_airlines.spirit.push(plane),
                            "AAL" => plane_airlines.american_al.push(plane),
                            "SWA" => plane_airlines.southwest.push(plane),
                            "UAL" => plane_airlines.united_al.push(plane),
                            _ => plane_airlines.any_airline.push(plane),
                        }
                    }
                }
            }
        } else {
        }
    }

    Ok(plane_airlines)
}
