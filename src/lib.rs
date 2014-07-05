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
 screen: *mut tsm_screen,
}

pub struct Vte {
  vte: *mut tsm_vte,
}

#[deriving(Show)]
pub enum ScreenError {
  OutOfMemory     = 12,
  InvalidArgument = 22,
  UnknownError    = 255
}

impl Screen {
  fn new(screen: *mut tsm_screen) -> Screen {
    Screen { screen: screen }
  }

  pub fn open() -> Result<Screen, ScreenError> {
    unsafe {
      let mut screen = ptr::mut_null();
      let err = tsm_screen_new(&mut screen, None, *ptr::mut_null());

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

  pub fn cursor_pos(&self) -> (u32,u32) {
    unsafe {
      let x = tsm_screen_get_cursor_x(self.screen);
      let y = tsm_screen_get_cursor_y(self.screen);
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
      let mut vte = ptr::mut_null();
      let err = tsm_vte_new(&mut vte, self.screen, None, *ptr::mut_null(), None, *ptr::mut_null());

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
  fn new(vte: *mut tsm_vte) -> Vte {
    Vte { vte: vte }
  }

  pub fn feed(&self, input: &[u8]) {
    unsafe { tsm_vte_input(self.vte, input.as_ptr(), input.len() as size_t) }
  }
}

fn error(err: i32) -> ScreenError {
  match err {
    ENOMEM => { OutOfMemory }
    EINVAL => { InvalidArgument }
    _ => { UnknownError }
  }
}
