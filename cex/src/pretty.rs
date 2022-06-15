//! Cex pretty-printing.

prelude!();

/// Pretty-printing spec.
#[derive(Debug, Clone, Copy)]
pub struct Spec {
    /// State index style.
    pub state_idx_style: ansi::Style,
    /// State variable style.
    pub svar_style: ansi::Style,
    /// Opening/closing delimiters for tuples.
    pub tuple_delim: (&'static str, &'static str),
    /// Opening/closing delimiters for string maps.
    pub smap_delim: (&'static str, &'static str),
    /// Opening/closing delimiters for bags.
    pub bag_delim: (&'static str, &'static str),
    /// Opening/closing delimiters for sets.
    pub set_delim: (&'static str, &'static str),
    /// String to put between keys and values, **including spaces**.
    pub smap_sep: &'static str,
    /// String to put between keys and values, **including spaces**.
    pub bag_sep: &'static str,
    /// String to put before count.
    pub bag_count_pref: &'static str,
    /// String to put between the count and the element.
    pub bag_count_sep: &'static str,
    /// String maps key style.
    pub smap_key_style: ansi::Style,
    /// Bag key style.
    pub bag_key_style: ansi::Style,
    /// Non-string literal style.
    pub non_string_lit_style: ansi::Style,
    /// String literal style.
    pub string_lit_style: ansi::Style,

    /// Underline style.
    pub uline: ansi::Style,

    /// If true, align smap/bag keys.
    pub align_keys: bool,
    /// Activates pretty-printing debug-info for collections.
    pub debug_pp_colls: bool,
}
impl Default for Spec {
    fn default() -> Self {
        if conf::color().unwrap_or(false) {
            Self::fancy_1()
        } else {
            Self::empty()
        }
    }
}
impl Spec {
    /// Style-less spec.
    pub fn empty() -> Self {
        Self {
            state_idx_style: ansi::Style::new(),
            svar_style: ansi::Style::new(),
            tuple_delim: ("(", ")"),
            smap_delim: ("[", "]"),
            bag_delim: ("⟬", "⟭"),
            set_delim: ("{", "}"),
            smap_sep: " ↦ ",
            bag_sep: " ↦ ",
            bag_count_pref: "| ",
            bag_count_sep: " of ",
            smap_key_style: ansi::Style::new(),
            bag_key_style: ansi::Style::new(),
            non_string_lit_style: ansi::Style::new(),
            string_lit_style: ansi::Style::new(),

            uline: ansi::Style::new(),

            align_keys: true,
            debug_pp_colls: false,
        }
    }
    /// Fancy spec.
    pub fn fancy_1() -> Self {
        let mut slf = Self::empty();

        slf.state_idx_style = ansi::Color::Red.bold();
        slf.svar_style = ansi::Style::new().bold().underline();
        slf.smap_key_style = ansi::Style::new().underline();
        slf.non_string_lit_style = ansi::Style::new().bold();
        slf.bag_key_style = slf.non_string_lit_style.clone();
        slf.string_lit_style = ansi::Color::Yellow.bold();

        slf.uline = ansi::Style::new().underline();

        slf
    }
}

pub struct PrettyStatePref<'spec> {
    /// Max char-len of the highest index.
    last_idx_len: usize,
    last_idx: usize,
    loops_to: Option<usize>,
    spec: &'spec Spec,

    // State-specific mutable part.
    loops_to_current: bool,
    loops_above: bool,
    is_last: bool,
}
impl<'spec> PrettyStatePref<'spec> {
    /// Constructor.
    pub fn new(spec: &'spec Spec, cex: &Cex) -> Self {
        let last_idx = {
            let mut tmp = cex.states.len();
            if tmp > 0 {
                tmp -= 1
            }
            tmp
        };
        let last_idx_len = last_idx.to_string().chars().count();
        let loops_to = match cex.shape {
            crate::Shape::Finite => None,
            crate::Shape::Stuttering => Some(last_idx),
            crate::Shape::Loop(idx) => Some(*idx),
        };
        Self {
            spec,
            last_idx_len,
            last_idx,
            loops_to,

            loops_to_current: false,
            loops_above: false,
            is_last: false,
        }
    }

