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
---- MODULE typ ----
\* Type-checking helpers, by `@Stevendeo'.

LOCAL INSTANCE Integers

LOCAL INSTANCE FiniteSets

LOCAL INSTANCE Sequences

LOCAL INSTANCE Bags



\* Type for booleans.
bool ==
    FALSE

\* Type for integers.
int ==
    0

\* Type for strings.
string ==
    ""

\* Any type.
any ==
    "___any_type___"


\* Polymorphic type of functions from `dom' to `cod'.
fun(
    Dom,
    Cod
) ==
    [ x \in {Dom} |-> Cod ]

\* Type of records.
record ==
    fun(string, any)

\* Polymorphic type for sets.
set(
    Elm
) ==
    {Elm}

\* Polymorphic type of sequences.
seq(
    Elm
) ==
    <<Elm>>



\* `TRUE' if `char' is a digit.
LOCAL is_digit(
    char
) ==
    char \in { "0", "1", "2", "3", "4", "5", "6", "7", "8", "9" }

\* `FALSE' if `s' is empty, `then' otherwise.
LOCAL nempty_then(
    s,
    then
) ==
    IF s = "" THEN FALSE ELSE then

\* Compares two types `t1' and `t2'.
LOCAL type_is(
    t1,
    t2
) ==
    ToString(t1) = ToString(t2)



\* TRUE if `val' is a string.
is_string(
    val
) ==
    
    LET s == ToString(val) IN
    nempty_then(s, Head(s) = "\"")

\* `TRUE' if `e' is a function.
is_fun(
    e
) ==
    
    LET head == Head(ToString(e)) IN
    head = "["
    \/ head = "("
    \/ head = "<"

\* Checks that `type' is a set type.
LOCAL is_set_type(
    val
) ==
    
    LET s == ToString(val) IN
    nempty_then(s, Head(s) = "{")

\* Checks that `Type' is a sequence.
LOCAL is_seq_type(
    val
) ==
    
    LET s == ToString(val) IN
    nempty_then(s, Head(s) = "<")

\* Assuming `setType' is a set type representant, applies `Pred' to the type of set.
LOCAL get_set_type(
    set_type,
    Pred(_)
) ==
    \A ty \in set_type: Pred(ty)

\* Assuming seqType is sequence type representant, applies `Pred' to the type of sequence.
LOCAL get_seq_type(
    seq_type,
    Pred(_)
) ==
    Pred(seq_type[1])


RECURSIVE _is(_, _, _, _)

LOCAL _is(
    orig_expr,
    orig_type,
    expr,
    type
) ==
    
    \/ type_is(type, any)
    
    \* Non function type
    \/
        LET str == ToString(expr) IN 
        IF str = ""
        THEN type_is(type, string)
        ELSE 
            LET fst == Head(str) IN
            LET snd == Head(Tail(str)) IN
            IF fst = "\""
            THEN type_is(type, string)
            ELSE IF str = "FALSE" \/ str = "TRUE"
            THEN type_is(type, bool)
            ELSE IF is_digit(fst)
            THEN type_is(type, int)
            ELSE IF fst = "{"
            THEN 
                /\ is_set_type(type)
                /\ 
                    IF snd = "}"
                    THEN _is(orig_expr, orig_type, type, set(any))
                    ELSE \A elt \in expr: 
                        get_set_type(
                            type,
                            LAMBDA ty: _is(orig_expr, orig_type, elt, ty)
                        )
            ELSE IF fst = "<"
            THEN 
                /\ is_seq_type(type)
                /\
                    IF expr = <<>> 
                    THEN _is(orig_expr, orig_type, type, seq(any))
                    ELSE 
                        get_seq_type(
                            type,
                            LAMBDA ty: _is(orig_expr, orig_type, expr[1], ty)
                        )
            ELSE FALSE
    
    \/ \* Record
        /\ is_fun(expr) 
        /\ is_fun(type)
        /\ DOMAIN expr = DOMAIN type
        /\ \A arg \in DOMAIN expr:
            _is(orig_expr, orig_type, expr[arg], type[arg])
    
    \/ \* Function
        /\ is_fun(expr) 
        /\ is_fun(type)
        /\
            \E ty \in DOMAIN type:
                \A key \in DOMAIN expr:              
                    /\ _is(orig_expr, orig_type, key, ty)
                    /\ _is(orig_expr, orig_type, expr[key], type[ty])


\* `TRUE' if `expr' has type `Type'.
is(
    expr,
    Type
) ==
    _is(expr, Type, expr, Type)

====
\* End of module `typ'.

typ == INSTANCE typ

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
