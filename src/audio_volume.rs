#![allow(non_snake_case)]
use std::{fmt::Display, ptr};

macro_rules! SAFE_RELEASE {
  ($punk: ident) => {{
    if let Some($punk) = unsafe { $punk.as_mut() } {
      unsafe {
        $punk.Release();
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

pub struct AudioEndpointVolume<'a> {
  pEndptVol: &'a mut IAudioEndpointVolume,
  pEnumerator: &'a mut IMMDeviceEnumerator,
  pDevice: &'a mut IMMDevice,
  min_db: f64,
  max_db: f64,
  #[allow(dead_code)]
  step_db: f64,
}

impl<'a> AudioEndpointVolume<'a> {
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
    let min_db = min_db as f64;
    let max_db = max_db as f64;
    let step_db = step_db as f64;

    Ok(Self {
      pEndptVol,
      pDevice,
      pEnumerator,
      min_db,
      max_db,
      step_db,
    })
  }
  pub fn setVol(&mut self, vol: f64) -> AudioResult<()> {
    if vol < 0.
      || vol > 1.
      || unsafe {
        self.pEndptVol.SetMasterVolumeLevel(
          (vol * (self.max_db - self.min_db) + self.min_db) as f32,
          ptr::null(),
        )
      } < 0
    {
      return Err(AudioError::InitFailed);
    }
    Ok(())
  }
  pub fn getVol(&self) -> AudioResult<f64> {
    let mut vol = 0.;
    if unsafe { self.pEndptVol.GetMasterVolumeLevel(&mut vol) } < 0
      || vol < self.min_db as f32
      || vol > self.max_db as f32
    {
      return Err(AudioError::InitFailed);
    }
    let vol = (vol as f64 - self.min_db) / (self.max_db - self.min_db);

    Ok(vol)
  }
}

impl<'a> Drop for AudioEndpointVolume<'a> {
  fn drop(&mut self) {
    unsafe {
      self.pEndptVol.Release();
    }
    unsafe {
      self.pDevice.Release();
    }
    unsafe {
      self.pEnumerator.Release();
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

  if unsafe {
    CoCreateInstance(
      &MMDeviceEnumerator::uuidof() as *const _,
      ptr::null_mut(),
      1,
      &IMMDeviceEnumerator::uuidof() as *const _,
      pEnumerator as *mut _ as *mut _,
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

  if unsafe {
    pDevice.as_ref()?.Activate(
      &IAudioEndpointVolume::uuidof() as *const _,
      CLSCTX_ALL,
      ptr::null_mut(),
      pEndptVol as *mut _ as *mut _,
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
  InitFailed,
}

impl Display for AudioError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use AudioError::*;
    f.write_str(match *self {
      InitFailed => "initialization failed",
    })
  }
}

impl std::error::Error for AudioError {}

pub type AudioResult<T> = Result<T, AudioError>;
