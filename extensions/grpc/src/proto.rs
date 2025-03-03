//! Module for handling protobuf-like binary encoding and decoding
//! 
//! This module provides functionality for encoding and decoding messages
//! in a protobuf-like binary format. It includes both encoding/decoding
//! between JSON and binary formats, as well as framing for gRPC messages.

pub mod framing {
    use grafbase_sdk::Error;

    /// Creates a gRPC frame from a message payload
    /// 
    /// A gRPC frame consists of:
    /// - 1 byte compression flag (0 = no compression)
    /// - 4 bytes message length (big endian)
    /// - The message payload
    pub fn create_grpc_frame(payload: &[u8]) -> Vec<u8> {
        let message_len = payload.len();
        let mut frame = Vec::with_capacity(message_len + 5);
        
        // Compression flag (0 = no compression)
        frame.push(0);
        
        // Message length (4 bytes, big endian)
        frame.extend_from_slice(&(message_len as u32).to_be_bytes());
        
        // Message payload
        frame.extend_from_slice(payload);
        
        frame
    }
    
    /// Extracts gRPC frames from a response
    /// 
    /// This function takes a byte array containing one or more gRPC frames
    /// and extracts each frame's payload into a vector of byte vectors.
    pub fn extract_grpc_frames(response_bytes: &[u8]) -> Result<Vec<Vec<u8>>, Error> {
        let mut frames = Vec::new();
        let mut offset = 0;
        
        while offset + 5 <= response_bytes.len() {
            // Read compression flag (1 byte)
            let compression_flag = response_bytes[offset];
            offset += 1;
            
            // Read message length (4 bytes, big endian)
            let mut length_bytes = [0u8; 4];
            length_bytes.copy_from_slice(&response_bytes[offset..offset + 4]);
            let message_length = u32::from_be_bytes(length_bytes) as usize;
            offset += 4;
            
            // Check if we have enough data for the message
            if offset + message_length > response_bytes.len() {
                return Err(format!("Incomplete gRPC frame: expected {} bytes but only {} available", 
                                  message_length, response_bytes.len() - offset).into());
            }
            
            // Extract the message
            let message = response_bytes[offset..offset + message_length].to_vec();
            frames.push(message);
            
            // Move to the next message
            offset += message_length;
        }
        
        if offset < response_bytes.len() {
            // If there's remaining data but not enough for a full frame
            return Err(format!("Trailing data in gRPC response: {} bytes", response_bytes.len() - offset).into());
        }
        
        Ok(frames)
    }
}

pub mod encoding {
    use serde_json::{Value, Map, Number};
    use std::collections::HashMap;
    use std::io::{Cursor, Read, Write};
    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

    // Wire types for our protobuf-like encoding
    const WIRE_TYPE_VARINT: u8 = 0;
    const WIRE_TYPE_FIXED64: u8 = 1;
    const WIRE_TYPE_LENGTH_DELIMITED: u8 = 2;
    const WIRE_TYPE_FIXED32: u8 = 5;

    // Value types for our protobuf-like encoding
    const TYPE_NULL: u8 = 0;
    const TYPE_BOOL: u8 = 1;
    const TYPE_NUMBER: u8 = 2;
    const TYPE_STRING: u8 = 3;
    const TYPE_ARRAY: u8 = 4;
    const TYPE_OBJECT: u8 = 5;

