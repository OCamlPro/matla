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

\* ANCHOR: assert_in_trans
LOCAL Matla == INSTANCE Matla
next ==
    bool(reset')
    /\ bool(start_stop')
    /\ Matla!assert(reset', "failing!")
    /\ (
        IF start_stop' THEN counting' = ~counting
        ELSE UNCHANGED counting
    ) /\ (
        IF reset' THEN cnt' = 0
        ELSE IF counting' /\ cnt < 59 THEN cnt' = cnt + 1
        ELSE UNCHANGED cnt
    )
\* ANCHOR_END: assert_in_trans

\* ANCHOR: invs
inv_cnt_pos == cnt >= 0
inv_reset == reset => (cnt = 0)

cnt_leq_10 == cnt <= 10
\* ANCHOR_END: invs

 ====