//! Custom TOML parsing.

prelude!();

use crate::customl::TlcCla as TlcClaToml;

macro_rules! tlc_cla_error {
    ($tlc_cla:expr => $field:ident => $value:expr, $source:expr) => {
        match $tlc_cla.$field.as_mut() {
            None => {
                $tlc_cla.$field = Some(($value, $source));
                Ok(())
            }
            Some(_prev) => {
                return Err(concat!(
                    "trying to specify `",
                    stringify!($field),
                    "` twice"
                ));
            }
        }
    };
}

peg::parser! {
    pub grammar config() for str {
        rule u64() -> u64
        = quiet! {
            n:$(['0'..='9']+) {?
                u64::from_str_radix(n, 10).map_err(|_| "integer")
            }
        }
        / expected!("integer (u64)")
        rule usize() -> usize
        = quiet! {
            n:$(['0'..='9']+) {?
                usize::from_str_radix(n, 10).map_err(|_| "integer")
            }
        }
        / expected!("integer (u64)")

        rule ident() -> &'input str
        = id:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '_' | '0'..='9']*) {
            id
        }

        rule cmt() = "#" [^'\n'|'\r']* ("\n" / "\r" / [^_])
        rule ws() = [' '|'\t'|'\n'|'\r']+
        rule _() = (ws() / cmt())*

        rule bool() -> bool
        = ("true" / "on") { true }
        / ("false" / "off") { false }

        rule string_of<T>(sub: rule<T>) -> T
        = "\"" sub:sub() "\"" { sub }
        / "'" sub:sub() "'" { sub }
        rule string_opt_of<T>(sub: rule<T>) -> T
        = "\"" sub:sub() "\"" { sub }
        / "'" sub:sub() "'" { sub }
        / sub()
        rule or_default<T>(sub: rule<T>) -> Option<T>
        = ("default" / "Default" / "_") { None }
        / sub:sub() { Some(sub) }
        rule or_random<T>(sub: rule<T>) -> Option<T>
        = ("random" / "Random" / "_") { None }
        / sub:sub() { Some(sub) }
        rule or_auto<T>(sub: rule<T>) -> Option<T>
        = ("auto" / "Auto" / "_") { None }
        / sub:sub() { Some(sub) }

        // Parsers for the fields of [`crate::customl::TlcCla`].
        rule workers() -> Option<usize>
        = "workers" _ "=" _ val:string_opt_of(<or_default(<usize()>)>) { val }
        rule diff_cexs() -> bool
        = "diff_cexs" _ "=" _ val:string_opt_of(<bool()>) { val }
        rule seed() -> Option<u64>
        = "seed" _ "=" _ val:string_opt_of(<or_random(<u64()>)>) { val }
        rule terse() -> bool
        = "terse" _ "=" _ val:string_opt_of(<bool()>) { val }
        rule max_set_size() -> Option<u64>
        = "max_set_size" _ "=" _ val:string_opt_of(<or_default(<u64()>)>) { val }
        rule check_deadlocks() -> bool
        = "check_deadlocks" _ "=" _ val:string_opt_of(<bool()>) { val }
        rule print_callstack() -> bool
        = "print_callstack" _ "=" _ val:string_opt_of(<bool()>) { val }
        rule timestats() -> bool
        = "timestats" _ "=" _ val:string_opt_of(<bool()>) { val }

        // Parses a full [`crate::customl::TlcCla`].
        rule sub_tlc_cla(source: customl::Source, tlc_cla: &mut TlcClaToml)
        = (
            val:workers() {?
                tlc_cla_error!(tlc_cla => workers => val, source)
            }
            / val:diff_cexs() {?
                tlc_cla_error!(tlc_cla => diff_cexs => val, source)
            }
            / val:seed() {?
                tlc_cla_error!(tlc_cla => seed => val, source)
            }
            / val:terse() {?
                tlc_cla_error!(tlc_cla => terse => val, source)
            }
            / val:max_set_size() {?
                tlc_cla_error!(tlc_cla => max_set_size => val, source)
            }
            / val:check_deadlocks() {?
                tlc_cla_error!(tlc_cla => check_deadlocks => val, source)
            }
            / val:print_callstack() {?
                tlc_cla_error!(tlc_cla => print_callstack => val, source)
            }
            / val:timestats() {?
                tlc_cla_error!(tlc_cla => timestats => val, source)
            }
        ) ** _
        rule section_tlc_cla(source: customl::Source, tlc_cla: &mut TlcClaToml)
        = "[" _ "tlc_cla" _ "]" _ sub_tlc_cla(source, tlc_cla)

        // Parses the toolchain part of a user's config.
        rule section_toolchain(target: &mut io::PathBuf)
        = "[" _ "config" _ "]" _
        "tla2tools" _ "=" _ "'" path:$([^'\'']*) "'" {
            *target = path.into();
        }
        // Parses the config part of a project's config
        rule section_project()
        = "[" _ "project" _ "]"

        // Parses the user's toml config file.
        pub rule user(path: &mut io::PathBuf, tlc_cla: &mut TlcClaToml)
        = _ section_toolchain(path) _ section_tlc_cla((customl::Source::User), tlc_cla) _

        // Parses the project's toml config file.
        pub rule project(tlc_cla: &mut TlcClaToml)
        = _ section_project() _ sub_tlc_cla((customl::Source::Project), tlc_cla) _
    }
}
