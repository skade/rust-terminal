rust-terminal
=============

A full port of asciinemas `terminal.c` to Rust, also using a more recent version of libtsm.

Build with `make lib && make exe`

To test, pipe `test/reference_input` to `bin/main --height 21 --width 82`.

See `test/reference_output` for an example what the original program produces.
