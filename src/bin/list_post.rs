use kraken_mesh::cots::{CotsListener, RDoASOI};
use kraken_mesh::krakenrf::KrakenClient;
use kraken_mesh::meshtastic::MeshRadio;
use tokio::time::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() {

    // Send a CoT message to OTak listener for a sensor finding.
    let cot_listener = CotsListener{ endpoint: "192.168.1.1:8088".to_string() };
    let mut publisher = cot_listener.connect().expect("Failed to connect to cots listener.");
    publisher.send_sensor(RDoASOI{ 
        s_contact: "doa-alpha".to_string(), 
        s_fov: 8, 
        s_range: 10000, 
        azimuth: 0,
        uid: "sig-0".to_string() 
    }).expect("Failed to update sensor data.");

    // Send message through mesh network.
    let mesh_radio = MeshRadio{serial_port: "/dev/ttyACM0".to_string()};
    let mut mesh_writer = mesh_radio.connect().await.unwrap();
    mesh_writer.send().await.unwrap();

    /*
    // Remove `None` variants to get the payload variant
    let payload_variant = match mesh_packet.payload_variant {
        Some(payload_variant) => payload_variant,
        None => {
            println!("Received mesh packet with no payload variant, not handling...");
            return Ok(0)
        }
    };

    // Only handle decoded (unencrypted) mesh packets
    let packet_data = match payload_variant {
        ::meshtastic::protobufs::mesh_packet::PayloadVariant::Decoded(decoded_mesh_packet) => {
            decoded_mesh_packet
        }
        ::meshtastic::protobufs::mesh_packet::PayloadVariant::Encrypted(_encrypted_mesh_packet) => {
            println!("Received encrypted mesh packet, not handling...");
            return Ok(0)
        }
    };

    // Meshtastic differentiates mesh packets based on a field called `portnum`.
    // Meshtastic defines a set of standard port numbers [here](https://meshtastic.org/docs/development/firmware/portnum),
    // but also allows for custom port numbers to be used.
    match packet_data.portnum() {
        ::meshtastic::protobufs::PortNum::TextMessageApp => {
            let decoded_text_message = String::from_utf8(packet_data.payload).unwrap();
            println!("Received text message packet: {:?}", decoded_text_message);
        },
        PortNum::AtakPlugin => {
            /* Received tak packet: TakPacket { is_compressed: false, 
            contact: Some(Contact { callsign: "PATAMO", device_callsign: "ANDROID-66ba0811125b0b38" }), 
            group: Some(Group { role: Rto, team: Blue }), status: Some(Status { battery: 36 }), 
            payload_variant: Some(Pli(Pli { latitude_i: 477390670, longitude_i: -1221965750, altitude: 74, speed: 0, course: 226 })) } */
            let tak_packet = TakPacket::decode(packet_data.payload.as_slice()).unwrap();
            println!("Received tak packet: {:?}", tak_packet);
        },
        _ => {
            println!(
                "Received mesh packet on port {:?}, not handling...",
                packet_data.portnum
            );
        }
    }*/

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port.
    /*while let Some(decoded) = decoded_listener.recv().await {
        println!("Received: {:?}", decoded);
        let payload_variant = match decoded.payload_variant {
            Some(payload_variant) => payload_variant,
            None => {
                println!("Received FromRadio packet with no payload variant, not handling...");
                return;
            }
        };
    
        // `FromRadio` packets can be differentiated based on their payload variant,
        // which in Rust is represented as an enum. This means the payload variant
        // can be matched on, and the appropriate user-defined action can be taken.
        match payload_variant {
            ::meshtastic::protobufs::from_radio::PayloadVariant::Channel(channel) => {
                println!("Received channel packet: {:?}", channel);
            }
            ::meshtastic::protobufs::from_radio::PayloadVariant::NodeInfo(node_info) => {
                println!("Received node info packet: {:?}", node_info);
            }
            ::meshtastic::protobufs::from_radio::PayloadVariant::Packet(mesh_packet) => {
                handle_mesh_packet(mesh_packet);
            }
            _ => {
                println!("Received other FromRadio packet, not handling...");
            }
        };
    }*/

    // Get max DoA value from attached Kraken SDR.
    let client = KrakenClient { endpoint: "http://192.168.1.106:8081".to_string()};
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
    }
}
