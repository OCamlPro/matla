# Init and project configuration

At this point, our toy project is

```text
> exa -a
.gitignore  sw_0.cfg  sw_0.tla

> bat .gitignore
───────┬────────────────────────────────────────────────────────────────────────
       │ File: .gitignore
───────┼────────────────────────────────────────────────────────────────────────
   1   │ # Ignore macos trash files
   2   │ .DS_Store
───────┴────────────────────────────────────────────────────────────────────────
```

Time to `matla`-ify this project, which is done with `matla init`.

```text
> matla init
Setting up your project, hang tight.
- adding build directory to gitignore if needed...
- setting up `Matla` module...
- setting up project configuration file...
Init complete, your project is ready to roll.

> exa -a
.gitignore  Matla.tla  Matla.toml  sw_0.cfg  sw_0.tla
```

We have two new files, but before we discuss them let's adress the `.gitignore`: in its output,
matla lets us know that it added its *"build directory"* to the gitignore *if needed*, meaning *if
one exists and the build directory is not already there"*.

```text
> bat .gitignore
───────┬────────────────────────────────────────────────────────────────────────
       │ File: .gitignore
───────┼────────────────────────────────────────────────────────────────────────
   1   │ # Ignore macos trash files
   2   │ .DS_Store
   3   │
   4   │ # Ignore matla build directory.
   5   │ /target
───────┴────────────────────────────────────────────────────────────────────────
```

Lines `3` to `5` are new and add `/target` as a directory to ignore. As we will see later, this
directory will be where matla puts all its compilation/runtime artifacts.
