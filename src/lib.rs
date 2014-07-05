#![crate_type = "lib"]
#![crate_id = "terminal#0.0.1"]
#![feature(globs,phase)]

extern crate libc;
extern crate serialize;
extern crate collections;

use libc::{c_void,size_t};
use libc::consts::os::posix88::{EINVAL,ENOMEM};
use std::ptr;
use c_bits::libtsm::*;

pub mod c_bits;

pub struct Screen {
  pub screen: *const tsm_screen,
}

pub struct Vte {
  vte: *const tsm_vte,
}

#[deriving(Show)]
pub enum ScreenError {
  OutOfMemory     = 12,
  InvalidArgument = 22,
  UnknownError    = 255
}

impl Screen {
  fn new(screen: *const tsm_screen) -> Screen {
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

  pub fn cursor_pos(&self) -> (int,int) {
    unsafe {
      let x = tsm_screen_get_cursor_x(self.screen) as int;
      let y = tsm_screen_get_cursor_y(self.screen) as int;
      (x,y)
    }
  }

  pub fn cursor_visible(&self) -> bool {
    unsafe {
      let flags = tsm_screen_get_flags(self.screen);
      !flags.contains_elem(TSM_SCREEN_HIDE_CURSOR)
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
  fn new(vte: *const tsm_vte) -> Vte {
    Vte { vte: vte }
  }

  pub fn feed(&self, input: &[u8]) {
    unsafe { tsm_vte_input(self.vte, input.as_ptr(), input.len() as u64) }
  }
}

extern fn write_cb(_: *const tsm_vte, _: *const u8, _: size_t, _: c_void) {}

fn error(err: i32) -> ScreenError {
  match err {
    ENOMEM => { OutOfMemory }
    EINVAL => { InvalidArgument }
    _ => { UnknownError }
  }
}
