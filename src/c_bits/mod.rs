pub mod libtsm {
  use libc::*;

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
    pub flags: c_uint,    /* bold character */
  }

  pub struct tsm_screen;
  pub struct tsm_vte;
  pub struct tsm_log_t;
  pub struct log_data;
  pub struct tsm_vte_write_cb;

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

  extern {
    pub fn tsm_screen_new(out: *const *const tsm_screen, log: tsm_log_t, log_data: c_void) -> c_int;
    pub fn tsm_screen_resize(con: *const tsm_screen, x: c_uint, y: c_uint) -> c_int;
    pub fn tsm_vte_new(out: *const *const tsm_vte, con: *const tsm_screen, write_cb: tsm_write_cb,
                   data: c_void, log: tsm_log_t, log_data: c_void) -> c_int;
    pub fn tsm_vte_input(vte: *const tsm_vte, input: *const u8, len: size_t);
    pub fn tsm_screen_draw(con: *const tsm_screen, draw_cb: tsm_screen_draw_cb, data: *mut c_void) -> tsm_age_t;
  }
}