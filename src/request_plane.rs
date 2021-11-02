use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

use opensky_api::errors::Error;

#[derive(Debug)]
pub struct Plane {
    pub longitude: f32,
    pub latitude: f32,
    pub on_ground: bool,
}
impl Plane {
    pub fn new(longitude: f32, latitude: f32, on_ground: bool) -> Self {
        Plane {
            longitude,
            latitude,
            on_ground,
        }
    }
}

pub struct PlaneRequester {
    planes_storage: Arc<Mutex<Arc<Vec<Plane>>>>,
}

impl PlaneRequester {
    pub fn new(runtime: &Runtime) -> Self {
        let planes_storage = Arc::new(Mutex::new(Arc::new(Vec::new())));

        runtime.spawn(plane_data_loop(planes_storage.clone()));

        PlaneRequester { planes_storage }
    }

    pub fn planes_storage(&self) -> Arc<Vec<Plane>> {
        let guard = self.planes_storage.lock().unwrap();
        guard.clone()
    }
}

async fn plane_data_loop(list_of_planes: Arc<Mutex<Arc<Vec<Plane>>>>) {
    loop {
        match request_plane_data().await {
            Ok(plane_data) => {
                let mut guard = list_of_planes.lock().unwrap();
                *guard = Arc::new(plane_data);
            }
            Err(_) => {
                println!("Error at getting plane data")
            }
        };

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

//Request the plane data and makes it into a Vec
async fn request_plane_data() -> Result<Vec<Plane>, Error> {
    let open_sky = opensky_api::OpenSkyApi::new();

    let state_request = open_sky.get_states();

    let open_sky = state_request.send().await?;
    let mut plane_list = Vec::new();
    for state in open_sky.states {
        let longitude = state.longitude;
        let latitude = state.latitude;
        let on_ground = state.on_ground;

        if let Some(longitude) = longitude {
            let latitude = latitude.unwrap();

            let plane = Plane {
                longitude,
                latitude,
                on_ground,
            };
            plane_list.push(plane);
        }
    }
    Ok(plane_list)
}

mod plane_test {

    #[tokio::test]
    async fn request_plane_list() {
        use super::request_plane_data;

        let _plane_list = request_plane_data().await.unwrap();
    }
}
