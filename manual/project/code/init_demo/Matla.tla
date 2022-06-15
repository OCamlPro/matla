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

\* Checks that two values are equal.
assert_eq(
    val_1,
    val_2,
    \* Message issued when `val_1' and `val_2' are not equal.
    message
) ==
    TLC!Assert(val_1 = val_2, message)

\* Checks that two values are different.
assert_ne(
    val_1,
    val_2,
    \* Message issued when `val_1' and `val_2' are equal.
    message
) ==
    TLC!Assert(val_1 /= val_2, message)

\* Checks that `pred' is true, yields `check` if it is, fails with `msg' otherwise.
check_and(
    \* Must be true for the check not to fail.
    pred,
    \* Message produced on fail.
    msg,
    \* Yielded if the check succeeds.
    and_then
) ==
    IF TLC!Assert(pred, msg) THEN and_then ELSE TLC!Assert(FALSE, "unreachable")

\* Checks all input checks are true, yields `check` if they are, fails with the
\* failed predicate's message otherwise.
checks_and(
    \* Sequence of `(predicate, message)' pairs.
    checks,
    \* Yielded if the check succeeds.
    and_then
) ==
    IF \A i \in DOMAIN checks: (assert(checks[i][1], checks[i][2]))
    THEN and_then
    ELSE TLC!Assert(FALSE, "unreachable")

\* Lazy version of `check_and'.
\*
\* Note that both input will be passed `TRUE` as argument, which should be ignored.
lazy_check_and(
    \* Yields the predicate to check and the failure message.
    pred_and_msg(_),
    \* Yielded if the check succeeds.
    and_then(_)
) ==
    LET to_check == pred_and_msg(TRUE) IN
    IF check_and(to_check[1], to_check[2], TRUE)
    THEN and_then(TRUE)
    ELSE TLC!Assert(FALSE, "unreachable")

\* Lazy version of `checks_and'
\*
\* Note that both input will be passed `TRUE` as argument, which should be ignored.
lazy_checks_and(
    \* Sequence of `(predicate, message)' pairs.
    checks(_),
    \* Yielded if the check succeeds.
    and_then(_)
) ==
    LET to_check == checks(TRUE) IN
    IF checks_and(to_check, TRUE)
    THEN and_then(TRUE)
    ELSE TLC!Assert(FALSE, "unreachable")

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

\* Checks that two values are equal.
\*
\* Active in debug and release.
assert_eq(
    val_1,
    val_2,
    \* Message issued when `val_1' and `val_2' are not equal.
    message
) ==
    TLC!Assert(val_1 = val_2, message)

\* Checks that two values are different.
\*
\* Active in debug and release.
assert_ne(
    val_1,
    val_2,
    \* Message issued when `val_1' and `val_2' are equal.
    message
) ==
    TLC!Assert(val_1 /= val_2, message)

\* Checks that `pred' is true, yields `check` if it is, fails with `msg' otherwise.
check_and(
    \* Must be true for the check not to fail.
    pred,
    \* Message produced on fail.
    msg,
    \* Yielded if the check succeeds.
    and_then
) ==
    IF TLC!Assert(pred, msg) THEN and_then ELSE TLC!Assert(FALSE, "unreachable")

\* Checks all input checks are true, yields `check` if they are, fails with the
\* failed predicate's message otherwise.
checks_and(
    \* Sequence of `(predicate, message)' pairs.
    checks,
    \* Yielded if the check succeeds.
    and_then
) ==
    IF \A i \in DOMAIN checks: (assert(checks[i][1], checks[i][2]))
    THEN and_then
    ELSE TLC!Assert(FALSE, "unreachable")

\* Lazy version of `check_and'.
\*
\* Note that both input will be passed `TRUE` as argument, which should be ignored.
lazy_check_and(
    \* Yields the predicate to check and the failure message.
    pred_and_msg(_),
    \* Yielded if the check succeeds.
    and_then(_)
) ==
    LET to_check == pred_and_msg(TRUE) IN
    IF check_and(to_check[1], to_check[2], TRUE)
    THEN and_then(TRUE)
    ELSE TLC!Assert(FALSE, "unreachable")

\* Lazy version of `checks_and'
\*
\* Note that both input will be passed `TRUE` as argument, which should be ignored.
lazy_checks_and(
    \* Sequence of `(predicate, message)' pairs.
    checks(_),
    \* Yielded if the check succeeds.
    and_then(_)
) ==
    LET to_check == checks(TRUE) IN
    IF checks_and(to_check, TRUE)
    THEN and_then(TRUE)
    ELSE TLC!Assert(FALSE, "unreachable")

====
\* End of module `Matla'.
