---- MODULE top ----

LOCAL INSTANCE Integers
LOCAL INSTANCE Sequences
LOCAL INSTANCE TLC
Matla == INSTANCE Matla

VARIABLES reset, start_stop, sys

svars == <<reset, start_stop, sys>>

\* Type-checking for booleans.
is_bool(b) == b \in {TRUE, FALSE}
\* Type-checking for naturals.
is_nat(n) == n \in Nat

\* Type of systems, which are records with fields `cnt: int`, `counting: bool`.
SysType == [
    cnt |-> Matla!dbg!typ!int,
    is_counting |-> Matla!dbg!typ!bool
]

\* System constructor.
sys_new(cnt, counting) ==
    Matla!dbg!checks_and(
        \* list of check/message pairs
        <<
            <<is_nat(cnt), "'cnt' should have type 'Nat'">>,
            <<is_bool(counting), "'counting' should have type 'Bool'">>
        >>,
        \* value to yield if all checks where successful
        [ cnt |-> cnt, is_counting |-> counting ]
    )

\* Type-checking for systems.
is_sys(s) == Matla!dbg!typ!is(s, SysType)

init_sys ==
    LET s0 == sys_new(0, FALSE) IN
    Matla!dbg!check_and(
        is_sys(s0), "error on initial system value : " \o ToString(s0),
        s0
    )

init ==
    \* inputs, must be booleans (this is not type-checking)
	is_bool(reset)
	/\ is_bool(start_stop)
    \* initial system value
	/\ sys = init_sys

next ==
    \* inputs, must be booleans (this is not type-checking)
	is_bool(reset')
	/\ is_bool(start_stop')
    \* type-check previous `sys`
    /\ Matla!dbg!assert(
        is_sys(sys),
        "expected system structure '[cnt: Nat, counting: Bool]'"
    )
    /\ (
        LET counting ==
            IF start_stop'
            THEN sys.is_counting
            ELSE ~sys.is_counting
        IN
        LET cnt ==
            IF reset'
            THEN 0
            ELSE IF counting /\ sys.cnt < 59
            THEN sys.cnt + 1
            ELSE sys.cnt
        IN
        sys' = sys_new(cnt, counting)
    )

inv_cnt_pos == sys.cnt >= 0
inv_reset == reset => (sys.cnt = 0)

pos_leq_10 == sys.cnt <= 10

====