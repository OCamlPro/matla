# Running

> This chapter builds on the toy project discussed in the [previous chapter](../project). It might
> be useful to be familiar with it, especially if this is your first time reading this.
>
> Also, all demos run with an unmodified `Matla.toml` project configuration file.

\
\

Running matla on your matla-`init`ialized TLA+ project is easy enough:

```text
> ls
Matla.tla  Matla.toml  sw_0.cfg  sw_0.tla

> matla run
system is safe

> ls
Matla.tla  Matla.toml  sw_0.cfg  sw_0.tla  target
```

> Matla project sources for this section available [here][run/ok].

That's a bit underwhelming, though we did get a *safe* result. This means all invariants/properties
were proved to hold by TLC (called through matla). There is a new `target` folder which is where
all build-time/run-time artifact live. Feel free to check out its content if you're interested in
peeking at how matla handles your sources and runs TLC on them. Also, note that you can clean your
project directory with `matla clean`. This is effectively the same as `rm -rf target`. Note that
`matla run` does not create or modify anything outside `target`, hence the simple cleanup
procedure.

Moving on, let's take a look at the `.cfg` file.

```text
{{ #include code/ok/sw_0.cfg }}
```

It turns out there was two invariants to check.

```text
> bat -r 29:32 sw_0.tla
───────┬────────────────────────────────────────────────────────────────────────
       │ File: sw_0.tla
───────┼────────────────────────────────────────────────────────────────────────
  29   │ inv_cnt_pos == cnt >= 0
  30   │ inv_reset == reset => (cnt = 0)
  31   │
  32   │ cnt_leq_10 == cnt <= 10
───────┴────────────────────────────────────────────────────────────────────────
```

Both are expected to hold, which TLC confirms. Next, we'll add some falsifiable
invariants/properties to see what happens.

[run/ok]: https://github.com/OCamlPro/matla/tree/latest/docs/manual/src/run/code/ok