    #[allow(non_upper_case_globals)]
    pub const v_line: char = '│';
    #[allow(non_upper_case_globals)]
    pub const h_line: char = '─';
    #[allow(non_upper_case_globals)]
    pub const south_east: char = '┌';
    #[allow(non_upper_case_globals)]
    pub const south_west: char = '┐';
    #[allow(non_upper_case_globals)]
    pub const north_west: char = '┘';
    #[allow(non_upper_case_globals)]
    pub const north_east: char = '└';
    #[allow(non_upper_case_globals)]
    pub const east_arrow: char = '►';
    #[allow(non_upper_case_globals)]
    pub const south_arrow: char = '▼';
    #[allow(non_upper_case_globals)]
    pub const south_branch: char = '┬';
    #[allow(non_upper_case_globals)]
    pub const north_branch: char = '┴';

    pub fn pretty_state_first_line(&self, buf: &mut String, idx: idx::State) {
        let loop_side = self.loops_above;
        let arrow_above = *idx > 0;

        if loop_side {
            buf.push(Self::v_line);
        } else {
            buf.push(' ');
        }
        buf.push(' ');
        buf.push(Self::south_east);
        for i in 0..(self.last_idx_len + 2) {
            if i == 1 && arrow_above {
                buf.push(Self::south_arrow);
            } else {
                buf.push(Self::h_line);
            }
        }
        buf.push(Self::south_west);
    }
    pub fn pretty_state_second_line(
        &self,
        buf: &mut String,
        idx: idx::State,
        info: Option<&StateInfo>,
    ) {
        use fmt::Write;

        let loop_side = self.loops_above;
        let loop_self = self.loops_to_current;

        buf.push('\n');
        if loop_self {
            buf.push(Self::south_east);
            buf.push(Self::h_line);
            buf.push(Self::east_arrow);
        } else {
            if loop_side {
                buf.push(Self::v_line);
            } else {
                buf.push(' ');
            }
            buf.push(' ');
            buf.push(Self::v_line);
        }
        buf.push(' ');
        for _ in 0..(self.last_idx_len - idx.to_string().chars().count()) {
            buf.push(' ');
        }
        write!(
            buf,
            "{}",
            self.spec.state_idx_style.paint(idx.to_string()).to_string()
        )
        .unwrap();
        buf.push(' ');
        buf.push(Self::v_line);
        buf.push(' ');
        if let Some(info) = info {
            write!(
                buf,
                "{}   @ {}{}:{}",
                self.spec.state_idx_style.paint(&info.action),
                self.spec.uline.paint(&info.module),
                self.spec.uline.paint(".tla"),
                info.span.0
            )
            .unwrap()
        } else {
            write!(buf, "{}", self.spec.state_idx_style.paint("initial state")).unwrap();
        }
    }
    pub fn pretty_state_third_line(&self, buf: &mut String, _idx: idx::State) {
        let loop_side = self.loops_above || self.loops_to_current;
        let south_branch = true; // *idx < self.last_idx || self.loops_to.is_some();

        buf.push('\n');
        if loop_side {
            buf.push(Self::v_line);
        } else {
            buf.push(' ');
        }
        buf.push(' ');
        buf.push(Self::north_east);
        for i in 0..(self.last_idx_len + 2) {
            if i == 1 && south_branch {
                buf.push(Self::south_branch);
            } else {
                buf.push(Self::h_line);
            }
        }
        buf.push(Self::north_west);
    }
    pub fn pretty_state_tail_line(
        &self,
        buf: &mut String,
        _idx: idx::State,
        line: impl AsRef<str>,
    ) {
        let loop_side = self.loops_above || self.loops_to_current;
        let down_arrow = true; // *idx < self.last_idx || self.loops_to.is_some();

        buf.push('\n');
        if loop_side {
            buf.push(Self::v_line);
        } else {
            buf.push(' ');
        }
        buf.push(' ');
        buf.push(' ');
        buf.push(' ');
        if down_arrow {
            buf.push(Self::v_line);
        } else {
            buf.push(' ');
        }
        buf.push(' ');
        buf.push_str(line.as_ref());
    }
    pub fn pretty_state_last_line_opt(&self, buf: &mut String) {
        if self.loops_to.is_some() {
            buf.push('\n');
            buf.push(Self::north_east);
            buf.push(Self::h_line);
            buf.push(Self::h_line);
            buf.push(Self::h_line);
            buf.push(Self::north_west);
        } else {
            buf.push('\n');
            buf.push(' ');
            buf.push(' ');
            buf.push(' ');
            buf.push(' ');
            buf.push(Self::north_branch);
        }
    }

