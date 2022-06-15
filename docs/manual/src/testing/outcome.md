# Test configuration and expected outcome

The ability to write tests that will be checked for success is not enough, in practice many tests
check that something bad is rejected. Matla does support this, in fact you can specify that you
expect pretty much any kind of outcome TLC can produce.

Matla lets you specify this by parsing an optional TOML test specification before the module header
of your test `tla` file. As you may know, TLC ignores everything before the module header (and after
the module footer), which allows us to write our test specification without making the file illegal
for TLC.

</br>

We did not do anything of the sort in the previous section because we wrote tests expected to
succeed, which is what matla assumes if we omit the test specification TOML header. Let's make the
default specification explicit: it takes the form of a `[test]` TOML section containing a few
fields. Full project available [here][testing/testing2].

```text
\* tests/encoding_1.tla
{{ #include code/testing_2/tests/encoding_1.tla }}
```

The first field is `only_in`, which specifies whether the test should only run in `debug` mode or
`release` mode. If you recall, `matla test` runs tests in `debug` mode while `matla test --release`
runs them in `release` mode. Here, `none` means that the test should run in both debug and release,
which is the same as omitting the `only_in` field completely. Besides `none`, `only_in`'s value can
be `debug` or `release`.

This can be useful to make sure that your type-checking assertions are present and correct. Such
checks are typically `Matla!dbg!assert`ions, which would fail if included in a `matla test
--release` run as debug assertions are compiled away in release mode. Conversely, some of your
tests might be expensive enough that you don't want type-checking assertions to be active to save
time, so you would have `only_in = .

</br>

Next is the last, more interesting field: `expected`. Note that its value can optionally be quoted,
*e.g.* `"success"`. Matla supports a relatively wide range of values. It's not necessary for you to
remember them all; instead, we advise you write a definitely illegal value such as `help me`. This
will cause `matla test` to fail parsing the value and produce a detailed explanation.

```text
\* tests/encoding_2.tla
{{ #include code/testing_2/tests/encoding_2.tla }}
```

The explanation actually goes over most of what we saw in this section:

```text
{{ #include code/testing_2.help_me.test:1 }}
{{ #include code/testing_2.help_me.test:3: }}
```

It seems to us that matla does a pretty good job at explaining how to write the test's
configuration and the `expected` field in particular, so we elaborate no further.

[testing/testing2]: https://github.com/OCamlPro/matla/tree/latest/docs/manual/src/testing/code/testing_2
