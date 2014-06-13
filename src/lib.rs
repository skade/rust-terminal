#![crate_type = "lib"]
#![crate_id = "terminal#0.0.1"]
#![feature(globs,phase)]
#![phase(syntax, link)] extern crate log;

extern crate libc;

use std::io::IoError;
use libc::*;

use libc::consts::os::posix88::{EINVAL,ENOMEM};
use std::ptr;
use c_bits::libtsm::*;

pub mod c_bits;

pub struct Screen {
  pub screen: *tsm_screen,
}

pub struct Vte {
  vte: *tsm_vte,
}

#[deriving(Show)]
pub enum ScreenError {
  OutOfMemory     = 12,
  InvalidArgument = 22,
  UnknownError    = 255
}

impl Screen {
  fn new(screen: *tsm_screen) -> Screen {
    Screen { screen: screen }
  }

  pub fn open() -> Result<Screen, ScreenError> {
    unsafe {
      let screen = ptr::null();
      let err = tsm_screen_new(&screen, *ptr::null(), *ptr::null());

      if err > 0 {
        Err(error(err))
      } else {
        Ok(Screen::new(screen))
      }
    }
  }

  pub fn resize(&self, x: u32, y: u32) -> Result<(), ScreenError> {
    unsafe {
      let err = tsm_screen_resize(self.screen, x, y);

      if err > 0 {
        Err(error(err))
      } else {
        Ok(())
      }
    }
  }

  pub fn vte(&self) -> Result<Vte, ScreenError> {
    unsafe {
      let vte = ptr::null();
      let err = tsm_vte_new(&vte, self.screen, write_cb, *ptr::null(), *ptr::null(), *ptr::null());

      if err > 0 {
        Err(error(err))
      } else {
        Ok(Vte::new(vte))
      }
    }
  }

  pub fn draw(&self, draw_cb: tsm_screen_draw_cb, data: *mut c_void) -> u32 {
    unsafe { tsm_screen_draw(self.screen, draw_cb, data) }
  }
}

impl Vte {
  fn new(vte: *tsm_vte) -> Vte {
    Vte { vte: vte }
  }

  pub fn feed(&self, input: &[u8]) {
    unsafe { tsm_vte_input(self.vte, input.as_ptr(), input.len() as u64) }
  }
}

extern fn write_cb(vte: *tsm_vte, ch: *u8, len: size_t, data: c_void) {}

fn error(err: i32) -> ScreenError {
  match err {
    ENOMEM => { OutOfMemory }
    EINVAL => { InvalidArgument }
    _ => { UnknownError }
  }
}