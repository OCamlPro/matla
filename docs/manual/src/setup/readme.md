# Install, build, setup and portable mode

This chapter covers the basics of installing and building matla, as well as its initial (optional)
setup.

## Downloading a release

Probably the easiest way to use matla is to download the latest release at

- <https://github.com/OCamlPro/matla/releases>

Put it wherever is convenient for you, ideally in your `$PATH`, and you're good to go. Now, this
installation method is not great for updating matla as it requires manually checking whether a new
version is available. The following installation method is arguably more convenient in that regard.


## Using cargo

Assuming you have [Rust](https://www.rust-lang.org/tools/install) installed or are willing to install it, you can use `cargo` to handle matla's installation for you.

Now, matla is **not** published as a [crates.io](https://crates.io) package. As such, Rust's usual
`cargo install matla` will not work; to install matla, please provide the repository's URL
explicitly as follows.

```bash
{{ #include code/cargo_install_git.sh }}
```

To update matla, simply run the same command with `-f` to force the update:

```bash
> cargo install -f https://github.com/...
```

Alternatively and if you are a frequent Rust flyer, consider using the extremely convenient
[`cargo-update`](https://github.com/nabijaczleweli/cargo-update) cargo plugin that can update
outdated binary Rust crates for you.

