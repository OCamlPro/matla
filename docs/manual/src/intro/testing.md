# Motivation: Testing

As discussed previously, testing is mandatory as it raises significantly the confidence in the
encoding, the invariants and properties, and thus the final analysis and its outcome. We wanted
projects to have an optional `tests` directory, separated from the actual codebase, where sanity
checks, regression tests *etc.* can be. These tests are akin to integration tests; on the other
hand, unit tests should live in source files in the actual codebase using special syntax to be
compiled away in the final analysis/es. Documentation tests and compiling/running/checking them
would also be very useful, both as a means of documentation and for catching bugs.

Obviously, we want to be able to check tests against an expected result. Matla's tests needed to
include a way for users to specify if the test is expected to succeed, fail at compile-time and how,
or fail at run-time and how ---invariant violation, temporal violation, type-checking error,
assertion failure *etc.*
