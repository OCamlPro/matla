# Test libraries

It can be quite useful to factor test-related boilerplate code for
[DRY](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself)'s sake. Moving this common code to
your project's sources is an option, but it's not a desirable one. It mixes testing and actual
sources in a way that's just dirty and unhygienic.

We discussed previously that matla treats `tla` files in `tests` that have no associated `cfg` file
as mistakes: tests you wrote the `tla` for, but forgot the `cfg` because you got distracted by a
hilarious cat meme 🙀.

</br>

Still, that's exactly what matla's *test libraries* are, with the caveat that they **must** have a
TOML test library configuration header. Just like tests, this header must be before the TLA+ module
opener. Unlike tests that specify their configuration with a `[test]` TOML section containing a few
fields, test libraries are configured with a `[lib]` TOML section with no fields, at least
**currently**.

As matla runs your tests, there is no real difference between a test `[lib]`rary and a module from
your project's actual sources. Your tests see and can refer to both of them transparently. The only
difference is the `[lib]` header, which TLC ignores, and the fact that test libraries are located
in `tests`. Hygiene all the way!

</br>

Let's illustrate this on the example from the previous section. We factor out the initialization of
the `cnt` state variable in all of our tests. You can retrieve the full project
[here][testing/testing3].

```text
\* tests/cnt_init.tla
\* does **not** have a `.cfg` file
{{ #include code/testing_3/tests/cnt_init.tla }}
```

```text
\* tests/encoding_1.tla
{{ #include code/testing_3/tests/encoding_1.tla }}
```

Our two other tests, `encoding_2` and `runtime_1`, have exactly the same content as `encoding_1`.
Matla does not even blink and handles everything gracefully as usual:

```text
{{ #include code/testing_3.test:1 }}
{{ #include code/testing_3.test:3: }}
```

[testing/testing3]: https://github.com/OCamlPro/matla/tree/latest/docs/manual/src/testing/code/testing_3
