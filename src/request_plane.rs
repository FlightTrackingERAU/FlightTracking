use std::sync::{Arc, Mutex};
use tokio::{runtime::Runtime, time::Instant};

use opensky_api::errors::Error;

///The body of a Plane
///
///Right Now we only care about Long and Lat;
///It will maybe be bigger depending on things we may like
///The planes to do.
#[derive(Debug)]
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
    planes_storage: Arc<Mutex<Arc<Vec<Plane>>>>,
}

impl PlaneRequester {
    ///Constructor on how to request the plane data.
    pub fn new(runtime: &Runtime) -> Self {
        let planes_storage = Arc::new(Mutex::new(Arc::new(Vec::new())));

        runtime.spawn(plane_data_loop(planes_storage.clone()));

        PlaneRequester { planes_storage }
    }

    ///Returns a clone of the Mutex list of planes.
    pub fn planes_storage(&self) -> Arc<Vec<Plane>> {
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
async fn plane_data_loop(list_of_planes: Arc<Mutex<Arc<Vec<Plane>>>>) {
    loop {
        let start = Instant::now();
        match request_plane_data().await {
            Ok(plane_data) => {
                let mut guard = list_of_planes.lock().unwrap();
                *guard = Arc::new(plane_data);
            }
            Err(_) => {
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
async fn request_plane_data() -> Result<Vec<Plane>, Error> {
    let open_sky = opensky_api::OpenSkyApi::new();

    let state_request = open_sky.get_states();

    let open_sky = state_request.send().await?;
    let mut plane_list = Vec::new();
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
                plane_list.push(plane);
            }
        } else {
        }
    }

    Ok(plane_list)
}
