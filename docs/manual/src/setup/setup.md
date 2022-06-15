# Setup and portable mode

At this point, you have a (hopefully recent) matla binary in your path.

```text
{{ #include code/matla_help.sh }}
```

Obviously, everything works out of the box:

```text
{{ #include code/matla_run_before_setup.sh }}
```

except it actually does not. Matla does let us know some setup is needed and
how to perform it, so let's discuss that.

**By default**, matla requires a setup step before running which we present below. This setup will
create a directory where matla can store your user configuration which controls the underlying TLC
configuration among other things. If that is not something you are comfortable with, do read the
following sections as the [last one](#portable-mode) discusses matla's **portable mode** which does
not need any user configuration files/directories to be created.

Also, if at any point you want matla to remove all user configuration data you can simply run
`matla uninstall`. There is no functional difference with manually deleting matla's user
configuration directory, which we discuss next.


## `$HOME/.config/matla`

Following modern unix-flavored conventions, matla's configuration directory is
`$HOME/.config/matla`.

> For Windows users, `$HOME` is your user account's `Documents` folder. Well, we're a *100%* almost
> sure it's probably that, but definitely do check just in case. And let us know if we were right
> if you feel like receiving our eternal (intangible) gratitude!

Previously, we ran `matla init` and caused matla to complain that we need to run `matla setup`.
Doing so causes matla to ask a few questions as we are going to see now, but you can check the
`setup` options with `matla help setup` if you already know the kind of setup you want.

```text
{{ #include code/matla_setup.sh:part_1_of_4 }}
```

If you decide to answer *no*, then your only option is [*portable mode*](#portable-mode). Say we
agree:

```text
{{ #include code/matla_setup.sh:part_2_of_4 }}
```

Answering *no* at this point causes matla to look for the TLA+ toolbox in your path, and fail if it
cannot find one. Having matla handle the toolbox for us is arguably more convenient, so let's do
that:

```text
{{ #include code/matla_setup.sh:part_3_of_4 }}
```

Nice, the TLA+ toolbox is now in the matla user configuration directory. Matla's setup is done at
this point, right after it displays the contents of your user (*default*, here) configuration file:

```text
{{ #include code/matla_setup.sh:part_4_of_4 }}
```

If you are familiar with TLC, you probably see right away what the `[tlc_cla]` TOML-section deals
with. It specifies your *user-level* TLC options; we will see later that this is a first level of
configuration, the other two being *project-level* configuration (a TOML file in your project
directory) and *command-line-level* configuration (options passed to `matla run`). Basically, your
user-level configuration is always used except for options specified in the project-level
configuration, except for options specified at command-line level.

You can uncomment any of the items in this file, change them, and thus decide what the default
behavior of TLC (through matla) should be. Just keep in mind that [project
configuration](../project/init.md#project-level-configuration) can preempt these settings, as can
matla's command-line arguments.

We also see that your user configuration file stores the path to the TLA+ toolbox, which is the
`jar` downloaded during setup. If you had asked matla not to download it but instead retrieve it
from the environment, then assuming it found `some/path/tla2tools.jar` somewhere that's what the
value of the `tla2tools` item of the configuration file would be.

At this point everything is in place and you can move on to the next chapters of this manual. The
section below is for users that do not want matla to create a user configuration directory for some
reason.


## Portable mode

Some readers might not like this *"hidden configuration directory"* approach and prefer a
*portable* solution, where *matla* has no such impact on your home directory. Although it is not
the intended way to use matla, such readers will be glad to know they can run matla in *portable
mode* with the `--portable` (`-p` for short) flag.

In *portable mode*, matla does not look for `$HOME/.config/matla` (which would fail) and instead
scans the environment it's running in (mostly your `$PATH` environment variable) for
`tla2tools.jar`. Assuming it finds one, matla will just use that and run normally. Obviously,
running in portable mode means you will not be able to have a user configuration file.
