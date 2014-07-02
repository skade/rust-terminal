#![feature(globs)]
extern crate terminal;
extern crate libc;

use terminal::Screen;
use terminal::c_bits::libtsm::*;
use libc::{c_uint,c_void,size_t,uint32_t};

#[deriving(Show,Copy)]
struct State {
  last_attribute: Option<tsm_screen_attr>,
}

fn main() {
  let mut stdin = std::io::stdio::stdin();
  let width = 82;
  let height = 21;

  let screen_opt = Screen::open();
  let screen = screen_opt.unwrap();
  let resize = screen.resize(width,height);

  let vte = screen.vte().unwrap();
  let mut state = State { last_attribute: None};

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
        screen.draw(draw_cb, state_ptr);
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
    println!{"{}", *attr}
    let mut last_attribute;
    if data.last_attribute.is_some() {
      last_attribute = data.last_attribute.unwrap();
      println!{"{}", last_attribute}
      println!("{:b}", last_attribute == *attr)
    }
    data.last_attribute = Some(*attr.clone());
  };
}
