use std::net::SocketAddr;
use std::thread::sleep;
use env_logger;
use log::{error, info};

use xplane_udp::dataref_type::{DataRefType, DataRefValueType};
use xplane_udp::session::Session;

fn main() {
    env_logger::init();
    // let session = Session::auto_discover_default(10000);
    let session = Session::manual(SocketAddr::from(([10, 0, 0, 10], 49000)),
                                   SocketAddr::from(([10, 0, 0, 10], 49001)));
    let mut session = match session {
        Ok(session) => {
            session
        },
        Err(e) => {
            error!("Failed to auto-discover X-Plane: {}", e);
            return;
        }
    };

    info!("Intercepting X-Plane beacon messages");
    let _ = match session.connect() {
        Ok(conn) => {
            conn
        },
        Err(e) => {
            error!("Failed to connect to X-Plane: {}", e);
            return;
        }
    };

    session.run();

    match session.subscribe("sim/aircraft/engine/acf_num_engines", 1, DataRefType::Int) {
        Ok(_) => {
            info!("Subscribed to sim/aircraft/engine/acf_num_engines");
        },
        Err(e) => {
            error!("Failed to subscribe to sim/aircraft/engine/acf_num_engines: {}", e);
        }
    }

    match session.subscribe("laminar/B738/toggle_switch/cockpit_dome_pos", 1, DataRefType::Int) {
        Ok(_) => {
            info!("Subscribed to laminar/B738/toggle_switch/cockpit_dome_pos");
        },
        Err(e) => {
            error!("Failed to subscribe to laminar/B738/toggle_switch/cockpit_dome_pos: {}", e);
        }
    }

    let loop_count = 10;

    for _ in 0..loop_count {
        let num_engines = session.get_dataref("sim/aircraft/engine/acf_num_engines");
        match num_engines {
            Some(DataRefValueType::Int(num_engines)) => {
                info!("Number of engines: {}", num_engines);
            },
            _ => {
                error!("Failed to get number of engines");
            }
        }
        let dome = session.get_dataref("laminar/B738/toggle_switch/cockpit_dome_pos");
        match dome {
            Some(DataRefValueType::Int(dome)) => {
                match dome {
                    -1 => {
                        session.cmd("laminar/B738/toggle_switch/cockpit_dome_up").unwrap();
                        info!("Dome: {:?}", dome);
                    },
                    0 => {
                        session.cmd("laminar/B738/toggle_switch/cockpit_dome_up").unwrap();
                        info!("Dome: {:?}", dome);
                    },
                    1 => {
                        session.cmd("laminar/B738/toggle_switch/cockpit_dome_dn").unwrap();
                        info!("Dome: {:?}", dome);
                    },
                    _ => {
                        info!("Unknown dome value: {:?}", dome);
                    }
                }
            },
            _ => {
                error!("Failed to get dome position");
            }
        }
        sleep(std::time::Duration::from_secs(1));
    }

    info!("Shutting down")
}
