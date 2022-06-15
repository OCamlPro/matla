//! Integration testing explanations.

/// Explains test configuration.
pub fn test_conf() -> &'static str {
    "\
Integration tests live in the optional `tests` directory of your project. A test is a *runnable* TLA
module, *i.e.* a TLA file and its companion `.cfg` file.
TLA integration test files must start with a test *configuration*, before the `----` module header.
The configuration is written in toml and looks as follows, **inside** the code block.
```toml
[test]
only_in = <'none'|'debug'|'release'>
expected = <result>
```
where `only_in` is optional and `none` by default; `expected` is also
optional and is `success` by default. Its value must be one of
- `success`
- `violation(assumption)`
- `violation(deadlock)`
- `violation(safety)`
- `violation(liveness)`
- `violation(assert)`
- `failure(spec)`
- `failure(safety)`
- `failure(liveness)`
- `error(spec_parse)`
- `error(config_parse)`
- `error(statespace_too_big)`
- `error(system)`\
    "
}
