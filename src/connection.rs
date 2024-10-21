use crate::error::MinecraftError;
use crate::packet::reader::PacketReader;
use crate::registry::manager::RegistryManager;
use crate::Result;
use bytes::{Buf, BufMut, BytesMut};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, instrument, warn};
use uuid::Uuid;

pub const PROTOCOL_VERSION: i32 = 767;

// Packet IDs
pub const HANDSHAKE_PACKET_ID: i32 = 0x00;

pub const STATUS_REQUEST_PACKET_ID: i32 = 0x00;
pub const STATUS_RESPONSE_PACKET_ID: i32 = 0x00;
pub const PING_REQUEST_PACKET_ID: i32 = 0x01;
pub const PONG_RESPONSE_PACKET_ID: i32 = 0x01;

pub const LOGIN_START_PACKET_ID: i32 = 0x00;
pub const LOGIN_SUCCESS_PACKET_ID: i32 = 0x02;
pub const LOGIN_ACKNOWLEDGED_PACKET_ID: i32 = 0x03;

pub const CLIENT_INFORMATION_PACKET_ID: i32 = 0x00;
pub const PLUGIN_MESSAGE_PACKET_ID: i32 = 0x02;
pub const FINISH_CONFIGURATION_PACKET_ID: i32 = 0x03;
pub const KNOWN_PACKS_PACKET_ID: i32 = 0x07;
pub const REGISTRY_DATA_PACKET_ID: i32 = 0x0E;

#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

pub struct Connection {
    socket: tokio::net::TcpStream,
    state: ConnectionState,
}

impl Connection {
    pub fn new(socket: tokio::net::TcpStream) -> Self {
        Self {
            socket,
            state: ConnectionState::Handshake,
        }
    }

    #[instrument(skip(self))]
    pub async fn handle_connection(&mut self) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(1024);

