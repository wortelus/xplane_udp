use std::io;
use std::net::SocketAddr;
use std::thread::sleep;
use env_logger;
use log::{error, info};

use xplane_udp::auto_discover::AutoDiscover;
use xplane_udp::dataref_type::{DataRefType, DataRefValueType};
use xplane_udp::session::Session;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    
    let auto_discover = AutoDiscover::auto_discover_default(10000).await;
    let auto_discover = match auto_discover {
        Ok(auto_discover) => {
            info!("Auto-discovered X-Plane");
            auto_discover
        }
        Err(e) => {
            error!("Failed to auto-discover X-Plane: {}", e);
            return Err(e);
        }
    };

    let mut session = Session::intercept_beacon(auto_discover).await;
    let mut session = match session {
        Ok(session) => {
            session
        }
        Err((e, _)) => {
            error!("Failed to intercept X-Plane: {}", e);
            return Err(e);
        }
    };

    session.run().await?;

    match session.subscribe("sim/aircraft/engine/acf_num_engines", 1, DataRefType::Int).await {
        Ok(_) => {
            info!("Subscribed to sim/aircraft/engine/acf_num_engines");
        }
        Err(e) => {
            error!("Failed to subscribe to sim/aircraft/engine/acf_num_engines: {}", e);
        }
    }

    match session.subscribe("laminar/B738/toggle_switch/cockpit_dome_pos", 1, DataRefType::Int).await {
        Ok(_) => {
            info!("Subscribed to laminar/B738/toggle_switch/cockpit_dome_pos");
        }
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
            }
            _ => {
                error!("Failed to get number of engines");
            }
        }
        let dome = session.get_dataref("laminar/B738/toggle_switch/cockpit_dome_pos");
        match dome {
            Some(DataRefValueType::Int(dome)) => {
                match dome {
                    -1 => {
                        session.cmd("laminar/B738/toggle_switch/cockpit_dome_up").await?;
                        info!("Dome: {:?}", dome);
                    }
                    0 => {
                        session.cmd("laminar/B738/toggle_switch/cockpit_dome_up").await?;
                        info!("Dome: {:?}", dome);
                    }
                    1 => {
                        session.cmd("laminar/B738/toggle_switch/cockpit_dome_dn").await?;
                        info!("Dome: {:?}", dome);
                    }
                    _ => {
                        info!("Unknown dome value: {:?}", dome);
                    }
                }
            }
            _ => {
                error!("Failed to get dome position");
            }
        }
        sleep(std::time::Duration::from_secs(1));
    }

    info!("Shutting down");
    session.shutdown().await;
    Ok(())
}

