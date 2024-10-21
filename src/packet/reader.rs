#![allow(unused)]

use crate::error::MinecraftError;
use crate::error::Result;
use bytes::BufMut;
use bytes::BytesMut;
use tracing::debug;

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

pub struct PacketReader;

impl PacketReader {
    pub fn read_varint(buffer: &mut &[u8]) -> Result<i32> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let byte = match buffer.first() {
                Some(&b) => {
                    *buffer = &buffer[1..];
                    b
                }
                None => return Err(MinecraftError::VarInt("buffer underflow".to_string())),
            };

            result |= ((byte & SEGMENT_BITS) as i32) << shift;
            if byte & CONTINUE_BIT == 0 {
                debug!("Read VarInt: {} (raw bytes: {:02x})", result, byte);
                return Ok(result);
            }
            shift += 7;
            if shift >= 32 {
                return Err(MinecraftError::VarInt("varint too long".to_string()));
            }
        }
    }

    pub fn read_string(buf: &mut &[u8]) -> Result<String> {
        let length = Self::read_varint(buf)?;

        if length < 0 {
            return Err(MinecraftError::Protocol(
                "String length cannot be negative".into(),
            ));
        }

        if buf.len() < length as usize {
            return Err(MinecraftError::BufferUnderrun(
                "String length exceeds buffer size".into(),
            ));
        }

        let string = std::str::from_utf8(&buf[..length as usize])?.to_string();
        *buf = &buf[length as usize..];

        Ok(string)
    }

    pub fn read_byte(buf: &mut &[u8]) -> Result<i8> {
        if buf.is_empty() {
            return Err(MinecraftError::BufferUnderrun(
                "Not enough bytes for byte".into(),
            ));
        }
        let value = buf[0] as i8;
        *buf = &buf[1..];
        Ok(value)
    }

    pub fn read_unsigned_byte(buf: &mut &[u8]) -> Result<u8> {
        if buf.is_empty() {
            return Err(MinecraftError::BufferUnderrun(
                "Not enough bytes for unsigned byte".into(),
            ));
        }
        let value = buf[0];
        *buf = &buf[1..];
        Ok(value)
    }

    pub fn read_boolean(buf: &mut &[u8]) -> Result<bool> {
        if buf.is_empty() {
            return Err(MinecraftError::BufferUnderrun(
                "Not enough bytes for boolean".into(),
            ));
        }
        let value = buf[0];
        *buf = &buf[1..];
        Ok(value != 0)
    }

    pub fn read_unsigned_short(buf: &mut &[u8]) -> Result<u16> {
        if buf.len() < 2 {
            return Err(MinecraftError::BufferUnderrun(
                "Not enough bytes for unsigned short".into(),
            ));
        }

        let result = ((buf[0] as u16) << 8) | (buf[1] as u16);
        *buf = &buf[2..];

        Ok(result)
    }

    pub fn read_long(buf: &mut &[u8]) -> Result<i64> {
        if buf.len() < 8 {
            return Err(MinecraftError::BufferUnderrun(
                "Not enough bytes for long".into(),
            ));
        }

        let mut result = 0;
        for i in 0..8 {
            result |= (buf[i] as i64) << ((7 - i) * 8);
        }

        *buf = &buf[8..];
        Ok(result)
    }

    pub fn read_plugin_message(buf: &mut &[u8]) -> Result<(String, Vec<u8>)> {
        let channel_id = Self::read_string(buf)?;

        //  (byte array)
        let data = buf.to_vec();
        *buf = &[]; // clear the buffer

        Ok((channel_id, data))
    }

    pub fn read_identifier(buf: &mut &[u8]) -> Result<(String, String)> {
        let full_id = Self::read_string(buf)?;
        let parts: Vec<&str> = full_id.split(':').collect();
        if parts.len() != 2 {
            return Err(MinecraftError::Protocol("Invalid identifier format".into()));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    pub fn get_varint_size(value: i32) -> usize {
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
    pub fn write_varint(buf: &mut BytesMut, mut value: i32) {
        loop {
            let mut byte = value as u8 & SEGMENT_BITS;

            value >>= 7;

            if value != 0 {
                byte |= CONTINUE_BIT;
            }

            buf.put_u8(byte);

            if value == 0 {
                break;
            }
        }
    }

    pub fn write_identifier(buf: &mut BytesMut, namespace: &str, path: &str) {
        let identifier = format!("{}:{}", namespace, path);
        Self::write_string(buf, &identifier);
    }

    pub fn write_string(buf: &mut BytesMut, value: &str) {
        Self::write_varint(buf, value.len() as i32);
        buf.put(value.as_bytes());
    }
}
