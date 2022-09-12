#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ptr};

macro_rules! SAFE_RELEASE {
  ($punk: ident) => {{
    if let Some($punk) = unsafe { $punk.as_mut() } {
      unsafe {
        let _ = $punk.Release();
      }
    }
    $punk = std::ptr::null_mut();
  }};
}

use winapi::{
  um::{
    combaseapi::{CoCreateInstance, CoUninitialize, CLSCTX_ALL},
    endpointvolume::IAudioEndpointVolume,
    mmdeviceapi::{eConsole, eRender, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator},
    objbase::CoInitialize,
  },
  Class, Interface,
};

use crate::utils::Lerpable;

pub struct EndptVol<'a> {
  pEndptVol: &'a mut IAudioEndpointVolume,
  pEnumerator: &'a mut IMMDeviceEnumerator,
  pDevice: &'a mut IMMDevice,
  min: DeciBel,
  max: DeciBel,
  #[allow(dead_code)]
  step: DeciBel,
}

impl<'a> EndptVol<'a> {
  pub fn new() -> AudioResult<Self> {
    let mut pEndptVol: *mut IAudioEndpointVolume = ptr::null_mut();
    let mut pEnumerator: *mut IMMDeviceEnumerator = ptr::null_mut();
    let mut pDevice: *mut IMMDevice = ptr::null_mut();
    let mut min_db = 0f32;
    let mut max_db = 0f32;
    let mut step_db = 0f32;

    if init_impl(
      &mut pEndptVol,
      &mut pEnumerator,
      &mut pDevice,
      &mut min_db,
      &mut max_db,
      &mut step_db,
    )
    .is_none()
    {
      SAFE_RELEASE!(pEndptVol);
      SAFE_RELEASE!(pEnumerator);
      SAFE_RELEASE!(pDevice);
      unsafe {
        CoUninitialize();
      }
    }

    let pEndptVol = unsafe { &mut *pEndptVol };
    let pDevice = unsafe { &mut *pDevice };
    let pEnumerator = unsafe { &mut *pEnumerator };
    let min = DeciBel(min_db as f64);
    let max = DeciBel(max_db as f64);
    let step = DeciBel(step_db as f64);

    Ok(Self {
      pEndptVol,
      pDevice,
      pEnumerator,
      min,
      max,
      step,
    })
  }
  pub fn setVol(&mut self, vol: VolumeLevel) -> AudioResult<()> {
    if unsafe {
      self
        .pEndptVol
        .SetMasterVolumeLevel(vol.to_db(self.min, self.max).0 as f32, ptr::null())
    } < 0
    {
      return Err(AudioError::VolumeSetting);
    }
    Ok(())
  }
  pub fn getVol(&self) -> AudioResult<DeciBel> {
    let mut vol = 0.;
    if unsafe { self.pEndptVol.GetMasterVolumeLevel(&mut vol) } < 0 {
      return Err(AudioError::GetVolume);
    }
    Ok(DeciBel(vol as f64))
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, PartialOrd)]
pub struct DeciBel(f64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, PartialOrd)]
pub struct LinVolt(f64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VolumeLevel {
  Log(DeciBel),
  Lin(LinVolt),
  Frac(f64),
}

impl Display for DeciBel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{}db", self.0))
  }
}

impl Display for LinVolt {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{}V", self.0))
  }
}

impl Display for VolumeLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      VolumeLevel::Log(n) => f.write_fmt(format_args!("{}", n)),
      VolumeLevel::Lin(n) => f.write_fmt(format_args!("{}", n)),
      VolumeLevel::Frac(n) => f.write_fmt(format_args!("{}", n)),
    }
  }
}

