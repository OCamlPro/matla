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