use crate::error::MinecraftError;
use crate::error::Result;
use bytes::BufMut;
use bytes::BytesMut;

pub struct PacketReader;

impl PacketReader {

    pub fn read_varint(buf: &mut &[u8]) -> Result<i32> {
        let mut result = 0;
        let mut length = 0;

        loop {
            if buf.is_empty() {
                return Err(MinecraftError::VarInt("Incomplete VarInt".into()));
            }

            let byte = buf[0];
            *buf = &buf[1..];

            result |= ((byte & 0x7F) as i32) << (length * 7);
            length += 1;

            if length > 5 {
                return Err(MinecraftError::VarInt("VarInt too big".into()));
            }

            if (byte & 0x80) == 0 {
                break;
            }
        }

        Ok(result)
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
            let mut byte = (value & 0x7F) as u8;

            value >>= 7;

            if value != 0 {
                byte |= 0x80;
            }

            buf.put_u8(byte);

            if value == 0 {
                break;
            }
        }
    }

    pub fn write_string(buf: &mut BytesMut, value: &str) {
        Self::write_varint(buf, value.len() as i32);
        buf.put(value.as_bytes());
    }
}
