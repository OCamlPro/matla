//! Message-handling parser.

prelude!();

pub use self::parser::*;

peg::parser! {
    grammar parser() for str {
        rule nl() = quiet! { ['\n' | '\r' ] }
        rule _ = quiet! {
            [' ' | '\t' | '\n' | '\r' ]*
        } / expected!("whitespace")
        rule num_char() = ['0'..='9']

        rule usize() -> usize
        = quiet! {
            num:$(num_char()+) {?
                usize::from_str_radix(num, 10).map_err(|_| "illegal integer")
            }
        } / expected!("`usize` value")

        rule pretty_usize() -> usize
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

        rule int() -> Int
        = quiet! {
            num:$(num_char()+) {?
                Int::parse_bytes(num.as_bytes(), 10).ok_or_else(|| "illegal integer")
            }
        } / expected!("`usize` value")

        rule pretty_int() -> Int
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

        rule dq_string() -> &'input str
        = quiet! {
            "\"" content:$(("\\\"" / [^'"'])*) "\"" { content }
        } / expected!("double-quoted string")

        rule file_or_dir() -> &'input str
        = quiet! {
            $([^'/']+)
        } / expected!("file or directory name")
        rule parent_path() -> &'input str
        = quiet! {
            $(
                ("/")? (file_or_dir() "/")*
            )
        } / expected!("filesystem path ending with `/`")

        rule ident() -> &'input str
        = quiet! {
            $(
                ['_' | 'a'..='z' | 'A'..='Z']
                ['_' | '-' | 'a'..='z' | 'A'..='Z' | '0'..='9']*
            )
        } / expected!("identifier")

        rule file_name() -> String
        = quiet! {
            name:$(
                ident() ("." ident())?
            ) { name.into() }
        } / expected!("file name")

        rule file_pos() -> source::Pos
        = quiet! {
            "line" _ row:usize() _ "," _ ("column" / "col") _ col:usize() {
                source::Pos::new(row, col)
            }
        }
        / expected!("line/column file position")

        rule file_pos_span() -> source::FileSpan
        = "Line" _ start_row:usize() _ "," _ "column" _ start_col:usize()
        _ "to" _ "line" _ end_row:usize() _ "," _ "column" _ end_col:usize()
        _ "in" _ module:ident() {
            source::FilePos::new(module, (start_row, start_col))
                .into_span((end_row, end_col))
        }

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

        pub rule parse_error_1_expected() -> String
        = "Was" _ "expecting" _ expected:dq_string() {
            expected.into()
        }
        pub rule parse_error_1_got() -> (String, source::Pos, Option<String>)
        = "Encountered" _ got:dq_string() _ "at" _ pos:file_pos()
        _ and:("and" _ "token"? _ "\""? and:$((!['.' | '\n' | '\r' | '\"'] [_])*) "\""? _ { and })? {
            (got.into(), pos, and.map(|s| s.to_string()))
        }
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
        pub rule parse_error_trace() -> Vec<(String, source::Pos)>
        = "Residual" _ "stack" _ "trace" _ "follows" _ ":" _ trace:(
            _ elm:parse_error_trace_elm() { elm }
        )* _ {
            trace
        }
        pub rule parse_error_tail() -> String
        = "Fatal" _ "errors" _ "while" _ "parsing" _ "TLA+" _ "spec"
        _ "in" _ "file" _ module_0:ident()
        _ _exc:exc()
        _ "***" _ "Abort" _ "messages" _ ":" _ _abort_msg:usize()
        _ "In" _ "module" _ module_1:ident()
        _ "Could" _ "not" _ "parse" _ "module" _ module_2:ident()
        _ "from" _ "file" _ _file:file_name() {
            module_0.into()
        }

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



        pub rule parse_error_2_encountered() -> (String, source::Pos, Option<String>)
        = "Encountered" _ encountered:dq_string() _ "at"
        _ pos:file_pos() _ "and" _ "token" _ token:dq_string() {
            (encountered.into(), pos, Some(token.into()))
        }

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



        pub rule parse_parse_error() -> tlc::err::ParseError
        = parse_parse_error_1()
        / parse_parse_error_2()



        pub rule lexical_error_got() -> (String, source::Pos)
        =
            "at" _ pos:file_pos() _ "."
            _ "Encountered" _ ":" _ token:dq_string()
            _ "(" _ usize() _ ")" {
                (token.into(), pos)
            }
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

        pub rule parse_warning() -> tlc::warn::TlcWarning
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
                }.into()
            }

        rule parsing_file() -> ModuleOrTop
        = "Parsing" _ "file" _ (parent_path())? module:ident() "." ext:ident() {
            if ext == "cfg" {
                ModuleOrTop::TopCfg
            } else {
                module.to_string().into()
            }
        }
        rule processing_file() -> ModuleOrTop
        = "Semantic" _ "processing" _ "of" _ "module" _ module:ident() {
            ModuleOrTop::Module(module.into())
        }
        /// Returns an *okay* flag which is false on errors.
        pub rule parsing(mode: &mut tlc::runtime::Parsing)
        = _ module:parsing_file() _ {
            mode.current_file = Some(module);
        }
        / _ module: processing_file() _ {
            mode.current_file = Some(module);
        }
        / _ "***" _ "Parse" _ "Error" _ "***" _ {
            mode.error_msg = Some(String::new());
        }
        / _ err:$("Lexical" _ "error" _ [_]*) {
            mode.error_msg = Some(err.into())
        }
        / _ "Fatal" _ "errors" _ "while" _ "parsing" _ "TLA+" _ "spec"
        _ "in" _ "file" _ module:ident() {
            mode.current_file = Some(ModuleOrTop::Module(module.into()));
            mode.error_msg = Some(String::new());
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

        pub rule cex_value_cst() -> cex::value::Cst = quiet! {
            i:pretty_int() { i.into() }
            / b:(
                "TRUE" { true }
                / "FALSE" { false }
            ) { b.into() }
            / s:dq_string() { s.into() }
        } / expected!("boolean, integer, double-quoted string")

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

        pub rule cex_plain_value() -> cex::value::Plain
        = cst:cex_value_cst() { cst.into() }
        / tuple:cex_value_tuple() { tuple.into() }
        / smap:cex_value_smap() { smap.into() }
        / bag:cex_value_bag() { bag.into() }
        / set:cex_value_set() { set.into() }

        pub rule cex_value() -> cex::Value
        = val:cex_plain_value() { val.into() }
        / "null" { cex::Value::Null }

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

        pub rule cex_ident_value() -> (&'input str, cex::Value)
        = ("/\\" _)? id:ident() _ "=" _ val:cex_value() {
            (id, val)
        }

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

        pub rule invariant_violated_behavior() -> &'input str
        = "Invariant" _ id:ident() _ "is" _ "violated" _ "." {
            id
        }

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

        pub rule assertion_failure_1() = quiet! {
            _ "The" _ "first" _ "argument" _ "of" _
            _dontcare:ident() _ "evaluated" _ "to" _ "FALSE" _ ";"
            _ "the" _ "second" _ "argument" _ "was" _ ":" _
        }
        / expected!("assertion failure description")
        pub rule assertion_failure_2() -> Option<String> = quiet! {
            _ msg:dq_string() _ {
                Some(msg.into())
            }
        }
        / expected!("assertion failure message")

        pub rule error_nested_expressions_1()
        = _ "The" _ "error" _ "occurred" _ "when" _ "TLC" _ "was"
        _ "evaluating" _ "the" _ "nested" _
        pub rule error_nested_expressions_2()
        = _ "expressions" _ "at" _ "the" _ "following" _ "positions" _ ":" _
        pub rule error_nested_expressions_location() -> source::FileSpan
        = _ _idx:usize() _ "." _ span:file_pos_span() _ {
            span
        }
    }
}
