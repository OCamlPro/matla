//! Value representation.

prelude!();

/// Either a plain value or `null`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Null,
    Plain(Plain),
}
implem! {
    impl('a) for Value {
        From<Plain> { |val| Self::Plain(val) }
        From<Cst> { |cst| Self::Plain(cst.into()) }
        From<bool> { |b| Self::Plain(b.into()) }
        From<Int> { |i| Self::Plain(i.into()) }
        From<String> { |s| Self::Plain(s.into()) }
        From<&'a str> { |s| s.to_string().into() }
        From<Tuple> { |tup| Self::Plain(tup.into()) }
        From<SMap> { |smap| Self::Plain(smap.into()) }
        From<Set> { |set| Self::Plain(set.into()) }
        From<Bag> { |bag| Self::Plain(bag.into()) }
    }
}
impl Value {
    // /// Structural depth.
    // pub fn depth(&self) -> usize {
    //     match self {
    //         Self::Null => 0,
    //         Self::Plain(plain) => plain.depth(),
    //     }
    // }
}

/// A plain value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Plain {
    Cst(Cst),
    Tuple(Tuple),
    SMap(SMap),
    Set(Set),
    Bag(Bag),
}
implem! {
    impl('a) for Plain {
        From<Cst> { |cst| Self::Cst(cst) }
        From<bool> { |b| Self::Cst(Cst::B(b)) }
        From<Int> { |i| Self::Cst(Cst::I(i)) }
        From<String> { |s| Self::Cst(Cst::S(s)) }
        From<&'a str> { |s| s.to_string().into() }
        From<Tuple> { |tup| Self::Tuple(tup) }
        From<SMap> { |smap| Self::SMap(smap) }
        From<Set> { |set| Self::Set(set) }
        From<Bag> { |bag| Self::Bag(bag) }
    }
}
impl Plain {
    /// True on constants.
    pub fn is_cst(&self) -> bool {
        match self {
            Self::Cst(_) => true,
            Self::Tuple(_) | Self::SMap(_) | Self::Set(_) | Self::Bag(_) => false,
        }
    }
    /// True on bool/int constants.
    pub fn is_tiny_cst(&self) -> bool {
        match self {
            Self::Cst(Cst::B(_) | Cst::I(_)) => true,
            Self::Cst(Cst::S(_)) | Self::Tuple(_) | Self::SMap(_) | Self::Set(_) | Self::Bag(_) => {
                false
            }
        }
    }

    /// Formatting-length of a value for formatting, helps deciding whether to one-line print.
    pub fn fmt_len(&self) -> usize {
        match self {
            Self::Cst(Cst::B(_) | Cst::I(_)) => 1,
            Self::Cst(Cst::S(s)) => s.len() / 10,
            Self::Tuple(t) => t.iter().fold(0, |sum, next| sum + next.fmt_len().max(1)),
            Self::SMap(smap) => smap
                .iter()
                .fold(0, |sum, (_key, value)| sum + 1 + value.fmt_len().max(1)),
            Self::Set(set) => set.iter().fold(0, |sum, next| sum + next.fmt_len().max(1)),
            // Bags are special because they require multi-line printing. So they are considered to
            // have max length unless they are empty.
            Self::Bag(bag) => {
                if bag.is_empty() {
                    0
                } else {
                    usize::MAX
                }
            }
        }
    }
    /// True on plain values that can be formatted on one line.
    pub fn is_one_line(&self) -> bool {
        const MAX_ELEMENT_COUNT: usize = 7;
        const MAX_BINDING_COUNT: usize = 5;
        match self {
            Self::Cst(_) => true,
            Self::Tuple(tuple) => {
                tuple.len() <= MAX_ELEMENT_COUNT
                    && tuple
                        .iter()
                        .all(|v| v.is_one_line() && (v.is_cst() || v.fmt_len() <= 2))
            }
            Self::Set(set) => {
                set.len() <= MAX_ELEMENT_COUNT
                    && set
                        .iter()
                        .all(|v| v.is_one_line() && (v.is_cst() || v.fmt_len() <= 2))
            }
            Self::SMap(smap) => {
                smap.len() <= MAX_BINDING_COUNT
                    && smap
                        .values()
                        .all(|v| v.is_one_line() && (v.is_cst() || v.fmt_len() <= 2))
            }
            Self::Bag(bag) => bag.is_empty(),
        }
    }
    // /// Structural depth.
    // pub fn depth(&self) -> usize {
    //     enum Frame<'a> {
    //         Tuple {
    //             max: usize,
    //             tail: std::slice::Iter<'a, Plain>,
    //         },
    //         SMap {
    //             max: usize,
    //             tail: std::collections::btree_map::Iter<'a, String, Plain>,
    //         },
    //     }

