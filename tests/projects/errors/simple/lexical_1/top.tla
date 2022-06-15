---- MODULE top ----

LOCAL INSTANCE Integers

VARIABLES reset, start_stop, sys

svars == <<reset, start_stop, sys>>

\* Type-checking for booleans.
is_bool(b) == b \in Bool
\* Type-checking for naturals.
is_nat(n) == n \in Nat

\* System constructor.
sys_new(cnt, counting) ==
    Matla!dbg!checks_and(
        \* list of check/message pairs
        <<
            <<is_nat(cnt), "`cnt` should have type `Nat`">>,
            <<is_bool(counting), "`counting` should have type `Bool`">>
        >>,
        \* value to yield if all checks where successful
        [ cnt |-> cnt, counting |-> counting ]
    )

\* Type-checking for systems.
is_sys(s) ==
    DOMAIN s = { "cnt", "counting" }
    /\ is_nat(s.cnt)
    /\ is_bool(s.counting)

init_sys ==
    LET s0 == sys_new(0, false) IN
    Matla!dbg!is_and(
        is_sys(s),
        "error on initial system value",
        s0
    )

init ==
    \* inputs, must be booleans (this is not type-checking)
	is_bool(reset)
	/\ is_bool(start_stop)
    \* initial system value
	/\ sys = s0

next ==
    \* inputs, must be booleans (this is not type-checking)
	is_bool(reset')
	/\ is_bool(start_stop')
    \* type-check previous `sys`
    /\ Matla!dbg!assert(
        is_sys(sys),
        "expected system structure `[cnt: Nat, counting: Bool]`"
    )
    /\ (
        LET counting ==
            IF start_stop'
            THEN sys.counting
            ELSE ~sys.counting
        IN
        LET cnt ==
            IF reset'
            THEN 0
            ELSE IF counting /\ cnt < 59
            THEN sys.cnt + 1
            ELSE sys.cnt
        IN
        sys_new(cnt, counting)
    )

inv_cnt_pos == sys.cnt >= 0
inv_reset == reset => (sys.cnt = 0)

pos_leq_10 == sys.cnt <= 10

====