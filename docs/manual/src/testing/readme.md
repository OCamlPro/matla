# Testing

As developers, (doc/unit/integration/binary) testing is the main way we convince ourselves that our
code does what we expect. At least until we all develop in [lean](https://leanprover.github.io) or
something similar.

Formal methods in general and formal specification and verification in particular exist to provide
strong, proof-based guarantees. It is thus crucial to make sure the specification makes sense and
behaves the way we want it to so that successful analyses actually mean something.

Currently, matla only supports *integration testing*. That is, tests that reside outside of your
project sources in a separate `tests` folder. Documentation/unit testing on the other hand would
typically live among your project's code. Matla does not support those just yet as, for now,
matla's design makes sure that your matla-project's sources are compatible with TLC: you can just
run TLC manually just like you would on any TLA+ codebase. This will probably change eventually,
but for now this constraint makes it difficult to decide exactly what the best way to provide
doc/unit testing is.
