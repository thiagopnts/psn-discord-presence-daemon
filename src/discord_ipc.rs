use serde_json::{json, Value};
use std::os::unix::net::UnixStream;
use std::{
    convert::TryInto,
    env::var,
    error::Error,
    io::{Read, Write},
    net::Shutdown,
    path::PathBuf,
};

pub fn pack(opcode: u32, data_len: u32) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for byte_array in &[opcode.to_le_bytes(), data_len.to_le_bytes()] {
        bytes.extend_from_slice(byte_array);
    }
    Ok(bytes)
}

pub fn unpack(data: Vec<u8>) -> Result<(u32, u32)> {
    let data = data.as_slice();
    let (opcode, header) = data.split_at(std::mem::size_of::<u32>());
    let opcode = u32::from_le_bytes(opcode.try_into()?);
    let header = u32::from_le_bytes(header.try_into()?);
    Ok((opcode, header))
}

// Environment keys to search for the Discord pipe
const ENV_KEYS: [&str; 4] = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"];

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[allow(dead_code)]
pub struct DiscordIpc {
    pub client_id: String,
    pub connected: bool,
    pub socket: Option<UnixStream>,
}

impl DiscordIpc {
    pub fn connect(&mut self) -> Result<()> {
        self.connect_ipc()?;
        self.send_handshake()?;

        Ok(())
    }

    fn connect_ipc(&mut self) -> Result<()> {
        for i in 0..10 {
            let path = DiscordIpc::get_pipe_pattern().join(format!("discord-ipc-{}", i));

            match UnixStream::connect(&path) {
                Ok(socket) => {
                    self.socket = Some(socket);
                    return Ok(());
                }
                Err(_) => continue,
            }
        }

        Err("Couldn't connect to the Discord IPC socket".into())
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        let socket = self.socket.as_mut().unwrap();

        socket.write_all(data)?;

        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<()> {
        let socket = self.socket.as_mut().unwrap();

        socket.read_exact(buffer)?;

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        let data = json!({});
        if self.send(data, 2).is_ok() {}

        let socket = self.socket.as_mut().unwrap();

        socket.flush()?;
        match socket.shutdown(Shutdown::Both) {
            Ok(()) => (),
            Err(_err) => (),
        };

        Ok(())
    }

    fn get_client_id(&self) -> &String {
        &self.client_id
    }

    fn send_handshake(&mut self) -> Result<()> {
        self.send(
            json!({
                "v": 1,
                "client_id": self.get_client_id()
            }),
            0,
        )?;
        self.recv()?;

        Ok(())
    }

    pub fn send(&mut self, data: impl serde::Serialize, opcode: u8) -> Result<()> {
        let data_string = serde_json::to_string(&data)?;
        let header = pack(opcode.into(), data_string.len() as u32)?;

        self.write(&header)?;
        self.write(data_string.as_bytes())?;

        Ok(())
    }

    fn recv(&mut self) -> Result<(u32, Value)> {
        let mut header = [0; 8];

        self.read(&mut header)?;
        let (op, length) = unpack(header.to_vec())?;

        let mut data = vec![0u8; length as usize];
        self.read(&mut data)?;

        let response = String::from_utf8(data.to_vec())?;
        let json_data = serde_json::from_str::<Value>(&response)?;

        Ok((op, json_data))
    }
}

impl DiscordIpc {
    fn get_pipe_pattern() -> PathBuf {
        let mut path = String::new();

        for key in &ENV_KEYS {
            match var(key) {
                Ok(val) => {
                    path = val;
                    break;
                }
                Err(_e) => continue,
            }
        }
        PathBuf::from(path)
    }
}
