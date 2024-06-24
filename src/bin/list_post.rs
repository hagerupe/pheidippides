use kraken_mesh::cots::{CotsListener, RDoASOI};
use kraken_mesh::krakenrf::KrakenClient;
use kraken_mesh::meshtastic::{DoASensorState, MeshRadio};
use tokio::time::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() {

    // Send message through mesh network.
    let mesh_radio = MeshRadio{serial_port: "/dev/ttyACM0".to_string()};
    let mut mesh_writer = mesh_radio.connect().await.unwrap();

    let mut i: u32 = 0;
    loop {
        mesh_writer.send(DoASensorState{azimuth: i}).await.unwrap();
        i += 1;
        i = i%360;
    }

    mesh_writer.send().await.unwrap();

    // Get max DoA value from attached Kraken SDR.
    /*let client = KrakenClient { endpoint: "http://192.168.1.106:8081".to_string()};
    loop {
        let val = client.get_doa().await.expect("Failed to acquire DOA sample");
        let index_of_max: Option<usize> = val.values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index);
        match index_of_max {
            Some(index) => println!("{}@{}", index, val.values[index] ),
            None => println!("No DOA sample"),
        }
        sleep(Duration::new(5, 0)).await;
    }*/
}
