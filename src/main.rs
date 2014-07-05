#![feature(globs)]
extern crate terminal;
extern crate libc;
extern crate serialize;
extern crate collections;

use terminal::Screen;
use terminal::c_bits::libtsm::*;
use libc::{c_uint,c_void,size_t,uint32_t};
use std::char::from_u32;
use serialize::{Encodable,Encoder};
use serialize::json;

#[deriving(Show,Copy)]
struct State {
  rendered: Vec<Vec<(String, Attribute)>>
}

#[deriving(Show,Encodable)]
struct CursorState {
  x: int,
  y: int,
  visible: bool
}

#[deriving(PartialEq,Show)]
struct Attribute {
  fg: Option<i8>,
  bg: Option<i8>,
  bold: bool,
  underline: bool,
  inverse: bool,
  blink: bool
}

fn main() {
  let mut stdin = std::io::stdio::stdin();
  let width = 82;
  let height = 21;

  let screen_opt = Screen::open();
  let screen = screen_opt.unwrap();
  let resize = screen.resize(width,height);
  match resize {
    Ok(()) => {},
    Err(_reason) => { fail!("error resizing screen") }
  }

  let vte = screen.vte().unwrap();

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
        let mut state = State { rendered: vec![] };
        let state_ptr: *mut c_void = &mut state as *mut _ as *mut c_void;
        screen.draw(draw_cb, state_ptr);
        let encoded = json::encode(&state);
        println!("{}", encoded)
      }
      'c' => {
        let (x,y) = screen.cursor_pos();
        let visible = screen.cursor_visible();
        let c_state = CursorState { x: x, y: y, visible: visible };
        let encoded = json::encode(&c_state);
        println!("{}", encoded)
      }
      _ => { fail!("unknown command!") }
    }
  }
}

impl State {
  fn push_if_same(&mut self, ch: char, attr: Attribute) -> bool {
    let current_line  = self.rendered.mut_last().unwrap();
    let current_stream = current_line.mut_last().unwrap();
    if current_stream.ref1() == &attr {
      current_stream.mut0().push_char(ch);
      true
    } else {
      false
    }
  }

  fn push_new_cell(&mut self, ch: char, attr: Attribute) {
    let current_line  = self.rendered.mut_last().unwrap();
    let element = (ch.to_str().to_string(), attr);
    current_line.push(element);
  }
}

impl<S: Encoder<E>, E> Encodable<S, E> for State {
  fn encode(&self, s: &mut S) -> Result<(), E> {
    self.rendered.encode(s)
  }
}

impl Attribute {
  fn new(attr: tsm_screen_attr) -> Attribute {
    Attribute {
      fg: attr.get_fg(),
      bg: attr.get_bg(),
      bold: attr.get_flag(Bold),
      underline: attr.get_flag(Underline),
      inverse: attr.get_flag(Inverse),
      blink: attr.get_flag(Blink)
    }
  }
}

impl<S: Encoder<E>, E> Encodable<S, E> for Attribute {
  fn encode(&self, e: &mut S) -> Result<(), E> {
    let mut emit_length: uint = 0;
    if self.fg.is_some() {
      emit_length = emit_length + 1
    }
    if self.bg.is_some() {
      emit_length = emit_length + 1
    }
    if self.bold {
      emit_length = emit_length + 1
    }
    if self.underline {
      emit_length = emit_length + 1
    }
    if self.inverse {
      emit_length = emit_length + 1
    }
    if self.blink {
      emit_length = emit_length + 1
    }

    e.emit_map(emit_length, |e| {
      let mut key_pos = 0;
      if self.fg.is_some() {
        try!(e.emit_map_elt_key(key_pos, |e| "fg".encode(e)));
        try!(e.emit_map_elt_val(key_pos, |e| self.fg.encode(e)));
        key_pos = key_pos + 1;
      }
      if self.bg.is_some() {
        try!(e.emit_map_elt_key(key_pos, |e| "bg".encode(e)));
        try!(e.emit_map_elt_val(key_pos, |e| self.bg.encode(e)));
        key_pos = key_pos + 1;
      }
      if self.bold {
        try!(e.emit_map_elt_key(key_pos, |e| "bold".encode(e)));
        try!(e.emit_map_elt_val(key_pos, |e| self.bold.encode(e)));
        key_pos = key_pos + 1;
      }
      if self.underline {
        try!(e.emit_map_elt_key(key_pos, |e| "underline".encode(e)));
        try!(e.emit_map_elt_val(key_pos, |e| self.underline.encode(e)));
        key_pos = key_pos + 1;
      }
      if self.inverse {
        try!(e.emit_map_elt_key(key_pos, |e| "inverse".encode(e)));
        try!(e.emit_map_elt_val(key_pos, |e| self.inverse.encode(e)));
        key_pos = key_pos + 1;
      }
      if self.blink {
        try!(e.emit_map_elt_key(key_pos, |e| "blink".encode(e)));
        try!(e.emit_map_elt_val(key_pos, |e| self.blink.encode(e)));
      }

      Ok(())
    })
  }
}

extern "C" fn draw_cb(
  _: *const tsm_screen,
  _: u32,
  ch: *const uint32_t,
  _: size_t,
  _: c_uint,
  _: c_uint,
  posy: c_uint,
  attr: *const tsm_screen_attr,
  _: tsm_age_t,
  data: *mut c_void
) {
  unsafe {
    let data: &mut State = &mut *(data as *mut State);
    let mut current_char;
    let attribute = Attribute::new(*attr);

    if *ch == 0 {
      current_char = ' ';
    } else {
      current_char = from_u32(*ch).unwrap();
    }

    if data.rendered.len() == (posy as uint) {
      data.rendered.push(vec![(current_char.to_str().to_string(), attribute)]);
      return
    }

    let added = data.push_if_same(current_char, attribute);
    if !added {
      data.push_new_cell(current_char, attribute);
    }
  };
}
