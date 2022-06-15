# matla

A manager for TLA+ projects by [OCamlPro][ocp].

Read the user manual [here][user manual].

## Features

- [x] does not require the TLA+ toolbox to be installed;
- [x] can use the toolbox from your environment;
- [x] uses a config directory `~/.config/matla` by default, but can run in portable mode with
  `--portable` (requires to have the TLA+ toobox jar in your path);
- [x] builds/runs everything in a build directory `target/debug` or `target/release`, meaning
  TLC's garbage files will not pollute your sources;
- [x] runs TLC;
- [x] partially processes TLC's output for readability;
- [ ] fully processes TLC's output for readability;
- [ ] reconstruct *"errors"* and pretty-present them
    - [x] basic parse errors
    - [x] assertion failures
    - [x] property/invariant falsification
    - [x] deadlocks
    - [ ] things I missed
- [x] provides a `Matla` module with assertions and checks, actually performed in `debug` mode but
  ignored in `release` mode;
- [ ] tla2tex-based *"doc"* generation;
- [ ] other (better, usable) doc generation;
- [ ] handles unit tests defined in your TLA+ modules;
- [ ] and more.

Non-features

- actually package the TLA+ toolbox in the binary.


## Install

Matla is not on [crates.io], you can still install it with `cargo` which you get when [installing rust].

Install from git (**not recommended** as long as the repo is private):

```text
> cargo install --git https://github.com/OCamlPro/matla#latest
...
```

Install from local clone (**recommended** as long as the repo is private):

```text
> git clone https://github.com/OCamlPro/matla
...
> cd matla
...
> cargo install --path matla
...
```

Make sure everything works:

```
> matla help
matla 0.1.0

Manager for TLA+ projects.

USAGE:
    matla [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -p, --portable    Infer toolchain from environment, load no user configuration
    -v                Increases verbosity, capped at 3
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -c, --color <true|on|false|off>    (De)activates colored output [default: on]

SUBCOMMANDS:
    clean        Cleans the current project: deletes the `target` directory.
    help         Prints this message or the help of the given subcommand(s)
    init         Initializes an existing directory as a matla project.
    run          Runs TLC on a TLA module in a project directory.
    setup        Performs this initial matla setup, required before running matla.
    test         Run the tests of a project.
    tlc          Calls TLC with some arguments.
    uninstall    Deletes your matla user directory (cannot be undone).
    update       Updates the `tla2tools` jar in the matla user directory.
```


## Configuration and Setup

Matla's configuration deals mostly with knowing how to call the TLA+ toolbox. Matla needs to create
a configuration folder to store information (or not, see the [portable mode](#portable-mode) below).
The path to this folder is `$HOME/.config/matla/`.

> TL;DR: run `matla setup` and answer matla's questions.

There are two ways matla handle the TLA+ toolbox. In `standalone` mode, matla will retrieve the
latest version of the toolbox and put it in your configuration folder, and use that anytime you call
it after setup. In this mode, `matla update` will make sure you have the latest version.

In `from_env` mode, matla retrieves the path to the toolbox from your path. If it fails to do so,
it will ask you for help. Naturally, this only works if the TLA+ toolbox is actually in your path.

While `matla setup` guides you towards setting up one of these modes, you can give it flags to tell
it what you want right away:

```text
> matla help setup
matla-setup
Performs this initial matla setup, required before running matla

USAGE:
    matla setup [FLAGS]

FLAGS:
        --from_env      Retrieve TLA toolbox path from the environment
    -o, --overwrite     Automatically overwrite config files when they exists
    -s, --standalone    Download the latest TLA toolbox to user directory and automatically use it
    -h, --help          Prints help information
    -V, --version       Prints version information
```

### Portable Mode

If you feel strongly about matla managing this user configuration folder, use `--portable` (or `-p`)
to let matla know: `matla --portable <other_my_arguments>`. Matla will not even look for a user
configuration folder and retrieve the TLA+ toolbox from the environment. Again, this means the
toolbox jar must be in your path.


## Init

`matla init <DIR>` initializes an *existing* project directory `<DIR>` (`.` by default). Pass
`--new` if you also want matla to create `<DIR>`. By default, `matla init <DIR>` will

- update the/create a `.gitignore` file in `<DIR>` with a rule to ignore the `target` (build)
  directory, and
- write the `Matla` module to `<DIR>/Matla.tla`.

This behavior can be changed by passing flags to `matla init`, see `matla help init` for details.

The `Matla` module contains helpers for writing assertions. In particular, it has a `dbg` sub-module
with helpers such as `Matla!dbg!assert(predicate, message)`. Helpers in `dbg` do nothing if you run
matla in `release` mode with `matla run --release`, as discussed below.


## Run

`matla run <MODULE_NAME>` runs TLC on your project with the entry point `<MODULE_NAME>`. Note that
`<MODULE_NAME>` must be *TLC-executable*, *i.e.* `<MODULE_NAME>.tla` and `<MODULE_NAME>.cfg` must
exist in your project directory.

If your project only has one TLC-executable module, `matla run` will automatically run on this
module.

Notable `matla run` command-line arguments:

- `--path`/`-p`: path to the project directory, `.` by default;
- `--release`: runs your project in release mode, see [below](#debugrelease).


### Debug/release

`matla run --release <MODULE_NAME>` runs TLC in *release* mode. As discussed in [`matla
init`](#init), matla comes with a `Matla` module providing helpers for writing assertions.
It features a `dbg` sub-module which behaves differently in *debug* and *release*.

> The `Matla` module is optional, if you don't want it make sure you initialize your project with
> `matla init --no_matla_module`.

If the `Matla` module is present, and you run your project in *debug* mode, then all assertion
helpers (in `Matla` and `Matla!dbg`) behave as expected: check some predicate(s) and fail with a
message if it is/they are not `TRUE`.

In *release* mode however, assertion helpers **in `Matla!dbg`** (**not** in `Matla`) are compiled
away: their definition is just `TRUE`.

If the `Matla` module is not present, there is currently no difference at all between *debug* mode
and *release* mode.



[crates.io]: https://crates.io
[installing rust]: https://www.rust-lang.org/tools/install
[user manual]: https://ocamlpro.github.io/matla/manual
[ocp]: https://ocamlpro.com