        loop {
            let mut temp_buf = [0; 1024];
            match self.socket.read(&mut temp_buf).await {
                Ok(0) => {
                    debug!("Connection closed by peer");
                    break;
                }
                Ok(n) => {
                    debug!(bytes = n, "Received data");
                    buffer.extend_from_slice(&temp_buf[..n]);

                    while !buffer.is_empty() {
                        match self.handle_packet(&mut buffer).await {
                            Ok(should_continue) => {
                                if !should_continue {
                                    return Ok(());
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self, buffer))]
    pub async fn handle_packet(&mut self, buffer: &mut BytesMut) -> Result<bool> {
        debug!("Received raw packet data: {:?}", &buffer[..]);
        let packet_length = match PacketReader::read_varint(&mut &buffer[..]) {
            Ok(len) => len as usize,
            Err(MinecraftError::VarInt(_)) => return Ok(true), // not enough data yet
            Err(e) => return Err(e),
        };

        let length_size = PacketReader::get_varint_size(packet_length as i32);
        let total_size = length_size + packet_length;

        // check if we have the full packet
        if buffer.len() < total_size {
            return Ok(true);
        }

        // read packet ID from the actual packet data
        let mut packet_data = &buffer[length_size..total_size];
        let packet_id = PacketReader::read_varint(&mut packet_data)?;
        debug!(
            packet_id,
            length = packet_length,
            state = ?self.state,
            "Processing packet"
        );

        match self.state {
            ConnectionState::Handshake => {
                if packet_id == HANDSHAKE_PACKET_ID {
                    let protocol_version = PacketReader::read_varint(&mut packet_data)?;
                    let server_address = PacketReader::read_string(&mut packet_data)?;
                    let server_port = PacketReader::read_unsigned_short(&mut packet_data)?;
                    let next_state = PacketReader::read_varint(&mut packet_data)?;

                    debug!(
                        %protocol_version,
                        %server_address,
                        %server_port,
                        %next_state,
                        "Handshake packet"
                    );

                    match next_state {
                        1 => self.state = ConnectionState::Status,
                        2 => self.state = ConnectionState::Login,
                        _ => {
                            warn!(next_state, "Unexpected next state in handshake");
                            return Err(MinecraftError::Protocol(format!(
                                "Unexpected next state: {next_state}"
                            )));
                        }
                    }
                }
            }
            ConnectionState::Status => match packet_id {
                STATUS_REQUEST_PACKET_ID => {
                    debug!("Received status request");
                    self.send_status_response().await?;
                }
                PING_REQUEST_PACKET_ID => {
                    let payload = PacketReader::read_long(&mut packet_data)?;
                    debug!(payload, "Received ping request");
                    self.send_pong_response(payload).await?;
                    return Ok(false);
                }
                _ => {
                    warn!(packet_id, "Unknown packet ID in Status state");
                }
            },
            ConnectionState::Login => match packet_id {
                LOGIN_START_PACKET_ID => {
                    let username = PacketReader::read_string(&mut packet_data)?;
                    self.send_login_success("00000000-0000-0000-0000-000000000001", &username)
                        .await?;
                }
                LOGIN_ACKNOWLEDGED_PACKET_ID => {
                    debug!("Login acknowledged, switching to Configuration state");
                    self.state = ConnectionState::Configuration;
                    self.send_known_packs().await?;
                }
                _ => warn!(packet_id, "Unknown packet ID in Login state"),
            },
            ConnectionState::Configuration => match packet_id {
                CLIENT_INFORMATION_PACKET_ID => {
                    // Client Information packet in Configuration
                    debug!("Received client information in Configuration state");
                    let locale = PacketReader::read_string(&mut packet_data)?;
                    let view_distance = PacketReader::read_byte(&mut packet_data)?;
                    let chat_mode = PacketReader::read_varint(&mut packet_data)?;
                    let chat_colors = PacketReader::read_boolean(&mut packet_data)?;
                    let displayed_skin_parts = PacketReader::read_unsigned_byte(&mut packet_data)?;
                    let main_hand = PacketReader::read_varint(&mut packet_data)?;
                    let enable_text_filtering = PacketReader::read_boolean(&mut packet_data)?;
                    let allow_server_listings = PacketReader::read_boolean(&mut packet_data)?;

                    debug!(
                        locale,
                        view_distance,
                        chat_mode,
                        chat_colors,
                        displayed_skin_parts,
                        main_hand,
                        enable_text_filtering,
                        allow_server_listings
                    );
                }
                PLUGIN_MESSAGE_PACKET_ID => {
                    // Plugin message (minecraft:brand)
                    // TODO actually do something with the plugin
                    debug!("Received plugin message in Configuration state");
                    let (_channel, _data) = PacketReader::read_plugin_message(&mut packet_data)?;
                }
                FINISH_CONFIGURATION_PACKET_ID => {
                    debug!("Ack configuration finished, switching to Play state");

                    self.state = ConnectionState::Play;
                    // TODO self.send_play_login().await?;
                    // TODO self.send_chunk_data().await?;
                }
                KNOWN_PACKS_PACKET_ID => {
                    let pack_count = PacketReader::read_varint(&mut packet_data)?;
                    debug!("Received Serverbound Known Packs request in Configuration state. packets={pack_count}");

                    let mut known_packs = Vec::new();
                    for _ in 0..pack_count {
                        let namespace = PacketReader::read_string(&mut packet_data)?;
                        let id = PacketReader::read_string(&mut packet_data)?;
                        let version = PacketReader::read_string(&mut packet_data)?;

                        debug!(
                            "Known Pack - Namespace: {}, ID: {}, Version: {}",
                            namespace, id, version
                        );

                        known_packs.push((namespace, id, version));
                    }

                    self.send_registry_data().await?;
                    self.send_finish_configuration().await?;
                }
                _ => warn!(packet_id, "Unknown packet ID in Configuration state"),
            },
            ConnectionState::Play => {
                debug!("Client in Play state, processing packet {}", packet_id);
                match packet_id {
                    _ => debug!(packet_id, "Unhandled Play state packet ID"),
                }
            }
        }

        buffer.advance(total_size);

        Ok(true)
    }

    // packet length     varint
    // packet id         varint
    // response          string
    async fn send_status_response(&mut self) -> Result<()> {
        let response = json!({
            "version": {
                "name": "1.21.1",
                "protocol": PROTOCOL_VERSION
            },
            "players": {
                "max": 100,
                "online": 4,
                "sample": [
                    {
                        "name": "Player",
                        "id": "00000000-0000-0000-0000-000000000001"
                    }
                ]
            },
            "description": {
                "text": "Hello world!"
            }
        });

        let response_str = response.to_string();
        debug!(response = %response_str, "Sending status response");

        let mut packet = BytesMut::new();
        let string_length = response_str.len() as i32;

        let total_length = string_length
            + (PacketReader::get_varint_size(string_length)
                + PacketReader::get_varint_size(STATUS_RESPONSE_PACKET_ID)) as i32;
        PacketReader::write_varint(&mut packet, total_length);
        PacketReader::write_varint(&mut packet, STATUS_RESPONSE_PACKET_ID);
        PacketReader::write_string(&mut packet, &response_str);

        self.socket.write_all(&packet).await?;
        Ok(())
    }

    // packet length  varint
    // packet id      varint
    async fn send_pong_response(&mut self, payload: i64) -> Result<()> {
        debug!(payload, "Sending pong response");

        let mut packet = BytesMut::with_capacity(1024);

        PacketReader::write_varint(&mut packet, 9); // payload is always 9 bytes
        PacketReader::write_varint(&mut packet, PING_REQUEST_PACKET_ID);

        packet.put_i64(payload);

        self.socket.write_all(&packet).await?;

        Ok(())
    }

    // packet length    varint
    // UUID             string
    // username         string
    // properties       varint
    // chat validation  boolean
    async fn send_login_success(&mut self, uuid_str: &str, username: &str) -> Result<()> {
        let mut content = BytesMut::new();

        PacketReader::write_varint(&mut content, LOGIN_SUCCESS_PACKET_ID);

        let uuid = Uuid::parse_str(uuid_str).unwrap();
        content.extend_from_slice(uuid.as_bytes());

        PacketReader::write_string(&mut content, username);

        // we have 0 properties
        PacketReader::write_varint(&mut content, 0);

        content.put_u8(0);

        let mut packet = BytesMut::new();
        PacketReader::write_varint(&mut packet, content.len() as i32);
        packet.extend_from_slice(&content);

        debug!("Sending login success packet: {:?}", packet);
        self.socket.write_all(&packet).await?;
        Ok(())
    }

    async fn send_keep_alive(&mut self) -> Result<()> {
        let mut packet = BytesMut::with_capacity(10);
        PacketReader::write_varint(&mut packet, 0x21);
        packet.put_i64(12345); // arbitrary payload, could be any number

        self.socket.write_all(&packet).await?;
        Ok(())
    }

    /// Sends known packs
    // packet length   varint
    // packet id       varint
    // amt of packs    varint
    //
    // foreach pack
    // ----------------------
    // namespace       string
    // title           string
    // version         string
    // ----------------------
    async fn send_known_packs(&mut self) -> Result<()> {
        let mut content = BytesMut::new();

        PacketReader::write_varint(&mut content, 0x0E);

        PacketReader::write_varint(&mut content, 1);

        PacketReader::write_string(&mut content, "minecraft");
        PacketReader::write_string(&mut content, "core");
        PacketReader::write_string(&mut content, "1.21.1");

        let mut packet = BytesMut::new();
        PacketReader::write_varint(&mut packet, content.len() as i32);
        packet.extend_from_slice(&content);

        debug!("Sending known packs packet: {:?}", packet);
        self.socket.write_all(&packet).await?;

        Ok(())
    }

    /// Sends default registry data
    async fn send_registry_data(&mut self) -> Result<()> {
        let manager = RegistryManager::new()?;

        manager.write_registry_data(&mut self.socket).await?;

        debug!("Sent registry data packet");
        Ok(())
    }

    // packet length  varint
    // packet id      varint
    async fn send_finish_configuration(&mut self) -> Result<()> {
        let mut packet = BytesMut::new();
        PacketReader::write_varint(&mut packet, 0x03);

        let mut final_packet = BytesMut::new();
        PacketReader::write_varint(&mut final_packet, packet.len() as i32);
        final_packet.extend_from_slice(&packet);

        self.socket.write_all(&final_packet).await?;

        Ok(())
    }

    /// Helper func to get varint size
    fn get_varint_size(value: i32) -> usize {
        let mut size = 0;
        let mut val = value as u32;

        loop {
            size += 1;
            val >>= 7;

            if val == 0 {
                break;
            }
        }

        size
    }
}
