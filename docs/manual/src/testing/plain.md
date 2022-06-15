# Plain tests

When we discussed [project layout](../project), we briefly mentioned that matla recognizes an
optional `tests` project sub-directory but postponed discussing it further. Until now, that is.

</br>

Let's focus on success-tests for now, meaning tests that are expected to compile and run without
failing in any way: no assertion failure, dynamic type errors, invariant/property falsification,
*etc.*

As you might expect, matla treats any test in `tests` as a success-test *by default*. We will see
later how to handle tests that should fail (assertion, falsification...).

A matla test is a regular TLA+ module `tests/my_test.tla` along with a `cfg` file
`tests/my_test.cfg`. Any TLA+ module in `tests` has access to all modules and can freely refer to
any/all of them as if they were in the same directory.

> ⚠ Since tests and sources live in the same moral namespace, test modules **cannot** have the same
> name as one of the module in your project's sources.
>
> In fact, matla handles tests by creating a temporary directory in the project's `target` build
> directory and moving all your sources and the specific test you're running there before running
> TLC. Hence the potential name-clashes.

</br>

Let's create some tests in some matla project. As far as this section is concerned, it can be any
project as long as it does not have `tests`, including an empty (but initialized) project. You can
retrieve the full project [here][testing/testing1].

```text
> exa
Matla.tla  Matla.toml  tests
```

We write a few tests

```text
> exa --tree tests
tests
├── encoding_1.cfg
├── encoding_1.tla
├── encoding_2.cfg
├── encoding_2.tla
├── runtime_1.cfg
└── runtime_1.tla
```

that *morally* test some (nonexistent, here) `encoding` and `runtime` module from our project. They
can contain anything for this demo, as long as running does not fail. We decided to have all `tla`
(`cfg`) contain the same code, respectively.

```text
\* tests/encoding_1.tla
{{ #include code/testing_1/tests/encoding_1.tla }}
```

```text
\* tests/encoding_1.cfg
{{ #include code/testing_1/tests/encoding_1.cfg}}
```

To run the tests, we simply run `matla test`. Note that this will run tests in `debug` mode.
Unsurprisingly, you can run them in `release` mode with `matla test --release`.

```text
{{ #include code/testing_1.test:1 }}
{{ #include code/testing_1.test:3: }}
```

> ⚠ If you have a `tests/my_test.tla` with no associated `cfg` file, matla will assume you wrote a
> `tla` for a test but forgot to write its `cfg` and produce an error.
>
> Well, actually, you **can** have modules with no `cfg`, called *"test libraries"*, but they
> require an annotation to let matla know you actually meant for this module to be a library used
> by other tests. We will see how [shortly](libs.md).

</br>

Sometimes, especially when we write a specific test, we don't want to run all tests. You can run a
single test by passing its module name (with or without `.tla`) to `matla test`.

```text
{{ #include code/testing_1.single.test:1 }}
{{ #include code/testing_1.single.test:3: }}
```

</br>

But what about a family of tests? Say we modified the (nonexistent, here, again) `encoding` module
from the project and only want to run test dealing with this module for instance; it turns out that
`matla test` accepts more than a module name, it supports regular expressions too:

```text
{{ #include code/testing_1.regex.test:1 }}
{{ #include code/testing_1.regex.test:3: }}
```

While different from a semantic analysis checking which test references which module, you can
accomplish the same result assuming you have some discipline in your test naming convention.

**⚠ Pro tip**: matla does not look for a full match of the regular expression, just a partial one.
Hence, you can also obtain the result from above by running the following.

```text
{{ #include code/testing_1.partial_regex.test:1 }}
{{ #include code/testing_1.partial_regex.test:3: }}
```

[testing/testing1]: https://github.com/OCamlPro/matla/tree/latest/docs/manual/src/testing/code/testing_1
