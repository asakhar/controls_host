use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum CommandInfo {
  Next,
  Prev,
  Pause,
  Vol(f64),
}

impl TryFrom<[u8; 16]> for CommandInfo {
  type Error = ClientIdentError;
  fn try_from(buf: [u8; 16]) -> Result<Self, ClientIdentError> {
    Ok(match buf {
      [1, 0, 0, 0, 0, 0, 0, 0, ..] => Self::Next,
      [2, 0, 0, 0, 0, 0, 0, 0, ..] => Self::Prev,
      [3, 0, 0, 0, 0, 0, 0, 0, ..] => Self::Pause,
      [4, 0, 0, 0, 0, 0, 0, 0, ..] => Self::Vol(f64::from_le_bytes(buf[8..16].try_into().unwrap())),
      _ => return Err(ClientIdentError::InvalidByte),
    })
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ClientIdentError {
  InvalidByte,
  AlreadyExists,
}

impl Display for ClientIdentError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      ClientIdentError::InvalidByte => "Invalid byte received for client type",
      ClientIdentError::AlreadyExists => "Host with different address already exists",
    })
  }
}

impl Error for ClientIdentError {}
