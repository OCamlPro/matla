---- MODULE sw_0 ----

LOCAL INSTANCE Integers

VARIABLES cnt, reset, start_stop, counting

svars == <<cnt, reset, start_stop, counting>>

bool(stuff) == stuff \in { TRUE, FALSE }

init ==
    bool(reset)
    /\ bool(start_stop)
    /\ (cnt = 0)
    /\ (counting = start_stop)

next ==
    bool(reset')
    /\ bool(start_stop')
    /\ (
        IF start_stop' THEN counting' = ~counting
        ELSE UNCHANGED counting
    ) /\ (
        IF reset' THEN cnt' = 0
        ELSE IF counting' /\ cnt < 59 THEN cnt' = cnt + 1
        ELSE UNCHANGED cnt
    )

inv_cnt_pos == cnt >= 0
inv_reset == reset => (cnt = 0)

cnt_leq_10 == cnt <= 10

well_thats_gonna_fail == (cnt = 1) ~> (cnt = 10)

 ====