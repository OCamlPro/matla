[test]
only_in = debug
expected = violation(assert)

---- MODULE type_check_fail ----

LOCAL INSTANCE TLC
LOCAL INSTANCE Integers
LOCAL INSTANCE Sequences
LOCAL Matla == INSTANCE Matla

VARIABLES reset, start_stop, sys

\* importing a file from the project
LOCAL Top == INSTANCE top WITH sys <- sys, reset <- reset, start_stop <- start_stop

not_a_sys == 3

init ==
    Top!init
step ==
    Matla!dbg!assert(
        Top!is_sys(not_a_sys),
        ToString(not_a_sys) \o " is not a legal system"
    )

====