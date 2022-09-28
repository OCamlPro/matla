//! Message-handling parsing helpers.

prelude!();

pub use self::parser::*;

peg::parser! {
    grammar parser() for str {
        /// Parses a newline.
        rule nl() = quiet! { ['\n' | '\r' ] }
        /// Parses a whitespace.
        ///
        /// Calling the rule `_` allows to write `_` instead of `<rule_name>()`.
        rule _ = quiet! {
            [' ' | '\t' | '\n' | '\r' ]*
        } / expected!("whitespace")

        /// A digit as a character.
        rule num_char() = ['0'..='9']

        /// Parses a `usize`.
        pub rule usize() -> usize
        = quiet! {
            num:$(num_char()+) {?
                usize::from_str_radix(num, 10).map_err(|_| "illegal integer")
            }
        } / expected!("`usize` value")

        /// Parses a `usize` with `,` delimiters (`10³` separators).
        pub rule pretty_usize() -> usize
        = quiet!{
            num:$(num_char()+) tail:( "," sub:$(num_char()+) { sub } )* {?
                if tail.is_empty() {
                    usize::from_str_radix(num, 10).map_err(|_| "illegal integer")
                } else {
                    let mut s = String::with_capacity(
                        tail.iter().cloned().fold(num.len(), |acc, sub| acc + sub.len())
                    );
                    s.push_str(num);
                    for sub in tail {
                        s.push_str(sub);
                    }
                    usize::from_str_radix(&s, 10).map_err(|_| "illegal integer")
                }
            }
        } / expected!("`usize` value")


        /// Parses an integer.
        ///
        /// TODO: test and fix.
        pub rule int() -> Int
        = quiet! {
            num:$(num_char()+) {?
                Int::parse_bytes(num.as_bytes(), 10).ok_or_else(|| "illegal integer")
            }
        } / expected!("`usize` value")

        /// Parses an integer with `,` delimiters (`10³` separators).
        ///
        /// TODO: test and fix.
        pub rule pretty_int() -> Int
        = quiet!{
            num:$(num_char()+) tail:( "," sub:$(num_char()+) { sub } )* {?
                if tail.is_empty() {
                    Int::parse_bytes(num.as_bytes(), 10).ok_or_else(|| "illegal integer")
                } else {
                    let mut s = String::with_capacity(
                        tail.iter().cloned().fold(num.len(), |acc, sub| acc + sub.len())
                    );
                    s.push_str(num);
                    for sub in tail {
                        s.push_str(sub);
                    }
                    Int::parse_bytes(s.as_bytes(), 10).ok_or_else(|| "illegal integer")
                }
            }
        } / expected!("`usize` value")

        /// Parses a double-quoted string.
        ///
        /// TODO: de-escape characters, probably.
        pub rule dq_string() -> &'input str
        = quiet! {
            "\"" content:$(("\\\"" / "\\\\" / [^'"'])*) "\"" { content }
        } / expected!("double-quoted string")

        /// A legal unix file/dir name.
        ///
        /// Spaces are expected to be escaped.
        ///
        /// # Examples
        ///
        /// ```rust
        /// # use project::tlc::parse::unix_file_name;
        /// let input = r#"some\ file.n_a-me.ext"#;
        /// assert_eq!(unix_file_name(input), Ok(input));
        /// ```
        pub rule unix_file_name() -> &'input str
        = quiet! {
            $(
                (
                    ['_' | '-' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '.']
                    / ("\\" " ")
                )+
            )
        } / expected!("legal unix file/directory name")

        /// A path with a trailing `/`, can be empty.
        pub rule parent_path() -> &'input str
        = quiet! {
            $(
                ("/")? (unix_file_name() "/")*
            )
        } / expected!("filesystem path ending with `/`")

        /// A TLA+ identifier.
        pub rule ident() -> &'input str
        = quiet! {
            $(
                ['_' | 'a'..='z' | 'A'..='Z']
                ['_' | '-' | 'a'..='z' | 'A'..='Z' | '0'..='9']*
            )
        } / expected!("identifier")

        /// A position `line <nat>, col <nat>`.
        ///
        /// Accepts `Line` instead of `line`, and `column` instead of `col`.
        pub rule file_pos() -> source::Pos
        = quiet! {
            ['l'|'L'] "ine" _ row:usize() _ "," _ ("column" / "col") _ col:usize() {
                source::Pos::new(row, col)
            }
        }
        / expected!("line/column file position")

        /// A span in a file for some module.
        pub rule file_pos_span() -> source::FileSpan
        = start:file_pos() _ "to" _ end:file_pos() _ "in" _ module:ident() {
            source::FilePos::new(module, start)
                .into_span(end)
        }

        /// A TLC date.
        pub rule date() -> chrono::NaiveDateTime
        = quiet! {
            date:$(
                // year
                num_char() num_char() num_char() num_char()
                "-"
                // month
                num_char() num_char()
                "-"
                // day
                num_char() num_char()
                " "
                // hour
                num_char() num_char()
                ":"
                // minutes
                num_char() num_char()
                ":"
                // seconds
                num_char() num_char()
            ) {?
                chrono::NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S")
                    .map_err(|_e| "date/time")
            }
        } / expected!("date/time")

        /// An exception from TLC.
        pub rule exc() -> tlc::err::Exc
        = quiet! {
            "tla2sany.semantic.AbortException" { tlc::err::Exc::Abort }
            / "java.lang.NullPointerException" { tlc::err::Exc::NullPointer }
        } / expected!("TLC-level exception")

        // # Semantic Error Stuff

        // rule semantic_error_line() -> Option<tlc::err::TlcErr>
        // = exc:exc() (&nl() / ![_]) {
        //     if exc.is_abort() {
        //         None
        //     } else {
        //         Some(tlc::err::TlcErr::new_msg("exception raised").with_exc(exc))
        //     }
        // }
        // / exc:exc() _ ":" _ blah:$([_]*) {
        //     let mut blah = blah.to_string();
        //     if !blah.is_empty() {
        //         let first_char = blah.chars().next().unwrap().len_utf8();
        //         (&mut blah[0..first_char]).make_ascii_lowercase()
        //     }
        //     Some(tlc::err::TlcErr::new_msg(blah).with_exc(exc))
        // }
        // / "***" _ "Abort" _ "messages" _ ":" _ _count:usize() {
        //     None
        // }
        // / "Unknown" _ "location" { None }
        // / "File" _ "name" _ "'" module:ident() "'" _ "does" _ "not" _ "match"
        // _ "the" _ "name" _ "'" name:ident() "'" _ "of" _ "the" _ "top" _ "level"
        // _ "module" _ "it" _ "contains" _ "." {
        //     Some(tlc::err::TlcErr::new_msg(format!(
        //         "expected module header to be `MODULE {}`, got `MODULE {}`",
        //         module, name,
        //     )))
        // }

        // pub rule semantic_error() -> tlc::err::SemanticError
        // = "Fatal" _ "errors" _ "while" _ "parsing" _ "TLA+" _ "spec"
        // _ "in" _ "file" _ module:ident()
        // _ msgs:(
        //     line:semantic_error_line() _ { line }
        // )* {
        //     tlc::err::SemanticError {
        //         module: Some(module.into()),
        //         msgs: msgs.into_iter().filter_map(|msg| msg).collect(),
        //     }
        // }

        // # Parse Error Stuff

        /// Parses a `Was expecting "..."` line.
        pub rule parse_error_1_expected() -> String
        = "Was" _ "expecting" _ expected:dq_string() {
            expected.into()
        }

        /// Parses a `Encountered "..." at ...` line.
        pub rule parse_error_1_got() -> (String, source::Pos, Option<String>)
        = "Encountered" _ got:dq_string() _ "at" _ pos:file_pos()
        _ and:(
            "and" _ "token"? _ "\""?
            and:$(
                (!['.' | '\n' | '\r' | '\"'] [_])*
            ) "\""? _ {
                and
            }
        )? {
            (got.into(), pos, and.map(|s| s.to_string()))
        }

        /// Parses a parse error element, *i.e.* a parsing step.
        ///
        /// Several of these steps can be reported for a given error, exposing what parsing state
        /// TLC was in when the error occurred.
        pub rule parse_error_trace_elm() -> (String, source::Pos)
        = "Module" _ "definition" _ "starting" _ "at" _ pos:file_pos() _ "." {
            ("module definition start".into(), pos)
        }
        / "Module" _ "body" _ "starting" _ "at" _ pos:file_pos() _ "." {
            ("module body start".into(), pos)
        }
        / "Begin" _ "module" _ "starting" _ "at" _ pos: file_pos() _ "." {
            ("module header start".into(), pos)
        }
        / "Definition" _ "starting" _ "at" _ pos: file_pos() _ "." {
            ("definition start".into(), pos)
        }
        / blah:$(
            (
                !"starting"
                (['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '!' | '_'])+
                _
            )*
        ) _ "starting"
        _ "at" _ pos:file_pos() _ "." {
            (blah.trim().into(), pos)
        }

        /// Parses a trace of [`parse_error_trace_elm`].
        ///
        /// Such a trace describes the parsing state where a parse error occurred.
        pub rule parse_error_trace() -> Vec<(String, source::Pos)>
        = "Residual" _ "stack" _ "trace" _ "follows" _ ":" _ trace:(
            _ elm:parse_error_trace_elm() { elm }
        )* _ {
            trace
        }

        /// Tail of a parse error, reports the actual error.
        pub rule parse_error_tail() -> String
        = "Fatal" _ "errors" _ "while" _ "parsing" _ "TLA+" _ "spec"
        _ "in" _ "file" _ module_0:ident()
        _ _exc:exc()
        _ "***" _ "Abort" _ "messages" _ ":" _ _abort_msg:usize()
        _ "In" _ "module" _ module_1:ident()
        _ "Could" _ "not" _ "parse" _ "module" _ module_2:ident()
        _ "from" _ "file" _ _file:unix_file_name() {
            module_0.into()
        }

        /// *Expected* / *got* full parse error.
        pub rule parse_parse_error_1() -> tlc::err::ParseError
        =
            expected:parse_error_1_expected()
            _ encountered:parse_error_1_got()
            _ trace:parse_error_trace()
            _ module:parse_error_tail() {
                tlc::err::ParseError {
                    module: module.into(), expected, encountered, trace
                }
            }

        /// *Encountered* / *at* / *and* error description.
        pub rule parse_error_2_encountered() -> (String, source::Pos, Option<String>)
        = "Encountered" _ encountered:dq_string() _ "at"
        _ pos:file_pos() _ "and" _ "token" _ token:dq_string() {
            (encountered.into(), pos, Some(token.into()))
        }

        /// *Encountered* / *at* / *and* full parse error.
        pub rule parse_parse_error_2() -> tlc::err::ParseError
        =
            encountered:parse_error_2_encountered()
            _ trace:parse_error_trace()
            _ module:parse_error_tail() {
                tlc::err::ParseError {
                    module: module.into(),
                    expected: "[??]".into(),
                    encountered,
                    trace,
                }
            }

        /// Parses any full parse error.
        pub rule parse_parse_error() -> tlc::err::ParseError
        =
            parse_parse_error_1()
            / parse_parse_error_2()



        // # Lexical Error

        /// Parses a line describing what lexical analysis found that caused an error.
        pub rule lexical_error_got() -> (String, source::Pos)
        =
            "at" _ pos:file_pos() _ "."
            _ "Encountered" _ ":" _ token:dq_string()
            _ "(" _ usize() _ ")" {
                (token.into(), pos)
            }

        /// Parses full lexical errors.
        pub rule parse_lexical_error() -> tlc::err::LexicalError
        =
        // = quiet! {
            "Lexical" _ "error" _
            encountered:lexical_error_got()
            _ "," _ "after" _ ":"
            _ code:dq_string()
            _ module:parse_error_tail() {
                let code = unescape_string(code);
                tlc::err::LexicalError {
                    module: module.into(), encountered, code
                }
            }
        // } / expected!("lexical error")

        /// Parses a full semantic error.
        pub rule semantic_error(module: &ModuleOrTop) -> tlc::err::TlcError
        = terrible_error:$(
            _ "java.lang.NullPointerException" _ ":"
            _ "Cannot" _ "invoke" _ "\"String.length()\"" _ "because" _ "\"str\"" _ "is" _ "null" _
            /
            _ "[" _ "java.lang.NullPointerException" _ "]" _ ":" _ "exception" _ "raised" _
            /
            _ "java.lang.NullPointerException" _
        ) {
            // Error seems to be quite erratic, discarding the actual error.
            let _ = terrible_error;
            tlc::err::SemanticError::new(
                module.clone(),
                None,
                "TLC-level exception `java.lang.NullPointerException`\n\
                This usually means your module opener/closer is ill-formed or inexistent.\n\
                Make sure your module starts with `---- MODULE file_name_without_tla_extension ----`\n\
                and ends with `====`",
                None,
            ).into()
        }
        /
        _ "tla2sany.semantic.AbortException"
        // Explicit on purpose, want to see if it can be something else and what it means.
        //                                      v
        _ "***" _ "Abort" _ "messages" _ ":" _ "1"
        // Same here.
        // vvvvvvv
        _ "Unknown" _ "location"
        _ "File" _ "name" _ "'" module:ident() "'"
        _ "does" _ "not" _ "match" _ "the" _ "name" _ "'" module_name:ident() "'"
        _ "of" _ "the" _ "top" _ "level" _ "module" _ "it" _ "contains" _ "." _ {
            tlc::err::SemanticError::new(
                ModuleOrTop::Module(module.into()),
                None,
                format!(
                    "\
                        In `{0}.tla`: unexpected top-most header module name `{1}`;\n\
                        top-most module header must have the file's basename (`{0}`, here) \
                        as the module's name\
                    ",
                    module,
                    module_name,
                ),
                None,
            ).into()
        }
        /
        _ "Semantic" _ "error" ("s")? _ ":"
        _ "***" _ "Error" ("s")? _ ":" _ count:usize()
        errors:(
            _ start:file_pos() _ "to" _ end:file_pos()
            _ "of" _ "module" _ module:ident() _ msg:$([^'\n']*) ("\n" / ![_]) {
                (module, start, end, msg)
            }
        )+ {
            let errors = errors.into_iter().map(|(module, start, end, msg)| {
                let module_or_top = ModuleOrTop::Module(module.into());
                tlc::err::SemanticError::new(
                    module_or_top,
                    None,
                    msg,
                    Some(source::FileSpan::new(source::FilePos::new(module, start), end)),
                ).into()
            });
            tlc::err::TlcError::new_list_during("semantic processing", errors.collect())
        }
        / "Item" _ "at" _ item_start:file_pos() _ "to" _ item_end:file_pos()
        _ "of" _ "module" _ item_module:ident()
        _ "is" _ "not" _ "properly" _ "indented" _ "inside" _ "conjunction"
        _ "or" _ "disjunction" _ "list" _ "item" _ "at" _
        _ list_start:file_pos() _ "to" _ list_end:file_pos()
        _ "of" _ "module" _ list_module:ident() _ "."?
        _ trace:parse_error_trace()
        _ tail:parse_error_tail() {
            let item = source::FileSpan::new(source::FilePos::new(item_module, item_start), item_end);
            let list = source::FileSpan::new(source::FilePos::new(list_module, list_start), list_end);
            let main = tlc::err::SemanticError {
                module: ModuleOrTop::new(item_module),
                err: None,
                blah: "this item is not propertly indented".into(),
                pos: Some(item),
            };
            let sub = tlc::err::SemanticError {
                module: ModuleOrTop::new(list_module),
                err: None,
                blah: "the item is subject to semantic-indentation because it is part of this item list".into(),
                pos: Some(list),
            };
            tlc::err::TlcError::new_list(vec![main.into(), sub.into()])
        }

        /// Parses a plain parse error, a semantic error, a lexical error, or an exception.
        pub rule parse_error(module: &ModuleOrTop) -> tlc::err::TlcError
        = err:(
            e:parse_parse_error() { e.force_module(module.clone()).into() }
            / e:semantic_error(module) { e.force_module(module.clone()).into() }
            / e:parse_lexical_error() { e.force_module(module.clone()).into() }
            / e:exc() {
                tlc::err::TlcErr::new_msg("TLC crashed with an exception, without providing any context")
                    .with_exc(e)
                    .into()
                }
        ) warns:(
            "***" _ "Warning" "s"? _ ":" _ usize() _
            warnings:( w:parse_warning() _ { w } )+ {
                warnings
            }
        )? _ (
            // AFAIK this is just a repetition of the errors above. Amazing.
            "Semantic" _ "processing" _ "of" _ "module" ([_])*
        )? {
            if let Some(warns) = warns {
                let mut errs = Vec::with_capacity(warns.len() + 1);
                errs.push(err);
                errs.extend(warns.into_iter().map(tlc::err::TlcError::from));
                tlc::err::TlcError::new_list(errs)
            } else {
                err
            }
        }

        /// Parses a redefinition warning.
        pub rule warning_redef() -> tlc::warn::Redef
        =
            sym_start:file_pos() _ "to" _ sym_end:file_pos() _ "of" _ "module" _ module:ident() _ "."?
            _ "Multiple" _ "declarations" _ "or" _ "definitions" _ "for" _ "symbol" _ sym:ident() _ "."?
            _ "This" _ "duplicates" _ "the" _ "one" _ "at"
            _ prev_start:file_pos() _ "to" _ prev_end:file_pos()
            _ "of" _ "module" _ prev_module:ident() _ "."? {
                tlc::warn::Redef {
                    pos: source::FileSpan::new(source::FilePos::new(module, sym_start), sym_end),
                    sym: sym.into(),
                    prev: source::FileSpan::new(source::FilePos::new(prev_module, prev_start), prev_end),
                }
            }

        /// Parses a warning.
        pub rule parse_warning() -> tlc::warn::TlcWarning
        =
            redef:warning_redef() { redef.into() }




        // # Status parsing


        /// Status: TLC is parsing a file.
        pub rule parsing_file() -> ModuleOrTop
        = "Parsing" _ "file" _ (parent_path())? module:ident() "." ext:ident() {
            if ext == "cfg" {
                ModuleOrTop::TopCfg
            } else {
                module.to_string().into()
            }
        }

        /// Status: TLC is processing a file.
        pub rule processing_file() -> ModuleOrTop
        = "Semantic" _ "processing" _ "of" _ "module" _ module:ident() {
            ModuleOrTop::Module(module.into())
        }

        /// Updates input parsing `mode` with current file, error...
        pub rule parsing(mode: &mut tlc::runtime::Parsing)
        = _ module:parsing_file() _ {
            mode.set_current_file(module);
        }
        / _ module: processing_file() _ {
            mode.set_current_file(module);
        }
        / _ "***" _ "Parse" _ "Error" _ "***" _ {
            mode.set_error_msg(String::new());
        }
        / _ err:$("Lexical" _ "error" _ [_]*) {
            mode.set_error_msg(err)
        }
        / _ "Fatal" _ "errors" _ "while" _ "parsing" _ "TLA+" _ "spec"
        _ "in" _ "file" _ module:ident() {
            mode.set_current_file(ModuleOrTop::Module(module.into()));
            mode.set_error_msg(String::new());
        }



        /// Returns the number of initial states computed and the end date.
        pub rule init_generated_1() -> tlc::code::Status
        = _ "Finished" _ "computing" _ "initial" _ "states" _ ":"
        _ state_count:pretty_usize()
        _ "distinct" _ "state" ("s")? _ "generated" _ "at" _ end_time:date() _ "." _ {
            tlc::code::Status::TlcInitGenerated1 {
                state_count, end_time
            }
        }

        /// Returns the number states generated and the number of distinct states / states left.
        pub rule stats() -> tlc::code::Tlc
        = generated:pretty_usize() _ "state" ("s")? _ "generated" _ ","
        _ distinct:pretty_usize() _ "distinct" _ "state" ("s")? _ "found" _ ","
        _ left:pretty_usize() _ "state" ("s")? _ "left" _ "on" _ "queue" _ "." {
            tlc::code::Tlc::TlcStats {
                generated,
                distinct,
                left,
            }
        }

        /// Returns the depth of the state graph.
        pub rule search_depth() -> tlc::code::Tlc
        = "The" _ "depth" _ "of" _ "the" _ "complete" _ "state" _ "graph" _ "search" _ "is"
        _ depth:pretty_usize() "." {
            tlc::code::Tlc::TlcSearchDepth { depth }
        }

        /// Returns the graph's outdegree, min, max and percentil.
        pub rule graph_outdegree() -> tlc::code::Tlc
        = "The" _ "average" _ "outdegree" _ "of" _ "the" _ "complete" _ "state" _ "graph" _ "is"
        _ outdegree:pretty_usize()
        _ "(" _ "minimum" _ "is"
        _ min:pretty_usize()
        _ "," _ "the" _ "maximum"
        _ max:pretty_usize()
        _ "and" _ "the"
        _ percentil_th:pretty_usize()
        _ "th" _ "percentile" _ "is"
        _ percentil:pretty_usize()
        _ ")" _ "." {
            tlc::code::Tlc::TlcStateGraphOutdegree {
                outdegree, min, max, percentil_th, percentil
            }
        }

        /// Progress statistics.
        pub rule progress_stats() -> tlc::code::Tlc
        = "Progress" _ "(" _ what_is_this:pretty_usize() _ ")" _ "at" _ date:date() _ ":"
        _ generated:pretty_usize() _ "states" _ "generated"
        _ gen_spm:(
            "(" _ gen_spm:pretty_usize() _ "s" _ "/" _ "min" _ ")" _ { gen_spm }
        )? ","
        _ distinct:pretty_usize() _ "distinct" _ "states" _ "found"
        _ dist_spm:(
            "(" _ dist_spm:pretty_usize() _ "ds" _ "/" _ "min" _ ")" _ { dist_spm }
        )? ","
        _ left:pretty_usize() _ "states" _ "left" _ "on" _ "queue" _ "." {
            tlc::code::Tlc::TlcProgressStats {
                generated,
                gen_spm,
                distinct,
                dist_spm,
                left,
            }
        }

        /// A int / bool / string constant in a counterexample.
        pub rule cex_value_cst() -> cex::value::Cst = quiet! {
            i:pretty_int() { i.into() }
            / b:(
                "TRUE" { true }
                / "FALSE" { false }
            ) { b.into() }
            / s:dq_string() { s.into() }
        } / expected!("boolean, integer, double-quoted string")

        /// A tuple value in a counterexample.
        pub rule cex_value_tuple() -> cex::value::Tuple
        = "<<" _ content:(
            head:cex_plain_value()
            tail:(
                _ "," _ elm:cex_plain_value() { elm }
            )* {
                let mut content = Vec::with_capacity(tail.len() + 1);
                content.push(head);
                content.extend(tail);
                content
            }
        )? _ ">>" {
            cex::value::Tuple::new(content.unwrap_or_else(Vec::new)).into()
        }

        /// A set value in a counterexample.
        pub rule cex_value_set() -> cex::value::Set
        = "{" _ content:(
            head:cex_plain_value()
            tail:(
                _ "," _ elm:cex_plain_value() { elm }
            )* {
                let mut content = Vec::with_capacity(tail.len() + 1);
                content.push(head);
                content.extend(tail);
                content
            }
        )? _ "}" {
            cex::value::Set::new(content.unwrap_or_else(Vec::new)).into()
        }

        /// A map value in a counterexample.
        pub rule cex_value_smap() -> cex::value::SMap
        = "[" _ content:(
            head_ident:ident() _ "|->" _ head_value:cex_plain_value()
            tail:(
                _ "," _ ident:ident() _ "|->" _ value:cex_plain_value() {
                    (ident.to_string(), value)
                }
            )* {
                let mut content = cex::value::SMap::new_empty();
                let _prev = content.insert(head_ident.into(), head_value);
                debug_assert_eq!(_prev, None);
                content.extend(tail);
                content
            }
        )? _ "]" {
            content.unwrap_or_else(cex::value::SMap::new_empty)
        }

        /// A bag value in a counterexample.
        pub rule cex_value_bag() -> cex::value::Bag
        = "(" _  content:(
            head_value:cex_plain_value() _ ":>" _ head_count:int()
            tail:(
                _ "@@" _ value:cex_plain_value() _ ":>" _ count:int() {
                    (value, count)
                }
            )* {
                let mut content = cex::value::Bag::new_empty();
                let _prev = content.insert(head_value, head_count);
                debug_assert_eq!(_prev, None);
                content.extend(tail);
                content
            }
        )? _ (",")? ")" {
            content.unwrap_or_else(cex::value::Bag::new_empty)
        }

        /// A plain value, collection or constant.
        pub rule cex_plain_value() -> cex::value::Plain
        = cst:cex_value_cst() { cst.into() }
        / tuple:cex_value_tuple() { tuple.into() }
        / smap:cex_value_smap() { smap.into() }
        / bag:cex_value_bag() { bag.into() }
        / set:cex_value_set() { set.into() }

        /// A value or `null` (undefined value).
        pub rule cex_value() -> cex::Value
        = val:cex_plain_value() { val.into() }
        / "null" { cex::Value::Null }

        /// Returns the state index, and the state info if not initial.
        pub rule state_info() -> (usize, Option<cex::StateInfo>)
        = n:usize() _ ":" _ "<" _ info:(
            "Initial" _ "predicate" { None }

            / action:ident() _ span_start:file_pos() _ "to" _ span_end:file_pos()
            _ "of" _ "module" _ module:ident() {
                Some(cex::StateInfo::new(
                    action, (span_start, span_end), module
                ))
            }
        ) _ ">" {
            (n, info)
        }

        /// Temporal cex: indicates the cex loops back to some state.
        pub rule back_to_state() -> usize
        = "Back" _ "to" _ "state" _ n:usize() _ ":" _ [_]* {
            n
        }
        / n:usize() _ ":" _ "Back" _ "to" _ "state" _ [_]* {
            n
        }

        /// Start of a state-variable value (`<ident> = <value>`) in a cex.
        pub rule cex_ident_value() -> (&'input str, cex::Value)
        = ("/\\" _)? id:ident() _ "=" _ val:cex_value() {
            (id, val)
        }

        /// A state in a trace of states and its index.
        pub rule trace_state() -> Res<(usize, cex::State)>
        = index_and_info:state_info() id_val:(
            _ id_val:cex_ident_value() {
                id_val
            }
        )* {
            let (index, info) = index_and_info;
            let mut state = cex::State::new(info);
            for (id, val) in id_val {
                let _prev = state.insert(id.to_string(), val);
                if _prev.is_some() {
                    bail!("TLC produced a cex state that mentions `{}` twice", id)
                }
            }
            Ok((index, state))
        }

        /// Notification that an invariant was falsified.
        pub rule invariant_violated_behavior() -> &'input str
        = "Invariant" _ id:ident() _ "is" _ "violated" _ "." {
            id
        }

        /// Status: TLC is done.
        pub rule finished() -> tlc::code::Status
        = "Finished" _ "in" _ time_ms:pretty_int() _ "ms" _ "at" _ "(" _ date:date() _ ")" {?
            let sub_sec_ms: Int = &time_ms % 1000;
            let secs = time_ms - &sub_sec_ms;
            sub_sec_ms
                .to_u64()
                .and_then(|sub_sec_ms| secs.to_u64().map(|secs| (sub_sec_ms, secs)))
                .ok_or("failed to convert millis to duration")
                .map(|(sub_sec_ms, secs)| {
                    let runtime =
                        time::Duration::from_millis(sub_sec_ms) + time::Duration::from_secs(secs);
                    tlc::code::Status::TlcFinished { runtime }
                })
        }

        /// First line of an assertion failure (what failed).
        pub rule assertion_failure_1() = quiet! {
            _ "The" _ "first" _ "argument" _ "of" _
            _dontcare:ident() _ "evaluated" _ "to" _ "FALSE" _ ";"
            _ "the" _ "second" _ "argument" _ "was" _ ":" _
        }
        / expected!("assertion failure description")

        /// Second line of an assertion failure (failure message).
        pub rule assertion_failure_2() -> Option<String> = quiet! {
            _ msg:dq_string() _ {
                Some(msg.into())
            }
        }
        / expected!("assertion failure message")

        /// Message indicating the error is nested in some evaluation.
        pub rule error_nested_expressions_1()
        = _ "The" _ "error" _ "occurred" _ "when" _ "TLC" _ "was"
        _ "evaluating" _ "the" _ "nested" _

        /// Continuation of [`error_nested_expressions_1`].
        pub rule error_nested_expressions_2()
        = _ "expressions" _ "at" _ "the" _ "following" _ "positions" _ ":" _

        /// Location of the nested expressions.
        pub rule error_nested_expressions_location() -> source::FileSpan
        = _ _idx:usize() _ "." _ span:file_pos_span() _ {
            span
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn dq_string() {
        let input = r#""IN""#;
        let res = super::dq_string(input).unwrap();
        assert_eq!(res, "IN");
        let input = r#""\in""#;
        let res = super::dq_string(input).unwrap();
        assert_eq!(res, "\\in");
        let input = r#""\\""#;
        let res = super::dq_string(input).unwrap();
        assert_eq!(res, "\\\\");
    }
}
