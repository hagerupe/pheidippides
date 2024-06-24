pub mod krakenrf {
    use std::error::Error;
    use csv::ReaderBuilder;
    use reqwest;

    pub struct DoaValue {
        pub values: [f32; 360]
    }

    pub struct KrakenClient {
        pub endpoint: String
    }

    impl KrakenClient {
        pub async fn get_doa(&self) -> Result<DoaValue, Box<dyn Error>> {
            let url = format!("{}/DOA_value.html", self.endpoint);
            let response = reqwest::get(url).await?.text().await?;
        
            // Parse the response as CSV
            let mut reader = ReaderBuilder::new()
                .has_headers(false)
                .from_reader(response.as_bytes());
        
            let mut value = DoaValue { values: [0.0; 360] };
            for result in reader.records() {
                let record = result?;
                let mut idx = 0;
                for field in record.iter().skip(17) { // Skip to DOA output.
                    value.values[idx] = field.replace(" ", "")
                                             .parse::<f32>().unwrap();
                    idx += 1;
                }
            }
            return Ok(value)
        }
    }
}

pub mod cots {

    use crate::cots::types::{Event, Detail, Point, Sensor, IntVal, FloatVal, BoolVal, Contact, Remark, Archive, Link};
    use std::{error::Error, net::TcpStream, time::Duration, ops::Add};

    use std::io::{self, Write};

    use chrono::{DateTime, Utc};
    use yaserde::ser::Config;

    pub struct CotsListener {
        pub endpoint: String,
    }
    impl CotsListener {
        pub fn connect(&self) -> Result<CotsPublisher, Box<dyn Error>> {
            Ok(CotsPublisher {
                stream: TcpStream::connect(self.endpoint.clone())?
            })
        }
    }

    /* represents state of a radio direction of arrival
       sensor's signal of interest. */
    pub struct RDoASOI {
        pub s_contact: String,
        pub s_fov: u32,
        pub s_range: i32,
        pub azimuth: i32,
        pub uid: String,
    }

    pub struct CotsPublisher {
        stream: TcpStream,
    }
    impl CotsPublisher {
        
        pub fn send_sensor(&mut self, soi: RDoASOI) -> Result<(), Box<dyn Error>> {

            let uca_fov = 8;
            let range = 10000;
        
            let now: DateTime<Utc> = Utc::now();
            let sample = Event {
                version: "2.0".to_string(),
                uid: soi.uid,
                uid_type: "a-f-G-U-C".to_string(),
                time: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                start: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                stale: now.add(Duration::from_secs(3600)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                how: "m-g".to_string(),
                detail: Detail {
                    sensor: Sensor {
                        vfov:soi.s_fov,
                        elevation:0,
                        fovBlue:0.0,
                        fovRed:1.0,
                        strokeWeight:0.0,
                        roll:0,
                        range:soi.s_range,
                        azimuth:soi.azimuth,
                        rangeLineStrokeWeight:0.55,
                        fov:soi.s_fov,
                        rangeLineStrokeColor:-16777216,
                        fovGreen:0.0,
                        fovLabels:true,
                        displayMagneticReference:0,
                        strokeColor:-16777216,
                        rangeLines:500,
                        fovAlpha:0.2627450980392157,
                    },
                    link: Vec::new(),
                    stroke_color: IntVal { value: -1 },
                    stroke_weight: FloatVal { value: 4.0 },
                    fill_color: IntVal { value: -1761607681 },
                    contact: Contact{callsign: soi.s_contact},
                    remarks: Vec::new(),
                    archive: Archive { },
                    labels_on: BoolVal { value: false },
                    color: IntVal { value: -1 },
                },
                point: Point {
                    lat: 12.0,
                    lon: 12.0,
                    ce: 10.0,
                    hae: 10.0,
                    le: 10.0,
                }
            };
        
            let data = yaserde::ser::to_string_with_config(&sample, &Config {
                perform_indent: false,
                write_document_declaration: false,
                indent_string: None,
            }).unwrap() + "\n\n";
        
            let buf = data.as_bytes();
            self.stream.write(&buf)?;
            Ok(())
        }
    }

    mod types {
        use yaserde::YaSerialize;

        #[derive(YaSerialize, PartialEq, Debug)]
        #[yaserde(rename = "event")]
        pub struct Event {
            #[yaserde(attribute)]
            pub version: String,
        
            #[yaserde(attribute)]
            pub uid: String,
            
            #[yaserde(attribute)]
            #[yaserde(rename = "type")]
            pub uid_type: String,
        
            #[yaserde(attribute)]
            pub time: String,
        
            #[yaserde(attribute)]
            pub start: String,
        
            #[yaserde(attribute)]
            pub stale: String,
        
            #[yaserde(attribute)]
            pub how: String,
        
            pub(crate) detail: Detail,
            pub point: Point,
        }
        
        #[derive(YaSerialize, PartialEq, Debug)]
        #[yaserde(rename = "point")]
        pub struct Point {
            #[yaserde(attribute)]
            pub lat: f32,
            
            #[yaserde(attribute)]
            pub lon: f32,
            
            #[yaserde(attribute)]
            pub ce: f32,
            
            #[yaserde(attribute)]
            pub hae: f32,
        
            #[yaserde(attribute)]
            pub le: f32,
        }
        
        #[derive(YaSerialize, PartialEq, Debug)]
        #[yaserde(rename = "detail")]
        pub struct Detail {
            pub sensor: Sensor,
        
