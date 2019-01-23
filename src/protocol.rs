use std::collections::HashMap;
use std::io::{Read, Write, Cursor};
use std::sync::atomic::{AtomicIsize, Ordering};

use serde::{Serialize, Deserialize};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

const _LENGTH: usize = 4;
const _HEADER_LENGTH: usize = 4;

static mut GLOBAL_OPAQUE: AtomicIsize = AtomicIsize::new(0);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Header {
    code: isize,
    language: String,
    version: isize,
    opaque: i32,
    flag: isize,
    remark: String,
    ext_fields: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub struct RemoteCommand {
    header: Header,
    body: Vec<u8>,
}

impl RemoteCommand {
    pub fn new(code: isize, flag: isize, remark: String, fields: HashMap<String, String>, body: Vec<u8>) -> Self {
        Self {
            header: Header {
                code: code,
                language: "OTHER".to_string(),
                version: 431,
                opaque: unsafe { GLOBAL_OPAQUE.fetch_add(1, Ordering::Relaxed) as i32 },
                flag,
                remark,
                ext_fields: fields,
            },
            body,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut wtr = Vec::new();
        let header_bytes = serde_json::to_vec(&self.header).unwrap();
        let header_len = header_bytes.len();
        let length = _HEADER_LENGTH + header_len + self.body.len();
        wtr.write_i32::<BigEndian>(length as i32).unwrap();
        wtr.write_i32::<BigEndian>(header_len as i32).unwrap();
        wtr.write_all(&header_bytes).unwrap();
        if !self.body.is_empty() {
            wtr.write_all(&self.body).unwrap();
        }
        wtr
    }

    pub fn from_buffer(input: &[u8]) -> Self {
        let mut rdr = Cursor::new(input);
        let length = rdr.read_i32::<BigEndian>().unwrap();
        let header_len = rdr.read_i32::<BigEndian>().unwrap();
        let mut header_buf = vec![0; header_len as usize];
        rdr.read_exact(&mut header_buf).unwrap();
        let header: Header = serde_json::from_slice(&header_buf).unwrap();
        let body_len = length as usize - _HEADER_LENGTH - header_len as usize;
        let body = {
            if body_len > 0 {
                let mut body_buf = vec![0; body_len];
                rdr.read_exact(&mut body_buf).unwrap();
                body_buf
            } else {
                Vec::new()
            }
        };
        Self {
            header,
            body,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::RemoteCommand;

    #[test]
    fn test_remote_command_encode_decode_roundtrip() {
        let mut fields = HashMap::new();
        fields.insert("messageId".to_string(), "123".to_string());
        fields.insert("offset".to_string(), "456".to_string());
        let cmd = RemoteCommand::new(10, 0, "remark".to_string(), fields, b"Hello World".to_vec());
        let encoded = cmd.encode();
        let decoded = RemoteCommand::from_buffer(&encoded);
        assert_eq!(cmd, decoded);
    }
}