//! Defines the matla TLA module.
//!
//! This module exposes convenience helpers, mostly for assertions. It has a `debug` and a `release`
//! version, the latter of which deactivates all debug checks for efficiency.

prelude!();

/// Matla module name.
pub const MATLA_MODULE_NAME: &str = "Matla";

/// Writes the matla TLA modules to some [`std::io::Write`].
macro_rules! matla_modules {
    {@($writer:expr, $release:expr $(,)?)
        $(#[doc = $doc:literal])*
        mod $mod_name:ident {
            $($content:tt)*
        }

        $($tail:tt)*
    } => {
        writeln!($writer,
            concat!("---- MODULE ", stringify!($mod_name), " ----")
        )?;
        $(
            writeln!($writer, concat!("\\*", $doc))?;
        )*

        matla_modules! {@($writer, $release)
            $($content)*
        }

        writeln!($writer, "\n====")?;
        writeln!($writer, concat!(
            "\\* End of module `", stringify!($mod_name), "'."
        ))?;

        matla_modules! {@($writer, $release)
            $($tail)*
        }
    };

    {@($writer:expr, $release:expr $(,)?)
        @sep

        $($tail:tt)*
    } => {
        writeln!($writer)?;

        matla_modules! {@($writer, $release)
            $($tail)*
        }
    };

    {@($writer:expr, $release:expr $(,)?)
        @rec $fun:ident ( $pat_head:pat $( , $pat_tail:pat )* $(,)? ) ;
        $($tail:tt)*
    } => {
        write!(
            $writer,
            "RECURSIVE {}({}",
            stringify!($fun),
            stringify!($pat_head),
        )?;
        $(
            write!($writer, ", {}", stringify!($pat_tail))?;
        )*
        writeln!($writer, ")")?;

        matla_modules! {@($writer, $release)
            $($tail)*
        }
    };

    {@($writer:expr, $release:expr $(,)?)
        $(#[doc = $doc:literal])*
        use($($vis:ident)?) $mod_name:ident as $import_name:ident;

        $($tail:tt)*
    } => {
        writeln!($writer)?;
        $(
            writeln!($writer, concat!("\\*", $doc))?;
        )*
        $(
            write!($writer, concat!(stringify!($vis), " "))?;
        )?
        writeln!($writer, concat!(
            stringify!($import_name),
            " == INSTANCE ",
            stringify!($mod_name),
        ))?;

        matla_modules! {@($writer, $release)
            $($tail)*
        }
    };

    {@($writer:expr, $release:expr $(,)?)
        $(#[doc = $doc:literal])*
        use($($vis:ident)?) $mod_name:ident;

        $($tail:tt)*
    } => {
        writeln!($writer)?;
        $(
            writeln!($writer, concat!("\\*", $doc))?;
        )*
        $(
            write!($writer, concat!(stringify!($vis), " "))?;
        )?
        writeln!($writer, concat!(
            "INSTANCE ",
            stringify!($mod_name),
        ))?;

        matla_modules! {@($writer, $release)
            $($tail)*
        }
    };

    {@($writer:expr, $release:expr $(,)?)
        $(#[doc = $doc:literal])*
        fn $([$vis:ident])? $fn_name:ident $(
            (
                $(
                    $(#[doc = $arg_doc:literal])*
                    $arg:ident $( ( $($arg_args:tt)* ) )?
                ),* $(,)?
            )
        )? {
            debug: $debug_def:expr,
            release: $release_def:expr $(,)?
        }

        $($tail:tt)*
    } => {
        // Write doc and signature.
        writeln!($writer)?;
        $(
            writeln!($writer, concat!("\\*", $doc))?;
        )*
        $(
            write!($writer, "{} ", stringify!($vis))?;
        )?
        write!($writer, "{}", stringify!($fn_name))?;
        $(
            write!($writer, "(")?;
            let _sep = "\n    ";
            let mut _pref = _sep;
            $(
                write!($writer, "{}", _pref)?;
                $(
                    write!($writer, concat!("\\*", $arg_doc, "{}"), _sep)?;
                )*
                let arg = stringify!($arg);
                write!($writer, "{}", arg)?;
                $(
                    write!($writer, "(")?;
                    $(
                        write!($writer, stringify!($arg_args))?;
                    )*
                    write!($writer, ")")?;
                )?
                _pref = ",\n    ";
            )*
            write!($writer, "\n)")?;
        )?
        writeln!($writer, " ==")?;

        let def_pref = "    ";
        let def = if $release { $release_def } else { $debug_def };
        for line in def.lines() {
            writeln!($writer, "{}{}", def_pref, line)?;
        }

        matla_modules! {
            @($writer, $release)
            $($tail)*
        }
    };

    {@($writer:expr, $release:expr $(,)?)
        $(#[doc = $doc:literal])*
        fn $([$vis:ident])? $fn_name:ident $( ($($args:tt)*) )?
        {
            $def:expr
        }
        $($tail:tt)*
    } => {
        matla_modules! {
            @($writer, $release)
            $(#[doc = $doc])*
            fn $([$vis])? $fn_name $( ( $($args)* ) )? {
                debug: $def,
                release: $def,
            }
            $($tail)*
        }
    };

    {@($writer:expr, $release:expr $(,)?)} => {};

    {@($writer:expr, $release:expr $(,)?) $tkn:tt $($tail:tt)*} => {
        compile_error!(concat!(
            "unexpected token `", stringify!($tkn), "`"
        ))
    };
}

pub fn test() -> Res<()> {
    let mut stdout = std::io::stdout();
    write_module(&mut stdout, true)?;
    Ok(())
}

pub fn write_module(w: &mut impl io::Write, release: bool) -> Res<()> {
    matla_modules! {@(w, release)
        /// Matla helpers, mostly for assertions and debug/release conditional compilation.
        mod Matla {

            /// TLC!Assertions are built on top of the standard `TLC' module.
            use(LOCAL) TLC as TLC;

            @sep
            @sep

            /// All functions in this module do nothing in `release' mode.
            mod dbg {
                /// Checks some predicate.
                fn assert(
                    /// Predicate that must be true.
                    pred,
                    /// Message issued when the predicate is false.
                    message,
                ) {
                    debug: "TLC!Assert(pred, message)",
                    release: "TRUE",
                }

                /// Checks that two values are equal.
                fn assert_eq(
                    val_1, val_2,
                    /// Message issued when `val_1' and `val_2' are not equal.
                    message,
                ) {
                    debug: "TLC!Assert(val_1 = val_2, message)",
                    release: "TRUE",
                }

                /// Checks that two values are different.
                fn assert_ne(
                    val_1, val_2,
                    /// Message issued when `val_1' and `val_2' are equal.
                    message,
                ) {
                    debug: "TLC!Assert(val_1 /= val_2, message)",
                    release: "TRUE",
                }

                /// Checks that `pred' is true, yields `check` if it is, fails with `msg' otherwise.
                fn check_and(
                    /// Must be true for the check not to fail.
                    pred,
                    /// Message produced on fail.
                    msg,
                    /// Yielded if the check succeeds.
                    and_then,
                ) {
                    debug: "\
                        IF TLC!Assert(pred, msg) THEN and_then ELSE TLC!Assert(FALSE, \"unreachable\")\
                    ",
                    release: "and_then",
                }

                /// Checks all input checks are true, yields `check` if they are, fails with the
                /// failed predicate's message otherwise.
                fn checks_and(
                    /// Sequence of `(predicate, message)' pairs.
                    checks,
                    /// Yielded if the check succeeds.
                    and_then,
                ) {
                    debug: "\
                        IF \\A i \\in DOMAIN checks: (assert(checks[i][1], checks[i][2]))\n\
                        THEN and_then\n\
                        ELSE TLC!Assert(FALSE, \"unreachable\")\
                    ",
                    release: "and_then",
                }

                /// Lazy version of `check_and'.
                ///
                /// Note that both input will be passed `TRUE` as argument, which should be ignored.
                fn lazy_check_and(
                    /// Yields the predicate to check and the failure message.
                    pred_and_msg(_),
                    /// Yielded if the check succeeds.
                    and_then(_),
                ) {
                    debug: "\
                        LET to_check == pred_and_msg(TRUE) IN\n\
                        IF check_and(to_check[1], to_check[2], TRUE)\n\
                        THEN and_then(TRUE)\n\
                        ELSE TLC!Assert(FALSE, \"unreachable\")\
                    ",
                    release: "and_then(TRUE)",
                }

                /// Lazy version of `checks_and'
                ///
                /// Note that both input will be passed `TRUE` as argument, which should be ignored.
                fn lazy_checks_and(
                    /// Sequence of `(predicate, message)' pairs.
                    checks(_),
                    /// Yielded if the check succeeds.
                    and_then(_),
                ) {
                    debug: "\
                        LET to_check == checks(TRUE) IN\n\
                        IF checks_and(to_check, TRUE)\n\
                        THEN and_then(TRUE)\n\
                        ELSE TLC!Assert(FALSE, \"unreachable\")\
                    ",
                    release: "and_then(TRUE)",
                }

                /// Type-checking helpers, by `@Stevendeo'.
                mod typ {
                    use (LOCAL) Integers;
                    use (LOCAL) FiniteSets;
                    use (LOCAL) Sequences;
                    use (LOCAL) Bags;

                    @sep
                    @sep

                    /// Type for booleans.
                    fn bool { "FALSE" }
                    /// Type for integers.
                    fn int { "0" }
                    /// Type for strings.
                    fn string { r#""""# }
                    /// Any type.
                    fn any{ r#""___any_type___""# }

                    @sep

                    /// Polymorphic type of functions from `dom' to `cod'.
                    fn fun(Dom, Cod) {
                        "[ x \\in {Dom} |-> Cod ]"
                    }

                    /// Type of records.
                    fn record {
                        "fun(string, any)"
                    }

                    /// Polymorphic type for sets.
                    fn set(Elm) {
                        "{Elm}"
                    }

                    /// Polymorphic type of sequences.
                    fn seq(Elm) {
                        "<<Elm>>"
                    }

                    @sep
                    @sep

                    /// `TRUE' if `char' is a digit.
                    fn [LOCAL] is_digit(char) {
                        debug: "char \\in { \
                            \"0\", \"1\", \"2\", \"3\", \"4\", \
                            \"5\", \"6\", \"7\", \"8\", \"9\" \
                        }",
                        release: "TRUE",
                    }

                    /// `FALSE' if `s' is empty, `then' otherwise.
                    fn [LOCAL] nempty_then(s, then) {
                        debug: "IF s = \"\" THEN FALSE ELSE then",
                        release: "TRUE",
                    }

                    /// Compares two types `t1' and `t2'.
                    fn [LOCAL] type_is(t1, t2) {
                        debug: "ToString(t1) = ToString(t2)",
                        release: "TRUE",
                    }

                    @sep
                    @sep

                    /// TRUE if `val' is a string.
                    fn is_string(val) {
                        debug: r#"
LET s == ToString(val) IN
nempty_then(s, Head(s) = "\"")"#,
                        release: "TRUE",
                    }

                    /// `TRUE' if `e' is a function.
                    fn is_fun(e) {
                        debug: r#"
LET head == Head(ToString(e)) IN
head = "["
\/ head = "("
\/ head = "<""#,
                        release: "TRUE",
                    }

                    /// Checks that `type' is a set type.
                    fn [LOCAL] is_set_type(val) {
                        r#"
LET s == ToString(val) IN
nempty_then(s, Head(s) = "{")"#
                    }

                    /// Checks that `Type' is a sequence.
                    fn [LOCAL] is_seq_type(val) {
                        r#"
LET s == ToString(val) IN
nempty_then(s, Head(s) = "<")"#
                    }

                    /// Assuming `setType' is a set type representant, applies `Pred' to the type of set.
                    fn [LOCAL] get_set_type(set_type, Pred(_)) {
                        r#"\A ty \in set_type: Pred(ty)"#
                    }

                    /// Assuming seqType is sequence type representant, applies `Pred' to the type of sequence.
                    fn [LOCAL] get_seq_type(seq_type, Pred(_)) {
                        r#"Pred(seq_type[1])"#
                    }

                    @sep
                    @sep

                    @rec _is(_, _, _, _);

                    fn [LOCAL] _is(orig_expr, orig_type, expr, type) {
                        r#"
\/ type_is(type, any)

\* Non function type
\/
    LET str == ToString(expr) IN 
    IF str = ""
    THEN type_is(type, string)
    ELSE 
        LET fst == Head(str) IN
        LET snd == Head(Tail(str)) IN
        IF fst = "\""
        THEN type_is(type, string)
        ELSE IF str = "FALSE" \/ str = "TRUE"
        THEN type_is(type, bool)
        ELSE IF is_digit(fst)
        THEN type_is(type, int)
        ELSE IF fst = "{"
        THEN 
            /\ is_set_type(type)
            /\ 
                IF snd = "}"
                THEN _is(orig_expr, orig_type, type, set(any))
                ELSE \A elt \in expr: 
                    get_set_type(
                        type,
                        LAMBDA ty: _is(orig_expr, orig_type, elt, ty)
                    )
        ELSE IF fst = "<"
        THEN 
            /\ is_seq_type(type)
            /\
                IF expr = <<>> 
                THEN _is(orig_expr, orig_type, type, seq(any))
                ELSE 
                    get_seq_type(
                        type,
                        LAMBDA ty: _is(orig_expr, orig_type, expr[1], ty)
                    )
        ELSE FALSE

\/ \* Record
    /\ is_fun(expr) 
    /\ is_fun(type)
    /\ DOMAIN expr = DOMAIN type
    /\ \A arg \in DOMAIN expr:
        _is(orig_expr, orig_type, expr[arg], type[arg])

\/ \* Function
    /\ is_fun(expr) 
    /\ is_fun(type)
    /\
        \E ty \in DOMAIN type:
            \A key \in DOMAIN expr:              
                /\ _is(orig_expr, orig_type, key, ty)
                /\ _is(orig_expr, orig_type, expr[key], type[ty])"#
                    }

                    @sep

                    /// `TRUE' if `expr' has type `Type'.
                    fn is(expr, Type) {
                        debug: "_is(expr, Type, expr, Type)",
                        release: "TRUE",
                    }
                }

                use() typ as typ;
            }

            /// Contains debug-only functions.
            use() dbg as dbg;

            @sep
            @sep

            /// Checks some predicate.
            ///
            /// Active in debug and release.
            fn assert(
                /// Predicate that must be true.
                pred,
                /// Message issued when the predicate is false.
                message,
            ) { "TLC!Assert(pred, message)" }

            /// Checks that two values are equal.
            ///
            /// Active in debug and release.
            fn assert_eq(
                val_1, val_2,
                /// Message issued when `val_1' and `val_2' are not equal.
                message,
            ) { "TLC!Assert(val_1 = val_2, message)" }

            /// Checks that two values are different.
            ///
            /// Active in debug and release.
            fn assert_ne(
                val_1, val_2,
                /// Message issued when `val_1' and `val_2' are equal.
                message,
            ) { "TLC!Assert(val_1 /= val_2, message)" }

            /// Checks that `pred' is true, yields `check` if it is, fails with `msg' otherwise.
            fn check_and(
                /// Must be true for the check not to fail.
                pred,
                /// Message produced on fail.
                msg,
                /// Yielded if the check succeeds.
                and_then,
            ) {
                "\
                    IF TLC!Assert(pred, msg) THEN and_then ELSE TLC!Assert(FALSE, \"unreachable\")\
                "
            }

            /// Checks all input checks are true, yields `check` if they are, fails with the
            /// failed predicate's message otherwise.
            fn checks_and(
                /// Sequence of `(predicate, message)' pairs.
                checks,
                /// Yielded if the check succeeds.
                and_then,
            ) {
                "\
                    IF \\A i \\in DOMAIN checks: (assert(checks[i][1], checks[i][2]))\n\
                    THEN and_then\n\
                    ELSE TLC!Assert(FALSE, \"unreachable\")\
                "
            }

            /// Lazy version of `check_and'.
            ///
            /// Note that both input will be passed `TRUE` as argument, which should be ignored.
            fn lazy_check_and(
                /// Yields the predicate to check and the failure message.
                pred_and_msg(_),
                /// Yielded if the check succeeds.
                and_then(_),
            ) {
                "\
                    LET to_check == pred_and_msg(TRUE) IN\n\
                    IF check_and(to_check[1], to_check[2], TRUE)\n\
                    THEN and_then(TRUE)\n\
                    ELSE TLC!Assert(FALSE, \"unreachable\")\
                "
            }

            /// Lazy version of `checks_and'
            ///
            /// Note that both input will be passed `TRUE` as argument, which should be ignored.
            fn lazy_checks_and(
                /// Sequence of `(predicate, message)' pairs.
                checks(_),
                /// Yielded if the check succeeds.
                and_then(_),
            ) {
                "\
                    LET to_check == checks(TRUE) IN\n\
                    IF checks_and(to_check, TRUE)\n\
                    THEN and_then(TRUE)\n\
                    ELSE TLC!Assert(FALSE, \"unreachable\")\
                "
            }
        }
    }

    Ok(())
}
