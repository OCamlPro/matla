# Motivation

Before matla and to the best of our knowledge, to compile and analyze (*"run"*) TLA+ specifications
(*"programs"*) consisted in either running [TLC](https://github.com/tlaplus/tlaplus) manually in a
terminal through the [TLA+ toolbox](https://lamport.azurewebsites.net/tla/toolbox.html) java jar
file, or to use the TLA+ toolbox IDE included in the TLA+ toolbox.
<!-- Both these workflows are equivalent, the latter being a GUI-click-this-button version of the former. -->

As frequent TLA+ developers, we write and maintain sizeable TLA+ codebases for formal specification
and verification purposes in industrial and semi-industrial (R&D) projects. It is our humble opinion
that the *normal* TLA+ workflow, *i.e.* calling TLC directly, does not handle various practical
aspects such as dealing with a test suite. Testing, and in particular sanity checks, is very
valuable to us since our final goal is usually to prove the safety of (the TLA+ encoding of)
whatever system we're working on. Sanity checks and regression tests raise our confidence that the
encoding is correct, the invariants and properties make sense, *etc.* and are crucial in our (and
thus the client's) confidence in (dis)proving the safety of the actual system.

> It quite obvious that TLC is not built to handle test suites and other project-level features such
> as the ones matla provides. TLC is akin to `gcc` or Rust's `rustc` compiler: it focuses on
> compiling and running, not managing a project. We are **not** criticizing TLC for lacking the
> features matla provides. Matla builds on top of TLC just like *cargo* builds on top of the `rustc`
> compiler.

The next chapters go over installing matla, its main features and how to use them. Before that, let
us go briefly over the core features we wanted in matla and why.

The first, basic feature we wanted matla to have is to deal with the TLA+ toolbox `tla2tools.jar`
(retrieve, handle, keep updated) to abstract it away from the user; much like *cargo* completely
abstracts away `rustc`. We also want the usual modern project manager comfort: initialize a project
with everything matla needs, automatically add the build directory to the `.gitignore` if one is
detected, *etc.*

The remaining main features are more involved and require more motivation, they are discussed in the
remaining sections of this chapter. Feel free to skip to the next chapter if you do not need further
motivating.
