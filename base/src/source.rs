//! Possible sources of a test.

use crate::*;

/// A position in a file.
#[readonly]
#[derive(Debug, Clone, PartialEq)]
pub struct Pos {
    /// Line (1-indexed).
    pub row: usize,
    /// Column (1-indexed).
    pub col: usize,
}
impl Pos {
    /// Constructor.
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn from_peg(line_col: peg::str::LineCol) -> Self {
        Self {
            row: line_col.line,
            col: line_col.column,
        }
    }
    pub fn is(&self, row: usize, col: usize) -> bool {
        self.row == row && self.col == col
    }
    pub fn is_start(&self) -> bool {
        self.is(1, 1)
    }

    /// Extracts the line corresponding to a position.
    pub fn pretty(&self, content: &str, text: Option<&str>) -> Res<Vec<String>> {
        let mut row = 0;
        for line in content.lines() {
            let tab_count = line.chars().filter(|c| *c == '\t').count();
            let line = line.replace('\t', "    ");
            row += 1;

            if row == self.row {
                let row_str = self.row.to_string();
                let row_str_len = row_str.len().max(4);
                let mut res = vec![];

                let pref = " ".repeat(row_str_len + 2);
                let line_1 = format!("{}|", pref);
                res.push(line_1);

                let line_2 = format!(" {0: >1$} | {2}", row_str, row_str_len, line);
                res.push(line_2);

                if let Some(text) = text {
                    let col = self.col - tab_count * 4;

                    let mut line_3 = format!("{}| ", pref);
                    for _ in 1..col {
                        line_3.push(' ');
                    }
                    line_3.push_str("^~~~~~ ");
                    line_3.push_str(text);
                    res.push(line_3)
                }
                return Ok(res);
            }
        }
        bail!("failed to reconstruct position from file content")
    }

    /// Extracts the line(s) corresponding to a span.
    pub fn pretty_span(
        &self,
        end: &Self,
        content: &str,
        start_text: Option<&str>,
        end_text: Option<&str>,
    ) -> Res<Vec<String>> {
        if self == end {
            return self.pretty(content, start_text);
        }
        let legal = self.row < end.row || (self.row == end.row && self.col < end.col);
        if !legal {
            bail!("illegal span: {} to {}", self, end);
        }
        let mut row = 0;
        let mut res = Vec::with_capacity(end.row - self.row + 2);
        let mut start_found = false;

        let row_str = end.row.to_string();
        let row_str_len = row_str.len().max(4);
        let pref = " ".repeat(row_str_len + 2);
        let monoline = self.row == end.row;

        let mut last_line = None;

        for line in content.lines() {
            // println!(" {: >5} | {}", row, line);
            let tab_count = line.chars().filter(|c| *c == '\t').count();
            let clean_line = line.replace('\t', "    ");
            row += 1;

            macro_rules! res_push {
                ($res_line:expr) => {
                    res.push($res_line);
                    last_line = Some(line.clone());
                };
            }

            if !start_found && row == self.row {
                start_found = true;

                {
                    let mut line_1 = format!("{}| ", pref);
                    let col = self.col - tab_count * 4;
                    for _ in 1..col {
                        line_1.push(' ')
                    }
                    if monoline {
                        line_1.push('v');
                        for _ in 1..end.col - self.col {
                            line_1.push('v');
                        }
                        line_1.push_str("v");
                        macro_rules! txt_pref {
                            () => {
                                line_1.push_str("~~~~~ ")
                            };
                        }
                        match (start_text, end_text) {
                            (Some(txt), _) | (None, Some(txt)) => {
                                txt_pref!();
                                line_1.push_str(txt);
                            }
                            (None, None) => (),
                        }
                    } else {
                        if let Some(text) = start_text {
                            line_1.push_str("v~~~~~ ");
                            line_1.push_str(text);
                        } else {
                            line_1.push_str("v");
                        }
                    }

                    res.push(line_1);
                }

                let line_2 = format!(" {0: >1$} | {2}", row, row_str_len, clean_line);
                res_push!(line_2);

                if self.row == end.row {
                    last_line = Some(line);
                    break;
                }
            }
            if start_found {
                if row <= end.row {
                    if row != self.row {
                        let tail_line = format!(" {0: >1$} | {2}", row, row_str_len, clean_line);
                        // println!("tail_line: {:?}", tail_line);
                        res_push!(tail_line);
                    }
                } else if row > end.row {
                    break;
                }
            }
        }

        if !start_found {
            bail!("failed to reconstruct position from file content")
        } else {
            if !monoline {
                let line = last_line.expect("failed to retrieve last line");
                let tab_count = line.chars().filter(|c| *c == '\t').count();
                let col = end.col - tab_count * 4;
                let mut l = format!("{}| ", pref);
                for _ in 1..col {
                    l.push(' ');
                }
                l.push_str("^~~~~~ ");
                l.push_str(end_text.unwrap_or("ending here"));
                res.push(l);
            } else {
                res.push(format!("{}|", pref));
            }
            res.shrink_to_fit();
            Ok(res)
        }
    }
}
implem! {
    for Pos {
        Display {
            |&self, fmt| write!(fmt, "{}:{}", self.row, self.col)
        }
        From<(usize, usize)> {
            |(row, col)| Self { row, col }
        }
    }
}