impl VolumeLevel {
  pub fn to_db(self, min: DeciBel, max: DeciBel) -> DeciBel {
    DeciBel(
      match self {
        VolumeLevel::Log(db) => db,
        VolumeLevel::Lin(lv) => lv.into(),
        VolumeLevel::Frac(frac) => {
          LinVolt(frac.lerp(LinVolt::from(min).0, LinVolt::from(max).0)).into()
        }
      }
      .0
      .clamp(min.0, max.0),
    )
  }
  pub fn to_linear(self, min: DeciBel, max: DeciBel) -> LinVolt {
    LinVolt(
      match self {
        VolumeLevel::Log(db) => db.into(),
        VolumeLevel::Lin(lv) => lv,
        VolumeLevel::Frac(frac) => LinVolt(frac.lerp(LinVolt::from(min).0, LinVolt::from(max).0)),
      }
      .0
      .clamp(LinVolt::from(min).0, LinVolt::from(max).0),
    )
  }
  pub fn to_fraction(self, min: DeciBel, max: DeciBel) -> f64 {
    match self {
      VolumeLevel::Log(db) => LinVolt::from(db)
        .0
        .inv_lerp(LinVolt::from(min).0, LinVolt::from(max).0),
      VolumeLevel::Lin(lv) => lv.0.inv_lerp(LinVolt::from(min).0, LinVolt::from(max).0),
      VolumeLevel::Frac(frac) => frac,
    }
    .clamp(0., 1.)
  }
}

impl From<LinVolt> for DeciBel {
  fn from(lv: LinVolt) -> Self {
    Self(20. * lv.0.log10())
  }
}

impl From<DeciBel> for LinVolt {
  fn from(db: DeciBel) -> Self {
    Self(10f64.powf(db.0 / 20.))
  }
}

impl<'a> Drop for EndptVol<'a> {
  fn drop(&mut self) {
    unsafe {
      let _ = self.pEndptVol.Release();
    }
    unsafe {
      let _ = self.pDevice.Release();
    }
    unsafe {
      let _ = self.pEnumerator.Release();
    }
    unsafe {
      CoUninitialize();
    }
  }
}

fn init_impl(
  pEndptVol: &mut *mut IAudioEndpointVolume,
  pEnumerator: &mut *mut IMMDeviceEnumerator,
  pDevice: &mut *mut IMMDevice,
  min_db: &mut f32,
  max_db: &mut f32,
  step_db: &mut f32,
) -> Option<()> {
  if unsafe { CoInitialize(ptr::null_mut()) } < 0 {
    return None;
  }

  let ppEnumerator: *mut _ = pEnumerator;

  if unsafe {
    CoCreateInstance(
      &MMDeviceEnumerator::uuidof(),
      ptr::null_mut(),
      1,
      &IMMDeviceEnumerator::uuidof(),
      ppEnumerator as *mut _,
    ) < 0
  } {
    return None;
  }

  if unsafe {
    pEnumerator
      .as_ref()?
      .GetDefaultAudioEndpoint(eRender, eConsole, pDevice)
      < 0
  } {
    return None;
  }

  let ppEndptVol: *mut _ = pEndptVol;

  if unsafe {
    pDevice.as_ref()?.Activate(
      &IAudioEndpointVolume::uuidof(),
      CLSCTX_ALL,
      ptr::null_mut(),
      ppEndptVol as *mut _,
    ) < 0
  } {
    return None;
  }

  if unsafe { pEndptVol.as_ref()?.GetVolumeRange(min_db, max_db, step_db) } < 0
    || *max_db < *min_db
    || *step_db > (*max_db - *min_db).abs()
  {
    return None;
  }

  Some(())
}

#[derive(Debug, Clone, Copy)]
pub enum AudioError {
  Initialization,
  VolumeSetting,
  GetVolume,
}

impl Display for AudioError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use AudioError::*;
    f.write_str(match *self {
      Initialization => "Initialization failed",
      VolumeSetting => "Set volume failed",
      GetVolume => "Get volume failed",
    })
  }
}

impl std::error::Error for AudioError {}

pub type AudioResult<T> = Result<T, AudioError>;
