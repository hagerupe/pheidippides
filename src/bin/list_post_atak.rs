use kraken_mesh::cots::{CotsListener, RDoASOI};
use kraken_mesh::meshtastic::MeshRadio;
use ::meshtastic::protobufs::from_radio::PayloadVariant::Packet;
use meshtastic::protobufs::MeshPacket;
use meshtastic::protobufs::mesh_packet::PayloadVariant;

fn handle_packet_from_radio(mesh_packet: MeshPacket) -> Result<Option<String>, std::io::Error> {

    // Remove `None` variants to get the payload variant
    let payload_variant = match mesh_packet.payload_variant {
        Some(payload_variant) => payload_variant,
        None => {
            println!("Received mesh packet with no payload variant, not handling...");
            return Ok(None)
        }
    };

    // Only handle decoded (unencrypted) mesh packets
    let packet_data = match payload_variant {
        PayloadVariant::Decoded(decoded_mesh_packet) => {
            decoded_mesh_packet
        }
        PayloadVariant::Encrypted(_encrypted_mesh_packet) => {
            println!("Received encrypted mesh packet, not handling...");
            return Ok(None)
        }
    };

    // Meshtastic differentiates mesh packets based on a field called `portnum`.
    // Meshtastic defines a set of standard port numbers [here](https://meshtastic.org/docs/development/firmware/portnum),
    // but also allows for custom port numbers to be used.
    //
    // Fixme: move this over to a custom port.  For now just sending text messages.
    match packet_data.portnum() {
        ::meshtastic::protobufs::PortNum::TextMessageApp => {
            let decoded_text_message = String::from_utf8(packet_data.payload).unwrap();
            println!("Received text message packet: {:?}", decoded_text_message);
            return Ok(Some(decoded_text_message))
        },
        _ => {
            println!(
                "Received mesh packet on port {:?}, not handling...",
                packet_data.portnum
            );
        }
    }
    Ok(None)
}

#[tokio::main]
async fn main() {

    // Send a CoT message to OTak listener for a sensor finding.
    let cot_listener = CotsListener{ endpoint: "192.168.1.1:8088".to_string() };
    let mut publisher = cot_listener.connect().expect("Failed to connect to cots listener.");

    // Connect to mesh device.
    let mesh_radio = MeshRadio{serial_port: "/dev/ttyUSB0".to_string()};
    let mut mesh_writer = mesh_radio.connect().await.unwrap();
    while let Some(decoded) = mesh_writer.decoded_listener.recv().await {
        println!("Received: {:?}", decoded);
        let payload_variant = match decoded.payload_variant {
            Some(payload_variant) => payload_variant,
            None => {
                println!("Received FromRadio packet with no payload variant, not handling...");
                return;
            }
        };
    
        // Filter for mesh packet payloads only.
        let message : Option<String>;
        match payload_variant {
            Packet(mesh_packet) => {
                message = handle_packet_from_radio(mesh_packet).unwrap();
            }
            _ => {
                println!("Received other FromRadio packet, not handling...");
                message = None;
            }
        };

        // Handle mesh message.
        match message {
            Some(message) => {
                publisher.send_sensor(RDoASOI{ 
                    s_contact: "doa-alpha".to_string(), 
                    s_fov: 8, 
                    s_range: 10000, 
                    azimuth: message.parse().unwrap(),
                    uid: "sig-0".to_string() 
                }).expect("Failed to update sensor data.");
            },
            None => { }
        }
    }

}