    pub fn pretty_state(
        &mut self,
        buf: &mut String,
        idx: idx::State,
        info: Option<&StateInfo>,
        content: &str,
    ) {
        self.loops_to_current = self.loops_to == Some(*idx);
        self.loops_above = self.loops_to.map(|s| s < *idx).unwrap_or(false);
        self.is_last = *idx == self.last_idx;
        debug_assert!(!(self.loops_to_current && self.loops_above));

        self.pretty_state_first_line(buf, idx);
        self.pretty_state_second_line(buf, idx, info);
        self.pretty_state_third_line(buf, idx);
        for line in content.lines() {
            if !line.is_empty() {
                self.pretty_state_tail_line(buf, idx, line);
            }
        }

        if *idx == self.last_idx {
            self.pretty_state_last_line_opt(buf);
        }
    }
}

impl Spec {
    /// Multi-line string representation of a cex.
    pub fn cex_to_ml_string(&self, cex: &Cex, buf: &mut String) {
        use fmt::Write;

        let mut pretty_state = PrettyStatePref::new(self, cex);

        for (idx, state) in cex.states.index_iter() {
            if *idx > 0 {
                buf.push('\n');
            }

            let content = {
                let mut buf = String::with_capacity(113);
                let max_key_len = state
                    .values
                    .keys()
                    .map(|key| key.chars().count())
                    .max()
                    .unwrap_or(0);
                for (key, value) in state.values.iter() {
                    buf.push('\n');
                    write!(buf, "{}", self.svar_style.paint(key)).unwrap();
                    if self.align_keys {
                        for _ in 0..max_key_len - key.chars().count() {
                            buf.push(' ');
                        }
                    }
                    buf.push(':');
                    buf.push(' ');
                    self.value_to_ml_string(value, &mut buf)
                }
                buf
            };

            pretty_state.pretty_state(buf, idx, state.info.as_ref(), &content);
        }
    }
    /// Multi-line string representation of a value.
    pub fn value_to_ml_string(&self, value: &value::Value, buf: &mut String) {
        match value {
            value::Value::Null => buf.push_str("null"),
            value::Value::Plain(value) => self.plain_value_to_ml_string(value, buf),
        }
    }
    /// Multi-line string representation of a plain value.
    pub fn plain_value_to_ml_string(&self, value: &value::Plain, buf: &mut String) {
        enum Frame<'a> {
            Tuple {
                ml: bool,
                indent: usize,
                tail: std::slice::Iter<'a, value::Plain>,
            },
            SMap {
                ml: bool,
                max_key_len: usize,
                indent: usize,
                tail: std::collections::btree_map::Iter<'a, String, value::Plain>,
            },
            Bag {
                ml: bool,
                max_count_len: usize,
                indent: usize,
                tail: std::collections::btree_map::Iter<'a, value::Plain, Int>,
            },
        }
        let mut current = value;
        let mut curr_indent = 0;
        macro_rules! pref {
            ($indent:expr) => {
                // write!(buf, "[{}] ", $indent).unwrap();
                for _ in 0..$indent {
                    buf.push_str("    ");
                }
            };
            () => {
                pref!(curr_indent)
            };
        }
        let mut stack: Vec<Frame> = Vec::with_capacity(7);

