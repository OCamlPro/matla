# Motivation: conditional compilation

This is actually the main reason we started developing matla. If you are familiar with TLA+, you
know it is a dynamically-typed language. Static typing, and especially static strong-typing, is
basically a static analysis that makes sure (state) variables cannot store values of unexpected
types. Dynamically-typed languages such as TLA+ are more flexible that statically-typed ones in that
variables can end up storing anything as the program runs. Whenever a primitive operation is applied
to a value, the runtime (TLC, here) checks the application is legal; if it is not, a runtime error
is generated.

Many a TLA+ developer have issues with this aspect of TLA+. On one hand, static typing means the
program will not compile because someone stored a nonsensical value in a variable, which raises the
trust in the TLA+ code and thus the analysis and its outcome. Also, this means a lengthy analysis
(hours, days, or even weeks) cannot fail because, say, for some reason `x` in `x + 1` happens to
store a string; meaning the bug must be fixed and the lengthy analysis must restart from scratch. On
the other hand, dynamic typing offers flexibility such as being able to build heterogeneous
lists/arrays.

Still, TLA+/TLC are what they are: dynamically-typed. As a consequence, static-typing fanatics like
us tend to **heavily** annotate their TLA+ code with type-checking *assertions*. Typically,
function definitions will start with a check that the arguments have the expected type to avoid
potentially misleading errors such as *"cannot compute length of integer"* with a more or less
relevant location.

Our TLA+ projects tend to have *a lot* of checks like these; especially since besides
type-checking, one usually also checks for structural invariants of the encoding as those also
greatly raise the trust in the relevance of any analysis.

While tedious at times, writing these assertions is a good exercise and we have little to no
complaints about that. This does change when we run the final analysis however. All our assertions
help us develop, sanity-check, debug, catch regressions... but we generally don't want them to run
in the final analyses. On large projects, TLC's analyses can take very long; checking each
assertion in this context might make sense for a few of them, but on the whole they tend to make
analyses take much, much, **much** longer.

Hence, we want to have a mechanism for *debug assertions*, very similar to [Rust's `debug_assert`
macros](https://doc.rust-lang.org/std/macro.debug_assert.html)). Users should then be able to run
analyses (and tests!) in `debug` or `release` mode, with debug assertions only active in `debug`
and compiled away in `release`.
