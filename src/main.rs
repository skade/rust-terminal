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
  state: int
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
        let state_ptr = &mut state as *mut _ as *mut c_void;
        println!("{:i}", state.state)
        screen.draw(draw_cb, state_ptr);
        println!("{:i}", state.state)
      }
      'c' => {}
      _ => { fail!("unknown command!") }
    }
  }
}


extern "C" fn draw_cb(
  con: *const tsm_screen,
  id: u32,
  len: size_t,
  width: uint,
  posx: uint,
  posy: uint,
  attr: *const tsm_screen_attr,
  age: tsm_age_t,
  data: *mut c_void
) {
  unsafe {
    println!("there")
    let data = &mut *(data as *mut State);
    println!("there2")
    println!("{:i}", data.state)
    println!("there3")
    data.state = 10;
  };
}