        'go_down: loop {
            match current {
                value::Plain::Cst(cst @ value::Cst::I(_)) => {
                    buf.push_str(&self.non_string_lit_style.paint(cst.to_string()).to_string());
                }
                value::Plain::Cst(cst @ value::Cst::B(_)) => {
                    buf.push_str(&self.non_string_lit_style.paint(cst.to_string()).to_string());
                }
                value::Plain::Cst(cst @ value::Cst::S(_)) => {
                    buf.push_str(&self.string_lit_style.paint(cst.to_string()).to_string());
                }
                value::Plain::Tuple(tuple) => {
                    buf.push_str(self.tuple_delim.0);
                    if self.debug_pp_colls {
                        buf.push_str(&format!("<{}>", current.fmt_len()));
                    }
                    let ml = !current.is_one_line();
                    let indent = curr_indent;
                    if ml {
                        curr_indent += 1;
                    }
                    let mut iter = tuple.iter();
                    if let Some(first) = iter.next() {
                        if ml {
                            buf.push('\n');
                            pref!();
                        } else {
                            buf.push(' ');
                        }
                        current = first;
                        stack.push(Frame::Tuple {
                            ml,
                            indent,
                            tail: iter,
                        });
                        continue 'go_down;
                    } else {
                        buf.push_str(self.tuple_delim.1);
                    }
                }
                value::Plain::Set(set) => {
                    buf.push_str(self.set_delim.0);
                    if self.debug_pp_colls {
                        buf.push_str(&format!("<{}>", current.fmt_len()));
                    }
                    let ml = !current.is_one_line();
                    let indent = curr_indent;
                    if ml {
                        curr_indent += 1;
                    }
                    let mut iter = set.iter();
                    if let Some(first) = iter.next() {
                        if ml {
                            buf.push('\n');
                            pref!();
                        } else {
                            buf.push(' ');
                        }
                        current = first;
                        stack.push(Frame::Tuple {
                            ml,
                            indent,
                            tail: iter,
                        });
                        continue 'go_down;
                    } else {
                        buf.push_str(self.set_delim.1);
                    }
                }
                value::Plain::SMap(smap) => {
                    let max_key_len = smap
                        .keys()
                        .map(|key| key.chars().count())
                        .max()
                        // `max_key_len` will not be used if there are no elements.
                        .unwrap_or(0);
                    buf.push_str(self.smap_delim.0);
                    if self.debug_pp_colls {
                        buf.push_str(&format!("<{}>", current.fmt_len()));
                    }
                    let ml = !current.is_one_line();
                    let indent = curr_indent;
                    if ml {
                        curr_indent += 1;
                    }
                    let mut iter = smap.iter();
                    if let Some((first_key, first_val)) = iter.next() {
                        if ml {
                            buf.push('\n');
                            pref!();
                        } else {
                            buf.push(' ');
                        }
                        buf.push_str(&self.smap_key_style.paint(first_key).to_string());
                        if ml && self.align_keys {
                            for _ in 0..max_key_len - first_key.chars().count() {
                                buf.push(' ');
                            }
                        }
                        buf.push_str(self.smap_sep);
                        current = first_val;
                        stack.push(Frame::SMap {
                            ml,
                            max_key_len,
                            indent,
                            tail: iter,
                        });
                        continue 'go_down;
                    } else {
                        buf.push_str(self.smap_delim.1);
                    }
                }
                value::Plain::Bag(bag) => {
                    let max_count_len = bag
                        .values()
                        .map(|count| count.to_string().chars().count())
                        .max()
                        // `max_key_len` will not be used if there are no elements.
                        .unwrap_or(0);
                    buf.push_str(self.bag_delim.0);
                    if self.debug_pp_colls {
                        buf.push_str(&format!("<{}>", current.fmt_len()));
                    }
                    let ml = !current.is_one_line();
                    let indent = curr_indent;
                    if ml {
                        curr_indent += 1;
                    }
                    let mut iter = bag.iter();
                    if let Some((first_elm, first_count)) = iter.next() {
                        if ml {
                            buf.push('\n');
                            pref!();
                        } else {
                            buf.push(' ');
                        }
                        let first_count_string = first_count.to_string();
                        buf.push_str(self.bag_count_pref);
                        buf.push_str(&self.bag_key_style.paint(&first_count_string).to_string());
                        if ml && self.align_keys {
                            for _ in 0..max_count_len - first_count_string.chars().count() {
                                buf.push(' ');
                            }
                        }
                        buf.push_str(self.bag_count_sep);
                        current = first_elm;
                        stack.push(Frame::Bag {
                            ml,
                            max_count_len,
                            indent,
                            tail: iter,
                        });
                        continue 'go_down;
                    } else {
                        buf.push_str(self.bag_delim.1);
                    }
                }
            }

