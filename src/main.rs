#![deny(
  nonstandard_style,
  macro_use_extern_crate,
  missing_copy_implementations,
  missing_debug_implementations,
  unsafe_op_in_unsafe_fn,
  unused_crate_dependencies,
  unused_extern_crates,
  unused_results,
  deprecated,
  drop_bounds,
  dyn_drop,
  exported_private_dependencies,
  invalid_value,
  non_shorthand_field_patterns
)]
#![warn(trivial_casts, trivial_numeric_casts, variant_size_differences)]
#![allow(dead_code)]

use std::{
  error::Error,
  io::{Read, Write},
  net::{SocketAddr, TcpStream},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use audio_volume::EndptVol;
use enigo::Enigo;

use crate::{io_events::Performable, utils::WriteSizedExt};

mod audio_volume;
mod io_events;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
  let _ = dotenv::dotenv()?;
  let mut enigo = Enigo::new();
  let spin = Arc::new(AtomicBool::new(true));
  let spinc = Arc::clone(&spin);
  ctrlc::set_handler(move || {
    println!("received Ctrl+C!");
    spinc.store(false, Ordering::Relaxed);
  })?;

  let mut endpt_vol = EndptVol::new()?;
  println!("Current volume is: {}", endpt_vol.getVol()?);

  let server_addr: SocketAddr = std::env::var("COMMAND_SERVER_ADDRESS")?.parse()?;
  let host_name = std::env::var("HOST_NAME")?.into_bytes();

  while spin.load(Ordering::Relaxed) {
    let stream = TcpStream::connect(server_addr)?;
    if let Err(why) = handle(stream, &mut enigo, &mut endpt_vol, &spin, &host_name) {
      eprintln!("Error: {why}");
    }
  }

  Ok(())
}

fn handle(
  mut stream: TcpStream,
  enigo: &mut Enigo,
  endpt_vol: &mut EndptVol,
  spin: &AtomicBool,
  host_name: &[u8],
) -> Result<(), Box<dyn Error>> {
  use std::io::{Error, ErrorKind};

  // pretend to be host
  stream.write_all(&[1])?;
  stream.write_sized(host_name)?;

  let mut buf = [0u8; 1];
  stream.read_exact(&mut buf)?;
  if buf[0] != 1 {
    return Err(Box::new(Error::new(
      ErrorKind::Other,
      "Failed to set up connection.",
    )));
  }

  while spin.load(Ordering::Relaxed) {
    // request command
    stream.write_all(&[1])?;

    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    match buf[0] {
      0 => continue,
      u8::MAX => break,
      _ => {}
    }
    let mut buf = vec![0u8; buf[0] as usize];
    stream.read_exact(&mut buf)?;

    let event_str = std::str::from_utf8(&buf)?;
    let event: io_events::InputEvent = serde_json::from_str(event_str)?;
    println!("Event: {event:?}");
    event.perform(enigo, endpt_vol);
  }
  stream.write_all(&[2])?;
  Ok(())
}