/// A position in a file.
#[derive(Debug, Clone)]
pub struct FilePos {
    pub file: String,
    pub pos: Pos,
}
impl FilePos {
    /// Constructor.
    pub fn new(file: impl Into<String>, pos: impl Into<Pos>) -> Self {
        Self {
            file: file.into(),
            pos: pos.into(),
        }
    }

    /// Turns itself into a span.
    pub fn into_span(self, end: impl Into<Pos>) -> FileSpan {
        FileSpan::new(self, end)
    }

    /// Extracts the line corresponding to a position.
    pub fn pretty(
        &self,
        load: impl FnOnce(&str, &mut String) -> Res<()>,
        text: Option<&str>,
    ) -> Res<Vec<String>> {
        let mut buf = String::with_capacity(1007);
        // println!("FilePos::loading `{}`", self.file);
        load(&self.file, &mut buf)?;
        self.pos.pretty(&buf, text)
    }
}
implem! {
    for FilePos {
        Display { |&self, fmt| write!(fmt, "{} at {}", self.file, self.pos) }
    }
}

/// A span in a file.
#[derive(Debug, Clone)]
pub struct FileSpan {
    pub start: FilePos,
    pub end: Pos,
}
implem! {
    for FileSpan {
        Deref<Target = FilePos> { |&self| &self.start }
    }
}
impl FileSpan {
    /// Constructor.
    pub fn new(start: FilePos, end: impl Into<Pos>) -> Self {
        Self {
            start,
            end: end.into(),
        }
    }

    /// Extracts the line corresponding to a span.
    pub fn pretty_span(
        &self,
        load: impl FnOnce(&str, &mut String) -> Res<()>,
        start_text: Option<&str>,
        end_text: Option<&str>,
    ) -> Res<Vec<String>> {
        // println!("FilePos::loading `{}`", self.file);
        if self.pos == self.end {
            return self.start.pretty(load, start_text);
        }
        let mut buf = String::with_capacity(1007);
        load(&self.file, &mut buf)?;
        self.start
            .pos
            .pretty_span(&self.end, &buf, start_text, end_text)
    }
}
implem! {
    for FileSpan {
        Display {
            |&self, fmt| write!(
                fmt,
                "{}, {} → {}",
                self.start.file,
                self.start.pos,
                self.end,
            )
        }
    }
}

/// A line-span in a file.
#[readonly]
#[derive(Debug, Clone)]
pub struct LineSpan {
    /// File path.
    pub path: io::PathBuf,
    /// Starting line (inclusive).
    pub start: usize,
    /// Ending line (inclusive).
    pub end: usize,
}
impl LineSpan {
    /// Constructor.
    pub fn new(path: io::PathBuf, (start, end): (usize, usize)) -> Res<Self> {
        if start > end {
            bail!("illegal `LineSpan`, start > end ({} > {})", start, end);
        }
        Ok(Self { path, start, end })
    }
}

macro_rules! line_iter_do {
    { $slf:expr, $iter:pat => $action:expr } => {
        let content = io::load_file(&$slf.path)?;
        let $iter = content.lines().enumerate().filter_map(
            |(row, line)|
                if row > $slf.end {
                    None
                } else if row >= $slf.start {
                    Some(line)
                } else {
                    None
                }
        );
        $action
    };
}
impl LineSpan {
    /// Applies an action to the lines of the line span.
    pub fn lines_do(&self, mut action: impl FnMut(&str) -> Res<()>) -> Res<()> {
        line_iter_do! {
            self, iter => for line in iter {
                action(line)?
            }
        }
        Ok(())
    }
}
