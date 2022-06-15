---- MODULE top ----

LOCAL INSTANCE TLC

VARIABLES b1, b2

assert(pred, msg) == Assert(pred, msg)

init ==
    b1 = FALSE
    /\ b2 = TRUE
    /\ assert(~b1, "b1 is not FALSE")

next ==
    b1' = ~b1
    /\ b2' = TRUE
    /\ assert(b1, "b1 is not TRUE")

====