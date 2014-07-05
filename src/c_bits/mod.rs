#![allow(non_camel_case_types)]

pub mod libtsm {
  use std::num::{FromPrimitive};
  use libc::{c_void, c_uint, c_int, size_t, uint32_t, c_char};
  use collections::enum_set::{EnumSet,CLike};

  static RGB_LEVELS : &'static [u8] = &[0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];

  #[deriving(PartialEq,Show,Clone,FromPrimitive)]
  #[repr(u8)]
  pub enum AttributeFlags {
    Bold,
    Underline,
    Inverse,
    Protect,
    Blink
  }

  #[deriving(PartialEq,Show,Clone,FromPrimitive)]
  #[repr(u8)]
  pub enum ScreenFlags {
    TSM_SCREEN_INSERT_MODE,
    TSM_SCREEN_AUTO_WRAP,
    TSM_SCREEN_REL_ORIGIN,
    TSM_SCREEN_INVERSE,
    TSM_SCREEN_HIDE_CURSOR,
    TSM_SCREEN_FIXED_POS,
    TSM_SCREEN_ALTERNATE
  }

  impl CLike for AttributeFlags {
    fn to_uint(&self) -> uint {
      *self as uint
    }

    fn from_uint(v: uint) -> AttributeFlags {
      match FromPrimitive::from_uint(v) {
        Some(v) => v,
        None => fail!("Decoding of uint into AttributeFlags failed!")
      }
    }
  }

  impl CLike for ScreenFlags {
    fn to_uint(&self) -> uint {
      *self as uint
    }
    fn from_uint(v: uint) -> ScreenFlags {
      match FromPrimitive::from_uint(v) {
        Some(v) => v,
        None => fail!("Decoding of uint into ScreenFlags failed!")
      }
    }
  }

  #[deriving(PartialEq,Show,Clone)]
  pub struct tsm_screen_attr {
    pub fccode: i8, /* foreground color code or <0 for rgb */
    pub bccode: i8,      /* background color code or <0 for rgb */
    pub fr: u8,     /* foreground red */
    pub fg: u8,      /* foreground green */
    pub fb: u8,     /* foreground blue */
    pub br: u8,     /* background red */
    pub bg: u8,     /* background green */
    pub bb: u8,     /* background blue */
    pub flags: EnumSet<AttributeFlags>,    /* misc flags */
  }

  fn get_color(code: i8, r: u8, g: u8, b: u8) -> Option<i8> {
    if code == -1 {
      Some(get_rgb_color(r, g, b) as i8)
    } else if code >= 0 && code < 16 {
      Some(code)
    } else {
      None
    }
  }

  fn get_rgb_color(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b && (r - 8) % 10 == 0 {
        return 232 + (r - 8) / 10;
    } else {
        return 16 + get_rgb_index(r) * 36 + get_rgb_index(g) * 6 + get_rgb_index(b);
    }
  }

  fn get_rgb_index(value: u8) -> u8 {
    RGB_LEVELS.iter().position(|level| *level == value).unwrap() as u8
  }

  impl tsm_screen_attr {
    pub fn get_fg(&self) -> Option<i8> {
      get_color(self.fccode, self.fr, self.fg, self.fb)
    }

    pub fn get_bg(&self) -> Option<i8> {
      get_color(self.bccode, self.br, self.bg, self.bb)
    }

    pub fn get_flag(&self, flag: AttributeFlags) -> bool {
      self.flags.contains_elem(flag)
    }
  }

  pub struct tsm_screen;
  pub struct tsm_vte;
  pub struct log_data;
  pub struct tsm_vte_write_cb;
  pub struct va_list;

  pub type tsm_age_t = u32;
  pub type tsm_write_cb = extern "C" fn(*const tsm_vte, *const u8, size_t, c_void);
  pub type tsm_screen_draw_cb = extern "C" fn(
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
  );

  pub type tsm_log_t = extern "C" fn(
    data: *mut c_void,
    line: c_int,
    func: *const c_char,
    subs: *const c_char,
    sev: c_uint,
    format: *const c_char,
    args: va_list
  );

  extern {
    pub fn tsm_screen_new(out: &mut *mut tsm_screen, log: Option<tsm_log_t>, log_data: c_void) -> c_int;
    pub fn tsm_screen_resize(con: *mut tsm_screen, x: c_uint, y: c_uint) -> c_int;
    pub fn tsm_vte_new(out: &mut *mut tsm_vte, con: *mut tsm_screen, write_cb: Option<tsm_write_cb>,
                   data: c_void, log: Option<tsm_log_t>, log_data: c_void) -> c_int;
    pub fn tsm_vte_input(vte: *mut tsm_vte, input: *const u8, len: size_t);
    pub fn tsm_screen_draw(con: *mut tsm_screen, draw_cb: tsm_screen_draw_cb, data: *mut c_void) -> tsm_age_t;

    pub fn tsm_screen_get_cursor_x(con: *mut tsm_screen) -> c_uint;
    pub fn tsm_screen_get_cursor_y(con: *mut tsm_screen) -> c_uint;
    pub fn tsm_screen_get_flags(con: *mut tsm_screen) -> EnumSet<ScreenFlags>;
  }
}
