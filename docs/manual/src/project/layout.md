# Project layout

> Matla project sources for this section available [here][run/ok].

Diving deeper, let's look at matla's project configuration file, `Matla.toml`:

```toml
{{ #include ../run/code/ok/Matla.toml}}
```

If you remember, this is [pretty much exactly what matla generates on setup as your user
configuration file](../setup/setup.md#homeconfigmatla). While users can customize how they want TLC
to behave (when calling matla) in their user configuration file, the project-level configuration
can override part or all of these settings. This can be useful to make sure all contributors use
the same TLC-level settings regardless of their user configuration such as seed, deadlock checking,
*etc.* if that makes sense for your project.


## The `Matla` module

The last file `matla init` generated is the `Matla.tla` file. This defines a `Matla` module which is
~385 lines long. Basically, this module provides functions for *asserting* things, *i.e.* wrappers
around calls to `TLC!Assert` as well as a few type-checking helpers. For clarity's sake, let's
discuss only one of the many `assert` variants.

```text
> bat -p Matla.tla
---- MODULE Matla ----
\* Matla helpers, mostly for assertions and debug/release conditional compilation.

\* TLC!Assertions are built on top of the standard `TLC' module.
LOCAL TLC == INSTANCE TLC
---- MODULE dbg ----
\* All functions in this module do nothing in `release' mode.

\* Checks some predicate.
assert(
    \* Predicate that must be true.
    pred,
    \* Message issued when the predicate is false.
    message
) ==
    TLC!Assert(pred, message)

====
\* End of module `dbg'.

\* Contains debug-only functions.
dbg == INSTANCE dbg

\* Checks some predicate.
\*
\* Active in debug and release.
assert(
    \* Predicate that must be true.
    pred,
    \* Message issued when the predicate is false.
    message
) ==
    TLC!Assert(pred, message)

====
\* End of module `Matla'.
```

This might be a bit odd: there are two versions of `assert` with exactly the same definition, but
one is in a `dbg` module under `Matla` while the other is at `Matla`'s top-level. The same goes for
all `assert` variants in the actual `Matla.tla`.

You can infer why this is by reading the comments, but basically it is tied to matla's *conditional
compilation* capabilities. As discussed earlier and detailed after, matla can run in either `debug`
mode or `release` mode. If, somewhere, we write `Matla!assert(...)` then regardless of the mode
matla runs in, the assertion will be evaluated and our run will crash if the assertion does not
hold.

`Matla!dbg!assert(...)` is the same but only in `debug` mode. In `release` mode, matla will compile
it away (use `TRUE` as its definition) to make your big, release-run faster.

> Technically, matla does not need to write this `Matla.tla` file here. It is actually ignored when
> matla runs, as matla generates whatever version correspond to the run mode (`debug`/`release`).
> The reason matla does generate this file is *i)* so that users can actually check what's in it
> and *ii)* to be compatible with IDEs that rely on the TLA+ toolbox to check your files and
> display errors if needed. Generating `Matla.tla` here essentially makes your code a legal TLC
> project.

Some matla users might not be interested in this conditional compilation feature. A quick look at
`matla help init` will lead you to the `--no_matla_module` flag which will do exactly what it
sounds like it's doing.


## Tests

Matla also recognizes the optional `tests` project sub-directory: this is where your integration
tests will reside. Let's forget about this for now as we will [discuss testing in details
later](../testing).
