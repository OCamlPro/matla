# Build from sources

Building matla from sources is quite easy and only requires [Rust to be
installed](https://www.rust-lang.org/tools/install).

Simply clone the repository, change to whatever branch you want to build, and `cargo build` or
`cargo build --release` matla.

```bash
{{ #include code/cargo_build.sh }}
```

Move/symlink the resulting binary as you see fit and start writing TLA+ projects using matla!

Alternatively, run `cargo install --path matla` to have cargo handle compilation and putting the
binary in your path.