            'go_up: loop {
                match stack.pop() {
                    None => {
                        return;
                    }
                    Some(Frame::Tuple {
                        ml,
                        indent,
                        mut tail,
                    }) => {
                        if let Some(next) = tail.next() {
                            curr_indent = indent + 1;
                            buf.push(',');
                            if ml {
                                buf.push('\n');
                                pref!();
                            } else {
                                buf.push(' ');
                            }
                            current = next;
                            stack.push(Frame::Tuple { ml, indent, tail });
                            continue 'go_down;
                        } else {
                            if ml {
                                buf.push('\n');
                                pref!(indent);
                            } else {
                                buf.push(' ');
                            }
                            buf.push_str(self.tuple_delim.1);
                            continue 'go_up;
                        }
                    }
                    Some(Frame::SMap {
                        ml,
                        max_key_len,
                        indent,
                        mut tail,
                    }) => {
                        if let Some((next_key, next_value)) = tail.next() {
                            curr_indent = indent + 1;
                            buf.push(',');
                            if ml {
                                buf.push('\n');
                                pref!();
                            } else {
                                buf.push(' ');
                            }
                            buf.push_str(&self.smap_key_style.paint(next_key).to_string());
                            if ml && self.align_keys {
                                for _ in 0..max_key_len - next_key.chars().count() {
                                    buf.push(' ');
                                }
                            }
                            buf.push_str(self.smap_sep);
                            current = next_value;
                            stack.push(Frame::SMap {
                                ml,
                                max_key_len,
                                indent,
                                tail,
                            });
                            continue 'go_down;
                        } else {
                            if ml {
                                buf.push('\n');
                                pref!(indent);
                            } else {
                                buf.push(' ');
                            }
                            buf.push_str(self.smap_delim.1);
                            continue 'go_up;
                        }
                    }
                    Some(Frame::Bag {
                        ml,
                        max_count_len,
                        indent,
                        mut tail,
                    }) => {
                        if let Some((next_elm, next_count)) = tail.next() {
                            curr_indent = indent + 1;
                            buf.push(',');
                            if ml {
                                buf.push('\n');
                                pref!();
                            } else {
                                buf.push(' ');
                            }
                            let next_count_string = next_count.to_string();
                            buf.push_str(self.bag_count_pref);
                            buf.push_str(&self.bag_key_style.paint(&next_count_string).to_string());
                            if ml && self.align_keys {
                                for _ in 0..max_count_len - next_count_string.chars().count() {
                                    buf.push(' ');
                                }
                            }
                            buf.push_str(self.bag_count_sep);
                            current = next_elm;
                            stack.push(Frame::Bag {
                                ml,
                                max_count_len,
                                indent,
                                tail,
                            });
                            continue 'go_down;
                        } else {
                            if ml {
                                buf.push('\n');
                                pref!(indent);
                            } else {
                                buf.push(' ');
                            }
                            buf.push_str(self.bag_delim.1);
                            continue 'go_up;
                        }
                    }
                }
            }
        }
    }
}

pub struct PrettyValue<'a, F> {
    pub spec: &'a Spec,
    pub value: &'a value::Plain,
    pub indent: usize,
    pub get_pref: F,
}
