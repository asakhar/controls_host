use std::{
    convert::TryInto,
    error::Error,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

use enigo::{Enigo, Key, KeyboardControllable};

use command_info::{ClientIdentError, CommandInfo};

mod command_info;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv()?;
    let mut enigo = Enigo::new();

    let server_addr: SocketAddr = std::env::var("COMMAND_SERVER_ADDRESS")?.parse()?;

    let mut stream = TcpStream::connect(server_addr)?;
    stream.write_all(&[1])?;
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    if buf[0] != 1 {
        eprintln!("Failed to connect as host. There probably already exists one");
        Err(ClientIdentError::AlreadyExists)?;
    }

    loop {
        stream.write_all(&[1])?;
        let mut buf = [0u8; 16];
        stream.read_exact(&mut buf)?;
        let command: CommandInfo = buf.try_into()?;
        match command {
            CommandInfo::Next => enigo.key_click(Key::RightArrow),
            CommandInfo::Prev => enigo.key_click(Key::LeftArrow),
            CommandInfo::Pause => enigo.key_click(Key::Space),
            CommandInfo::Vol(_) => todo!(),
        }
    }

    // Ok(())
}