            pub link: Vec<Link>,
        
            #[yaserde(rename = "strokeColor")]
            pub stroke_color: IntVal,
            
            #[yaserde(rename = "strokeWeight")]
            pub stroke_weight: FloatVal,
        
            #[yaserde(rename = "fillColor")]
            pub fill_color: IntVal,
        
            pub contact: Contact,
        
            pub remarks: Vec<Remark>,
        
            pub archive: Archive,
        
            pub labels_on: BoolVal,
        
            pub color: IntVal,
        }
        
        
        #[derive(YaSerialize, PartialEq, Debug)]
        #[yaserde(rename = "sensor")]
        pub struct Sensor {
            #[yaserde(attribute)]
            pub vfov: u32,
            #[yaserde(attribute)]
            pub elevation: u32,
            #[yaserde(attribute)]
            pub fovBlue: f32,
            #[yaserde(attribute)]
            pub fovRed: f32,
            #[yaserde(attribute)]
            pub strokeWeight: f32,
            #[yaserde(attribute)]
            pub roll: u32,
            #[yaserde(attribute)]
            pub range: i32,
            #[yaserde(attribute)]
            pub azimuth: i32,
            #[yaserde(attribute)]
            pub rangeLineStrokeWeight: f32,
            #[yaserde(attribute)]
            pub fov: u32,
            #[yaserde(attribute)]
            pub rangeLineStrokeColor: i32,
            #[yaserde(attribute)]
            pub fovGreen: f32,
            #[yaserde(attribute)]
            pub fovLabels: bool,
            #[yaserde(attribute)]
            pub displayMagneticReference: i32,
            #[yaserde(attribute)]
            pub strokeColor: i32,
            #[yaserde(attribute)]
            pub rangeLines: i32,
            #[yaserde(attribute)]
            pub fovAlpha: f32,
        }
        
        #[derive(YaSerialize, PartialEq, Debug)]
        #[yaserde(rename = "remark")]
        pub struct Remark {
        
        }
        
        #[derive(YaSerialize, PartialEq, Debug)]
        #[yaserde(rename = "archive")]
        pub struct Archive {
        
        }
        
        #[derive(YaSerialize, PartialEq, Debug)]
        #[yaserde(rename = "link")]
        pub struct Link {
            #[yaserde(attribute)]
            pub point: String,
        }
        
        #[derive(YaSerialize, PartialEq, Debug)]
        #[yaserde(rename = "contact")]
        pub struct Contact {
            #[yaserde(attribute)]
            pub callsign: String
        }
        
        #[derive(YaSerialize, PartialEq, Debug)]
        pub struct IntVal {
            #[yaserde(attribute)]
            pub value: i32
        }
        
        #[derive(YaSerialize, PartialEq, Debug)]
        pub struct FloatVal {
            #[yaserde(attribute)]
            pub value: f32
        }
        
        #[derive(YaSerialize, PartialEq, Debug)]
        pub struct BoolVal {
            #[yaserde(attribute)]
            pub value: bool
        }
    }

}

pub mod meshtastic {

    use meshtastic::api::ConnectedStreamApi;
    use ::meshtastic::packet::PacketDestination::Broadcast;
    use ::meshtastic::packet::PacketRouter;
    use meshtastic::protobufs::FromRadio;
    use ::meshtastic::types::{MeshChannel, NodeId};
    use ::meshtastic::utils::stream::build_serial_stream;
    use ::meshtastic::api::StreamApi;
    use ::meshtastic::utils;
    use tokio::sync::mpsc::UnboundedReceiver;

    pub struct MeshRadio {
        pub serial_port: String
    }
    impl MeshRadio {
        pub async fn connect(&self) -> Result<MeshWriteStream, std::io::Error> {
            let stream_api = StreamApi::new();
            let serial_stream = build_serial_stream(self.serial_port.clone(), None, None, None).unwrap();
            let (decoded_listener, stream_api) = stream_api.connect(serial_stream).await;
            let config_id = utils::generate_rand_id();
            Ok(MeshWriteStream{
                decoded_listener: decoded_listener,
                stream_api: stream_api.configure(config_id).await.unwrap()})
        }
    }

    pub struct DoASensorState {
        pub azimuth: i32
    }

    pub struct MeshWriteStream {
        pub decoded_listener: UnboundedReceiver<FromRadio>,
        pub stream_api: ConnectedStreamApi,
    }
    impl MeshWriteStream {
        pub async fn send(&mut self, sense_state: DoASensorState) -> Result<(), std::io::Error> {
            let mut packet_router = NoOpRouter{};
            let channel = MeshChannel::new(0).unwrap();
            self.stream_api.send_text(&mut packet_router, sense_state.azimuth.to_string(), Broadcast, true, channel).await.unwrap();        
            Ok(())
        }
    }

    pub struct NoOpRouter { } 
    impl PacketRouter<i32, std::io::Error> for NoOpRouter {
        fn handle_packet_from_radio(&mut self, _packet: ::meshtastic::protobufs::FromRadio) -> Result<i32, std::io::Error> {
            return Ok(0);
        }
    
        fn handle_mesh_packet(&mut self, _packet: ::meshtastic::protobufs::MeshPacket) -> Result<i32, std::io::Error> {
            return Ok(0);
        }
    
        fn source_node_id(&self) -> NodeId {
            return NodeId::new(0);
        }
    }

}