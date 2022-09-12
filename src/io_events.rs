use enigo::Enigo;
use enigo::KeyboardControllable;
use enigo::MouseControllable;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;

use crate::audio_volume::EndptVol;
use crate::audio_volume::VolumeLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButtonAction {
  Down,
  Up,
  Click(Duration),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MouseButtonEvent {
  action: MouseButtonAction,
  button: enigo::MouseButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseMoveOrigin {
  Rel,
  Abs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MouseMoveEvent {
  origin: MouseMoveOrigin,
  x: i32,
  y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseScrollDir {
  Ver,
  Hor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MouseScrollEvent {
  dir: MouseScrollDir,
  len: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MouseEvent {
  Button(MouseButtonEvent),
  Move(MouseMoveEvent),
  Scroll(MouseScrollEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyboardAction {
  Up,
  Down,
  Click(Duration),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardEvent {
  action: KeyboardAction,
  key: enigo::Key,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
  Mouse(MouseEvent),
  Keyboard(KeyboardEvent),
  Volume(VolumeLevel),
}

pub trait Performable {
  fn perform(self, enigo: &mut Enigo, endpt_vol: &mut EndptVol);
}

impl Performable for InputEvent {
  fn perform(self, enigo: &mut Enigo, endpt_vol: &mut EndptVol) {
    match self {
      InputEvent::Mouse(me) => me.perform(enigo, endpt_vol),
      InputEvent::Keyboard(ke) => ke.perform(enigo, endpt_vol),
      InputEvent::Volume(ve) => ve.perform(enigo, endpt_vol),
    }
  }
}

impl Performable for MouseEvent {
  fn perform(self, enigo: &mut Enigo, endpt_vol: &mut EndptVol) {
    match self {
      MouseEvent::Button(mbe) => mbe.perform(enigo, endpt_vol),
      MouseEvent::Move(mme) => mme.perform(enigo, endpt_vol),
      MouseEvent::Scroll(mse) => mse.perform(enigo, endpt_vol),
    }
  }
}

impl Performable for KeyboardEvent {
  fn perform(self, enigo: &mut Enigo, _: &mut EndptVol) {
    match self.action {
      KeyboardAction::Up => enigo.key_up(self.key),
      KeyboardAction::Down => enigo.key_down(self.key),
      KeyboardAction::Click(dur) => {
        enigo.key_down(self.key);
        std::thread::sleep(dur);
        enigo.key_up(self.key);
      }
    }
  }
}

impl Performable for MouseButtonEvent {
  fn perform(self, enigo: &mut Enigo, _: &mut EndptVol) {
    match self.action {
      MouseButtonAction::Down => enigo.mouse_down(self.button),
      MouseButtonAction::Up => enigo.mouse_up(self.button),
      MouseButtonAction::Click(dur) => {
        enigo.mouse_down(self.button);
        std::thread::sleep(dur);
        enigo.mouse_up(self.button);
      }
    }
  }
}

impl Performable for MouseMoveEvent {
  fn perform(self, enigo: &mut Enigo, _: &mut EndptVol) {
    match self.origin {
      MouseMoveOrigin::Rel => enigo.mouse_move_relative(self.x, self.y),
      MouseMoveOrigin::Abs => enigo.mouse_move_to(self.x, self.y),
    }
  }
}

impl Performable for MouseScrollEvent {
  fn perform(self, enigo: &mut Enigo, _: &mut EndptVol) {
    match self.dir {
      MouseScrollDir::Ver => enigo.mouse_scroll_y(self.len),
      MouseScrollDir::Hor => enigo.mouse_scroll_x(self.len),
    }
  }
}

impl Performable for VolumeLevel {
  fn perform(self, _: &mut Enigo, endpt_vol: &mut EndptVol) {
    let _ = endpt_vol.setVol(self);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use enigo::Key::*;

  #[test]
  fn key_up() {
    let event = InputEvent::Keyboard(KeyboardEvent {
      action: KeyboardAction::Up,
      key: Alt,
    });
    assert_eq!(
      &serde_json::to_string(&event).unwrap(),
      r#"{"Keyboard":{"action":"Up","key":"Alt"}}"#
    );
  }
  #[test]
  fn key_click() {
    let event = InputEvent::Keyboard(KeyboardEvent {
      action: KeyboardAction::Click(Duration::from_millis(100)),
      key: Alt,
    });
    assert_eq!(
      &serde_json::to_string(&event).unwrap(),
      r#"{"Keyboard":{"action":{"Click":{"secs":0,"nanos":100000000}},"key":"Alt"}}"#
    );
  }
  #[test]
  fn audio() {
    let event = InputEvent::Volume(VolumeLevel::Frac(0.5));
    assert_eq!(
      &serde_json::to_string(&event).unwrap(),
      r#"{"Volume":{"Frac":0.5}}"#
    );
  }

  #[test]
  fn mouse() {
    let event = InputEvent::Mouse(MouseEvent::Move(MouseMoveEvent {
      origin: MouseMoveOrigin::Rel,
      x: 10,
      y: 15,
    }));
    assert_eq!(
      &serde_json::to_string(&event).unwrap(),
      r#"{"Mouse":{"Move":{"origin":"Rel","x":10,"y":15}}}"#
    );
  }
}
