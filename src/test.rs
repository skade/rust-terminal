extern crate terminal;

#[cfg(test)]
mod tests {
  use terminal::Screen;

  #[test]
  fn initialize_screen() {
    let screen = Screen::open();
    assert!(screen.is_ok())
  }

  #[test]
  fn resize_screen() {
    let screen = Screen::open();
    let resize = screen.unwrap().resize(20,20);
    assert!(resize.is_ok())
  }

  #[test]
  fn get_vte() {
    let screen = Screen::open();
    let vte = screen.unwrap().vte();
    assert!(vte.is_ok())
  }
}