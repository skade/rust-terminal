#![feature(globs)]
extern crate terminal;
extern crate libc;

use terminal::Screen;
use terminal::c_bits::libtsm::*;
use libc::*;
use std::str::from_utf8;
use std::mem::transmute;

#[deriving(Show,Copy)]
struct State {
  state: u32
}

fn main() {
  let mut stdin = std::io::stdio::stdin();
  let width = 82;
  let height = 21;

  let screen_opt = Screen::open();
  let screen = screen_opt.unwrap();
  let resize = screen.resize(width,height);

  let vte = screen.vte().unwrap();
  let mut state = State { state: 1 };

  loop {
    let line = stdin.read_line();
    if line.is_err() {
      return;
    }
    match line.unwrap().as_slice().chars().next().unwrap() {
      'd' => {
        let next_line = stdin.read_line();
        if next_line.is_ok() {
          let n: uint = from_str(next_line.unwrap().as_slice().trim()).unwrap();
          let result = stdin.read_exact(n);
          vte.feed(result.unwrap().as_slice())
        } else {
          return;
        }
      }
      'p' => {
        let state_ptr: *mut c_void = &mut state as *mut _ as *mut c_void;
        println!("before: {:u}", state.state)
        screen.draw(draw_cb, state_ptr);
        println!("after: {:u}", state.state)
      }
      'c' => {}
      _ => { fail!("unknown command!") }
    }
  }
}


extern "C" fn draw_cb(
  con: *const tsm_screen,
  id: u32,
  ch: *const uint32_t,
  len: size_t,
  width: c_uint,
  posx: c_uint,
  posy: c_uint,
  attr: *const tsm_screen_attr,
  age: tsm_age_t,
  data: *mut c_void
) {
  unsafe {
    let data: &mut State = &mut *(data as *mut State);
    //println!("{:u}", data.state)
    data.state = data.state + 1;
    //println!("{:u}", data.state)
  };
}
