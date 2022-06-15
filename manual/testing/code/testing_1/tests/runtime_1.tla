---- MODULE runtime_1 ----

LOCAL INSTANCE Integers

VARIABLES cnt

init ==
    cnt = 0
    next ==
    cnt' = (
        IF cnt < 10 THEN cnt + 1 ELSE cnt
    )

cnt_pos == cnt >= 0
====