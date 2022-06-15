---- MODULE runtime_1 ----

LOCAL INSTANCE Integers
LOCAL cnt_init == INSTANCE cnt_init

VARIABLES cnt

init ==
	cnt_init!doit(cnt)
next ==
	cnt' = (
		IF cnt < 10 THEN cnt + 1 ELSE cnt
	)

cnt_pos == cnt >= 0
====