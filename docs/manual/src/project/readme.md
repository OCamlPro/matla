# Init and project layout

> Matla project sources for this section available [here][run/ok].

Let's dive in on an non-matla TLA+ toy project. From this point, we assume you have performed
[matla's setup](../setup/setup.md).

```text
> ls -a
.gitignore  sw_0.cfg  sw_0.tla

> bat .gitignore
───────┬────────────────────────────────────────────────────────────────────────
       │ File: .gitignore
───────┼────────────────────────────────────────────────────────────────────────
   1   │ # Ignore macos trash files
   2   │ .DS_Store
───────┴────────────────────────────────────────────────────────────────────────
```

For the sake of reproducibility, here is the content of the `.tla` file. It encodes a stopwatch
(`sw`) system counting time with `cnt`, featuring `reset` and `start_stop` buttons, and an
"internal" `counting` flag. The counter saturates at `59`.

```text
\* sw_0.tla
{{ #include ../run/code/ok/sw_0.tla }}
```

And the `.cfg` file:

```text
\* sw_0.cfg
{{ #include ../run/code/ok/sw_0.cfg }}
```

[run/ok]: https://github.com/OCamlPro/matla/tree/latest/docs/manual/src/run/code/ok