    /// Encodes a JSON value to a binary format that resembles protobuf
    pub fn encode_json_to_binary(value: &serde_json::Value) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();
        encode_value(&mut buffer, value)?;
        Ok(buffer)
    }

    /// Decodes a binary message back into a JSON value
    pub fn decode_binary_to_json(data: &[u8]) -> Result<serde_json::Value, String> {
        let mut cursor = Cursor::new(data);
        decode_value(&mut cursor)
    }

    // Helper functions for encoding

    fn encode_value(buffer: &mut Vec<u8>, value: &Value) -> Result<(), String> {
        match value {
            Value::Null => {
                // Just write a type byte for null
                buffer.push(TYPE_NULL);
            }
            Value::Bool(b) => {
                buffer.push(TYPE_BOOL);
                buffer.push(if *b { 1 } else { 0 });
            }
            Value::Number(n) => {
                buffer.push(TYPE_NUMBER);
                encode_number(buffer, n)?;
            }
            Value::String(s) => {
                buffer.push(TYPE_STRING);
                encode_string(buffer, s)?;
            }
            Value::Array(arr) => {
                buffer.push(TYPE_ARRAY);
                // Write array length as varint
                write_varint(buffer, arr.len() as u64)?;
                // Write each array element
                for item in arr {
                    encode_value(buffer, item)?;
                }
            }
            Value::Object(obj) => {
                buffer.push(TYPE_OBJECT);
                // Write object field count as varint
                write_varint(buffer, obj.len() as u64)?;
                // Write each key-value pair
                for (key, value) in obj {
                    encode_string(buffer, key)?;
                    encode_value(buffer, value)?;
                }
            }
        }
        Ok(())
    }

    fn encode_number(buffer: &mut Vec<u8>, n: &Number) -> Result<(), String> {
        if let Some(i) = n.as_i64() {
            // Integer encoding
            buffer.push(0); // integer flag
            write_varint(buffer, zigzag_encode(i))?;
        } else if let Some(f) = n.as_f64() {
            // Float encoding
            buffer.push(1); // float flag
            buffer.write_f64::<BigEndian>(f)
                .map_err(|e| format!("Failed to write f64: {}", e))?;
        } else {
            return Err("Unsupported number format".to_string());
        }
        Ok(())
    }

    fn encode_string(buffer: &mut Vec<u8>, s: &str) -> Result<(), String> {
        let bytes = s.as_bytes();
        // Write length as varint
        write_varint(buffer, bytes.len() as u64)?;
        // Write string bytes
        buffer.extend_from_slice(bytes);
        Ok(())
    }

    // Helper functions for decoding

    fn decode_value(cursor: &mut Cursor<&[u8]>) -> Result<Value, String> {
        if cursor.position() as usize >= cursor.get_ref().len() {
            return Err("Unexpected end of data".to_string());
        }

        let mut type_buf = [0u8; 1];
        cursor.read_exact(&mut type_buf)
            .map_err(|e| format!("Failed to read value type: {}", e))?;

        match type_buf[0] {
            TYPE_NULL => Ok(Value::Null),
            TYPE_BOOL => {
                let mut bool_buf = [0u8; 1];
                cursor.read_exact(&mut bool_buf)
                    .map_err(|e| format!("Failed to read boolean value: {}", e))?;
                Ok(Value::Bool(bool_buf[0] != 0))
            },
            TYPE_NUMBER => decode_number(cursor),
            TYPE_STRING => {
                let string = decode_string(cursor)?;
                Ok(Value::String(string))
            },
            TYPE_ARRAY => {
                let array_len = read_varint(cursor)? as usize;
                let mut array = Vec::with_capacity(array_len);
                
                for _ in 0..array_len {
                    array.push(decode_value(cursor)?);
                }
                
                Ok(Value::Array(array))
            },
            TYPE_OBJECT => {
                let field_count = read_varint(cursor)? as usize;
                let mut map = Map::new();
                
                for _ in 0..field_count {
                    // Read field name
                    let key = decode_string(cursor)?;
                    
                    // Read field value
                    let value = decode_value(cursor)?;
                    
                    map.insert(key, value);
                }
                
                Ok(Value::Object(map))
            },
            unknown_type => Err(format!("Unknown value type: {}", unknown_type)),
        }
    }

    fn decode_number(cursor: &mut Cursor<&[u8]>) -> Result<Value, String> {
        // Read number type flag
        let mut flag_buf = [0u8; 1];
        cursor.read_exact(&mut flag_buf)
            .map_err(|e| format!("Failed to read number type flag: {}", e))?;
            
        match flag_buf[0] {
            0 => {
                // Integer
                let varint = read_varint(cursor)?;
                let i = zigzag_decode(varint);
                Ok(Value::Number(Number::from(i)))
            },
            1 => {
                // Float
                let f = cursor.read_f64::<BigEndian>()
                    .map_err(|e| format!("Failed to read f64: {}", e))?;
                
                // Create Number from f64, handling potential failure
                Number::from_f64(f)
                    .map(Value::Number)
                    .ok_or_else(|| format!("Cannot represent {} as a JSON number", f))
            },
            unknown_flag => Err(format!("Unknown number type flag: {}", unknown_flag)),
        }
    }

    fn decode_string(cursor: &mut Cursor<&[u8]>) -> Result<String, String> {
        // Read string length
        let string_len = read_varint(cursor)? as usize;
        
        // Read string bytes
        let mut string_bytes = vec![0u8; string_len];
        cursor.read_exact(&mut string_bytes)
            .map_err(|e| format!("Failed to read string bytes: {}", e))?;
            
        // Convert bytes to string
        String::from_utf8(string_bytes)
            .map_err(|e| format!("Invalid UTF-8 sequence: {}", e))
    }

    // Helper for writing varints (variable-length integers)
    fn write_varint(buffer: &mut Vec<u8>, mut value: u64) -> Result<(), String> {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            
            if value != 0 {
                // More bytes to come
                byte |= 0x80;
            }
            
            buffer.push(byte);
            
            if value == 0 {
                break;
            }
        }
        
        Ok(())
    }

    // Helper for reading varints
    fn read_varint(cursor: &mut Cursor<&[u8]>) -> Result<u64, String> {
        let mut result: u64 = 0;
        let mut shift: u32 = 0;
        
        loop {
            if shift > 63 {
                return Err("Varint is too large".to_string());
            }
            
            let mut byte_buf = [0u8; 1];
            cursor.read_exact(&mut byte_buf)
                .map_err(|e| format!("Failed to read varint byte: {}", e))?;
                
            let byte = byte_buf[0];
            result |= ((byte & 0x7F) as u64) << shift;
            
            if byte & 0x80 == 0 {
                // This is the last byte
                break;
            }
            
            shift += 7;
        }
        
        Ok(result)
    }

    // ZigZag encoding for signed integers (maps signed integers to unsigned integers)
    fn zigzag_encode(value: i64) -> u64 {
        ((value << 1) ^ (value >> 63)) as u64
    }

    // ZigZag decoding (maps unsigned integers back to signed integers)
    fn zigzag_decode(value: u64) -> i64 {
        ((value >> 1) as i64) ^ (-((value & 1) as i64))
    }
}
