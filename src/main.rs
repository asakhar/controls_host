use std::{
  convert::TryInto,
  error::Error,
  io::{Read, Write},
  net::{SocketAddr, TcpStream},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use audio_volume::AudioEndpointVolume;
use enigo::{Enigo, Key, KeyboardControllable};

use command_info::{ClientIdentError, CommandInfo};

mod audio_volume;
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

  let spin = Arc::new(AtomicBool::new(true));
  let spinc = Arc::clone(&spin);
  ctrlc::set_handler(move || {
    println!("received Ctrl+C!");
    spinc.store(false, Ordering::Relaxed);
  })?;

  let mut audio = AudioEndpointVolume::new()?;
  println!("Current volume is: {}", audio.getVol()?);

  while spin.load(Ordering::Relaxed) {
    stream.write_all(&[1])?;
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    match buf[0] {
      0 => continue,
      1 => {}
      _ => break,
    }
    let mut buf = [0u8; 16];
    stream.read_exact(&mut buf)?;
    let command: CommandInfo = buf.try_into()?;
    match command {
      CommandInfo::Next => enigo.key_click(Key::RightArrow),
      CommandInfo::Prev => enigo.key_click(Key::LeftArrow),
      CommandInfo::Pause => enigo.key_click(Key::Space),
      CommandInfo::Vol(t) => audio.setVol(t)?,
    }
  }
  stream.write_all(&[2])?;

  Ok(())
}