    //     let mut stack: Vec<(usize, Frame)> = Vec::with_capacity(17);
    //     let mut current = self;
    //     let mut offset = 0;

    //     'go_down: loop {
    //         let mut depth = match current {
    //             Self::Cst(_) => offset,
    //             Self::Tuple(tuple) => {
    //                 let mut iter = tuple.elms.iter();
    //                 if let Some(first) = iter.next() {
    //                     current = first;
    //                     stack.push((offset, Frame::Tuple { max: 0, tail: iter }));
    //                     continue 'go_down;
    //                 } else {
    //                     offset
    //                 }
    //             }
    //             Self::SMap(smap) => {
    //                 let mut iter = smap.elms.iter();
    //                 if let Some((_key, first)) = iter.next() {
    //                     current = first;
    //                     stack.push((offset, Frame::SMap { max: 0, tail: iter }));
    //                     continue 'go_down;
    //                 } else {
    //                     offset
    //                 }
    //             }
    //         };

    //         'go_up: loop {
    //             match stack.pop() {
    //                 Some((off, Frame::Tuple { max, mut tail })) => {
    //                     let max = max.max(depth);
    //                     if let Some(next) = tail.next() {
    //                         stack.push((off, Frame::Tuple { max, tail }));
    //                         current = next;
    //                         offset = off;
    //                         continue 'go_down;
    //                     } else {
    //                         depth = off + max;
    //                         continue 'go_up;
    //                     }
    //                 }
    //                 Some((off, Frame::SMap { max, mut tail })) => {
    //                     let max = max.max(depth);
    //                     if let Some((_key, next)) = tail.next() {
    //                         stack.push((off, Frame::SMap { max, tail }));
    //                         current = next;
    //                         offset = off;
    //                         continue 'go_down;
    //                     } else {
    //                         depth = off + max;
    //                         continue 'go_up;
    //                     }
    //                 }
    //                 None => {
    //                     return depth;
    //                 }
    //             }
    //         }
    //     }
    // }
}

/// A constant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cst {
    B(bool),
    I(Int),
    S(String),
}
implem! {
    impl('a) for Cst {
        From<bool> { |b| Cst::B(b) }
        From<Int> { |i| Cst::I(i) }
        From<String> { |s| Cst::S(s) }
        From<&'a str> { |s| s.to_string().into() }

        Display { |&self, fmt| match self {
            Self::B(b) => b.fmt(fmt),
            Self::I(i) => i.fmt(fmt),
            Self::S(s) => write!(fmt, "\"{}\"", s),
        } }
    }
}

/// A tuple.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tuple {
    pub elms: Vec<Plain>,
}
impl Tuple {
    /// Constructor.
    pub fn new(elms: Vec<Plain>) -> Self {
        Self { elms }
    }
}
implem! {
    for Tuple {
        Deref<Target = Vec<Plain>> {
            |&self| &self.elms,
            |&mut self| &mut self.elms,
        }
    }
}

/// A set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Set {
    pub elms: Vec<Plain>,
}
impl Set {
    /// Constructor.
    pub fn new(elms: Vec<Plain>) -> Self {
        Self { elms }
    }
}
implem! {
    for Set {
        Deref<Target = Vec<Plain>> {
            |&self| &self.elms,
            |&mut self| &mut self.elms,
        }
    }
}

/// A string-map (called *structure* in TLA+).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SMap {
    pub elms: Map<String, Plain>,
}
impl SMap {
    /// Constructor.
    pub fn new(elms: Map<String, Plain>) -> Self {
        Self { elms }
    }

    /// Empty constructor.
    pub fn new_empty() -> Self {
        Self { elms: Map::new() }
    }
}
implem! {
    for SMap {
        Deref<Target = Map<String, Plain>> {
            |&self| &self.elms,
            |&mut self| &mut self.elms,
        }
    }
}

/// A bag.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bag {
    pub elms: Map<Plain, Int>,
}
impl Bag {
    /// Constructor.
    pub fn new(elms: Map<Plain, Int>) -> Self {
        Self { elms }
    }

    /// Empty constructor.
    pub fn new_empty() -> Self {
        Self { elms: Map::new() }
    }
}
implem! {
    for Bag {
        Deref<Target = Map<Plain, Int>> {
            |&self| &self.elms,
            |&mut self| &mut self.elms,
        }
    }
}
