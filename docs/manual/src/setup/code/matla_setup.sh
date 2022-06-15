# ANCHOR: part_1_of_4
> matla setup
|===| TLA+ toolchain setup
| Config will live in `~/.config/matla`, okay to create this directory? [Yn]
# ANCHOR_END: part_1_of_4
# ANCHOR: part_2_of_4
| y
|
| Matla can either:
| - retrieve the tla2tools jar from your environment, or
| - download it for you.
| Download the tla2tools to `~/.config/matla`? If not, matla will attempt to find it in your path [Yn]
# ANCHOR_END: part_2_of_4
# ANCHOR: part_3_of_4
| y
| Downloading toolbox from `https://github.com/tlaplus/tlaplus/releases/latest/download/tla2tools.jar`...
| Download completed successfully.
| Writing downloaded file to `~/.config/matla/tla2tools.jar`...
# ANCHOR_END: part_3_of_4
# ANCHOR: part_4_of_4
|
| Writing configuration file to user directory, its content is:
|
| ```
| [config]
| tla2tools = '/Users/adrien/.config/matla/tla2tools.jar'
| [tlc_cla]
| # # Full configuration for TLC runtime arguments customization
| #
| # # Sets the number of workers, `0` or `auto` for `auto`.
| # workers = 'auto' # <int|'auto'>#
| # # If active, counterexample traces will only display state variables when they change.
| # diff_cexs = 'on' # <'on'|'off'|'true'|'false'>#
| # # Sets the seed when running TLC, random if none.
| # seed = 0 # <int|'random'>#
| # # If active, TLC will not output print statements.
| # terse = 'off' # <'on'|'off'|'true'|'false'>#
| # # Maximum size of the sets TLC is allowed to enumerate.
| # max_set_size = 'default' # <u64|'default'>#
| # # If active, TLC will check for (and fail on) deadlocks.
| # check_deadlocks = 'on' # <'on'|'off'|'true'|'false'>#
| # # If active, matla will present the callstack on errors, whenever possible.
| # print_callstack = 'off' # <'on'|'off'|'true'|'false'>#
| # # If active, matla will present time statistics during runs.
| # print_timestats = 'on' # <'on'|'off'|'true'|'false'>
| ```
|
| Configuration regarding `tlc_cla` (TLC command-line arguments) corresponds to
| options for `matla run`. You can check them out with `matla help run`.
| The configuration above corresponds to matla's defaults, and all items are optional.
|
| Setup complete, matla is ready to go.
|===|
# ANCHOR_END: part_4_of_4