use crate::error::MinecraftError;
use crate::packet::reader::PacketReader;
use crate::Result;
use bytes::{Buf, BufMut, BytesMut};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, instrument, warn};

const PROTOCOL_VERSION: i32 = 767;

const STATUS_RESPONSE_PACKET_ID: i32 = 0x00;
const PING_PACKET_ID: i32 = 0x01;
const LOGIN_PACKET_ID: i32 = 0x02;

#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
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
                if packet_id == 0x00 {
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
                0x00 => {
                    debug!("Received status request");
                    self.send_status_response().await?;
                }
                0x01 => {
                    let payload = PacketReader::read_long(&mut packet_data)?;
                    debug!(payload, "Received ping request");
                    self.send_pong_response(payload).await?;
                    return Ok(false);
                }
                _ => {
                    warn!(packet_id, "Unknown packet ID in Status state");
                }
            },
            ConnectionState::Login => {
                info!("User trying to log in");

                match packet_id {
                    0x00 => {
                        let username = PacketReader::read_string(&mut packet_data)?;
                        debug!(%username, "Received login start packet");

                        let player_uuid = "00000000-0000-0000-0000-000000000001".to_string();
                        self.send_login_success(&player_uuid, &username).await?;

                        self.state = ConnectionState::Play;
                        info!("Login successful for user: {}", username);
                    }
                    _ => {
                        warn!("Unknown packet ID during login: {}", packet_id);
                    }
                }
            }
            ConnectionState::Play => {
                debug!("Client in play state, processing packets");
            }
        }

        buffer.advance(total_size);

        Ok(true)
    }

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    async fn send_pong_response(&mut self, payload: i64) -> Result<()> {
        debug!(payload, "Sending pong response");
        let mut packet = BytesMut::with_capacity(1024);
        PacketReader::write_varint(&mut packet, 9);
        PacketReader::write_varint(&mut packet, PING_PACKET_ID);
        packet.put_i64(payload);

        self.socket.write_all(&packet).await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn send_login_success(&mut self, uuid: &str, username: &str) -> Result<()> {
        debug!("Sending login success for {} ({})", username, uuid);

        let mut packet = BytesMut::new();

        PacketReader::write_varint(&mut packet, LOGIN_PACKET_ID);

        PacketReader::write_string(&mut packet, uuid);
        PacketReader::write_string(&mut packet, username);

        self.socket.write_all(&packet).await?;
        Ok(())
    }
}
