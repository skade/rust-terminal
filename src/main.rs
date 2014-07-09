#![feature(globs)]
extern crate terminal;
extern crate libc;
extern crate serialize;
extern crate collections;
extern crate getopts;

use getopts::{reqopt,getopts};
use std::os;
use std::io::IoResult;
use std::io::EndOfFile;
use terminal::{Screen,Vte,ScreenError};
use terminal::c_bits::libtsm::*;
use libc::{c_uint,c_void,size_t,uint32_t};
use std::char::from_u32;
use std::str::from_char;
use serialize::{Encodable,Encoder};
use serialize::json;

#[deriving(Show,Copy)]
struct State {
  rendered: Vec<Vec<(String, Attribute)>>
}

#[deriving(Show,Encodable)]
struct CursorState {
  x: u32,
  y: u32,
  visible: bool
}

#[deriving(PartialEq,Show)]
struct Attribute {
  fg: Option<u8>,
  bg: Option<u8>,
  bold: bool,
  underline: bool,
  inverse: bool,
  blink: bool
}

fn main() {
  let args: Vec<String> = os::args();

  let opts = [
      reqopt("h", "height", "the height of the console", "HEIGHT"),
      reqopt("w", "width", "the width of the console", "WIDTH")
  ];
  let matches = match getopts(args.tail(), opts) {
     Ok(m) => { m }
     Err(f) => { fail!("{}", f) }
  };
  let width: u32 = from_str(matches.opt_str("w").unwrap().as_slice()).unwrap();
  let height: u32 = from_str(matches.opt_str("h").unwrap().as_slice()).unwrap();

  let (screen, vte) = match setup_screen(width, height) {
    Err(reason) => { fail!("screen setup failed for {}", reason) }
    Ok(v) => v
  };

  match run(screen, vte) {
    Ok(()) => {},
    Err(reason) => {
      if !(reason.kind == EndOfFile) {
        fail!("runloop failed for {}", reason)
      }
    }
  }
}

fn setup_screen(width: u32, height: u32) -> Result<(Screen,Vte), ScreenError> {
  let screen = try!(Screen::open());
  try!(screen.resize(width,height));
  let vte = try!(screen.vte());
  Ok((screen,vte))
}

fn run(screen: Screen, vte: Vte) -> IoResult<()> {
  let mut stdin = std::io::stdio::stdin();

  loop {
    let line = try!(stdin.read_line());

    match line.as_slice().chars().next() {
      Some('d') => {
        let next_line = try!(stdin.read_line());
        let n: uint = from_str(next_line.as_slice().trim()).expect("d must be followed by int");
        let result = try!(stdin.read_exact(n));
        vte.feed(result.as_slice())
      }
      Some('p') => {
        let mut state = State { rendered: vec![] };
        let state_ptr: *mut c_void = &mut state as *mut _ as *mut c_void;
        screen.draw(draw_cb, state_ptr);
        let encoded = json::encode(&state);
        println!("{}", encoded)
      }
      Some('c') => {
        let (x,y) = screen.cursor_pos();
        let visible = screen.cursor_visible();
        let c_state = CursorState { x: x, y: y, visible: visible };
        let encoded = json::encode(&c_state);
        println!("{}", encoded)
      }
      None => { fail!("empty input!") }
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
    let element = (from_char(ch), attr);
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
    let fields_to_emit = vec![self.fg.is_some(), self.bg.is_some(), self.bold, self.underline, self.inverse, self.blink];
    let no_of_fields_to_emit = fields_to_emit.iter().filter(|val| **val).count();

    e.emit_map(no_of_fields_to_emit, |e| {
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
      data.rendered.push(vec![(from_char(current_char), attribute)]);
      return
    }

    let added = data.push_if_same(current_char, attribute);
    if !added {
      data.push_new_cell(current_char, attribute);
    }
  };
}
