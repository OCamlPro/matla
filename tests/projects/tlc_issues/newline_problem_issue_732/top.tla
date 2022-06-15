---- MODULE top ----

VARIABLES b
svars == <<b>>

init ==
    b = TRUE
next ==
    b' = ~b
    /\ 7

====