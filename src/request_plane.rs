use std::sync::{Arc, Mutex};
use tokio::{runtime::Runtime, time::Instant};

use opensky_api::errors::Error;

use crate::{Airline, BasicAirline, DynamicAirline, PlaneType};

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
    pub fn empty_commercial(airline: Airline) -> Self {
        PlaneBody {
            planes: Vec::new(),
            airline,
            plane_type: PlaneType::Commercial,
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

    let mut spirit_planes: PlaneBody = PlaneBody::empty_commercial(BasicAirline::Spirit.into());
    let mut american_al_planes: PlaneBody =
        PlaneBody::empty_commercial(BasicAirline::American.into());
    let mut southwest_planes: PlaneBody =
        PlaneBody::empty_commercial(BasicAirline::Southwest.into());
    let mut united_al_planes: PlaneBody = PlaneBody::empty_commercial(BasicAirline::United.into());
    let mut other_planes: PlaneBody = PlaneBody::empty_commercial(Airline::Unknown);

    let dynamic_plane_types = get_dynamic_plane_types();

    let open_sky = state_request.send().await?;
    for state in open_sky.states {
        let longitude = state.longitude;
        let latitude = state.latitude;
        let track = (-state.true_track.unwrap_or(0.0) + 90.0) * (std::f32::consts::PI / 180.0);

        if !state.on_ground {
            if let Some(longitude) = longitude {
                let latitude = latitude.unwrap();
                let mut maybe_airline = None;
                let mut maybe_callsign = None;
                let mut maybe_plane_type = None;

                if let Some(callsign) = &state.callsign {
                    maybe_callsign = Some(callsign.clone());
                    if callsign.len() > 3 {
                        let callsign_header = &callsign[0..3];
                        match callsign_header {
                            "NKS" => maybe_airline = Some(BasicAirline::Spirit.into()),
                            "AAL" => maybe_airline = Some(BasicAirline::American.into()),
                            "SWA" => maybe_airline = Some(BasicAirline::Southwest.into()),
                            "UAL" => maybe_airline = Some(BasicAirline::United.into()),
                            "DAL" => maybe_airline = Some(BasicAirline::Delta.into()),
                            _ => {
                                //Try to match dynamic airlines
                                for (dyn_airline, dyn_plane_type) in &dynamic_plane_types {
                                    // println!(
                                    //     "Matched callsign: {} - {}",
                                    //     callsign_header, callsign
                                    // );
                                    if dyn_airline.callsign == callsign_header {
                                        maybe_airline = Some(Airline::Dynamic(dyn_airline.clone()));
                                        maybe_plane_type = Some(*dyn_plane_type);
                                    }
                                }
                            }
                        }
                    }
                }

                let plane_type = match (maybe_plane_type, &maybe_airline) {
                    (Some(plane_type), _) => plane_type,
                    (_, Some(Airline::Basic(_))) => PlaneType::Commercial,
                    _ => PlaneType::Unknown,
                };
                let plane = Plane {
                    longitude,
                    latitude,
                    track,
                    airline: maybe_airline.clone().unwrap_or(Airline::Unknown),
                    //Default to commercial because we only set it in the case of spirit, american etc.
                    plane_type,
                    callsign: maybe_callsign.unwrap_or("Unknown".to_owned()),
                };

                match maybe_airline {
                    Some(Airline::Basic(BasicAirline::Spirit)) => spirit_planes.planes.push(plane),
                    Some(Airline::Basic(BasicAirline::American)) => {
                        american_al_planes.planes.push(plane)
                    }
                    Some(Airline::Basic(BasicAirline::Southwest)) => {
                        southwest_planes.planes.push(plane)
                    }
                    Some(Airline::Basic(BasicAirline::United)) => {
                        united_al_planes.planes.push(plane)
                    }
                    _ => other_planes.planes.push(plane),
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

fn get_dynamic_plane_types() -> Vec<(DynamicAirline, PlaneType)> {
    let mut result = Vec::new();

    let data = r#"
ATN - Air Transport International - cargo
ASA - Alaska Airlines - airline
AAY - Allegiant Air - airline
AIP - Alpine Air Express - cargo
AAL - American Airlines - airline
AMF - Ameriflight - airline
AJT - Amerijet International - airline
GTI - Atlas Air - airline
DAL - Delta Air Lines - airline
ASQ - ExpressJet - cargo
FDX - FedEx Express - cargo
FFT - Frontier Airlines - airline
HAL - Hawaiian Airlines - airline
SWQ - iAero Airways - airline
JBU - JetBlue - airline
SKW - SkyWest Airlines - airline
SOO - Southern Air - airline
SWA - Southwest Airlines - airline
NKS - Spirit Airlines - airline
UAL - United Airlines - airline
UPS - UPS Airlines - airline
CAL - China Airlines - airline
BAW - British Airways - airline
DLH - Lufthansa - airline
ACA - Air Canada - airline
VRD - Virgin America - airline
VIR - Virgin Atlantic - airline
AFR - Air France - airline
EGF - American Eagle Airlines - airline
IBK - Norwegian Air International - airline
AMX - Aeromexico - airline
ERU - Embry_Riddle - trainer
SCX - Sun Country Airlines - airline
VXP - Avelo Airlines - airline
EJA - NetJets - business
RPA - Republic Airlines - airline
"#;
    for line in data.lines() {
        if line.is_empty() {
            continue;
        }
        let mut split = line.split('-');
        let callsign = split.next().unwrap().trim();

        //Replace the dash in Embry-Riddle _after_we split by `-`
        let airline_name = split.next().unwrap().trim().replace('_', "-");
        let type_str = split.next().unwrap().trim();
        let plane_type = match type_str {
            "airline" => PlaneType::Commercial,
            "cargo" => PlaneType::Cargo,
            "trainer" => PlaneType::Trainer,
            "business" => PlaneType::Business,
            s => unreachable!(s),
        };

        result.push((
            DynamicAirline {
                callsign: callsign.to_owned(),
                name: airline_name.to_owned(),
            },
            plane_type,
        ));
    }

    result
}
