//! Handles TLC errors.

prelude!();

#[derive(Debug, Clone)]
pub enum TlcError {
    NoJavaRuntime,
    Parse(ParseError),
    Semantic(SemanticError),
    Lexical(LexicalError),
    Run(RunError),
    Tlc(TlcErr),
    List {
        during: Option<String>,
        errs: Vec<TlcError>,
    },
    Warning(tlc::warn::TlcWarning),
}
implem! {
    for TlcError {
        From<ParseError> {
            |e| Self::Parse(e)
        }
        From<SemanticError> {
            |e| Self::Semantic(e)
        }
        From<LexicalError> {
            |e| Self::Lexical(e)
        }
        From<RunError> {
            |e| Self::Run(e),
        }
        From<TlcErr> {
            |e| Self::Tlc(e),
        }
        From<tlc::warn::TlcWarning> {
            |w| Self::Warning(w)
        }
    }
}
impl TlcError {
    /// Pretty, multi-line representation.
    pub fn pretty(&self, project: &crate::FullProject, styles: &conf::Styles) -> Res<Vec<String>> {
        match self {
            Self::NoJavaRuntime => Ok(vec![
                format!(
                    "The operation couldn’t be completed. Unable to locate a {}.",
                    styles.bad.paint("Java Runtime")
                ),
                format!(
                    "Please visit {} for information on installing Java.",
                    styles.uline.paint("http://www.java.com")
                ),
            ]),
            Self::Parse(e) => e.pretty(project, styles),
            Self::Semantic(e) => e.pretty(project, styles),
            Self::Lexical(e) => e.pretty(project, styles),
            Self::Run(e) => e.pretty(project, styles),
            Self::Tlc(e) => e.pretty(project, styles),
            Self::Warning(e) => e.pretty(project, styles),
            Self::List { during, errs } => {
                let mut res = vec![format!(
                    "multiple problems occurred{}",
                    if let Some(during) = during.as_ref() {
                        format!(" during {}", styles.fatal.paint(during))
                    } else {
                        "".into()
                    }
                )];
                for e in errs {
                    let warn = if e.is_warning() {
                        format!("{}: ", styles.bad.paint("warning"))
                    } else {
                        "".into()
                    };
                    let pretty = e.pretty(project, styles)?;
                    for (idx, line) in pretty.into_iter().enumerate() {
                        let pref = if idx == 0 {
                            format!("- {}", warn)
                        } else {
                            "  ".into()
                        };
                        res.push(format!("{}{}", pref, line))
                    }
                }
                Ok(res)
            }
        }
    }

    /// True if the error is a warning.
    pub fn is_warning(&self) -> bool {
        match self {
            Self::Warning(_) => true,
            Self::List { errs, .. } => errs.iter().all(Self::is_warning),
            Self::NoJavaRuntime
            | Self::Parse(_)
            | Self::Semantic(_)
            | Self::Lexical(_)
            | Self::Run(_)
            | Self::Tlc(_) => false,
        }
    }

    /// Attempts to turn the error into a [`RunError`].
    pub fn try_into_run_error(&mut self) {
        match self {
            Self::Semantic(e) => *self = e.clone().into_run_error().into(),
            _ => (),
        }
    }

    /// Generates the relevant TLC failed outcome.
    pub fn to_outcome(&self) -> Option<tlc::FailedOutcome> {
        match self {
            Self::Parse(e) => e.to_outcome(),
            Self::Semantic(e) => e.to_outcome(),
            Self::Lexical(e) => e.to_outcome(),
            Self::Run(e) => e.to_outcome(),
            Self::Tlc(e) => e.to_outcome(),
            Self::NoJavaRuntime => Some(FailedOutcome::Plain(
                "unable to locate a Java Runtime".into(),
            )),
            Self::Warning(_) => None,
            Self::List { errs, .. } => {
                for err in errs {
                    let outcome = err.to_outcome();
                    if outcome.is_some() {
                        return outcome;
                    }
                }
                None
            }
        }
    }

    /// Run error constructor.
    pub fn new_run(e: impl Into<RunError>) -> Self {
        Self::Run(e.into())
    }

    /// Parse error constructor.
    pub fn new_parse(lines: &str, module: &ModuleOrTop) -> Res<Self> {
        match tlc::parse::parse_error(lines, module) {
            Ok(res) => Ok(res),
            Err(e) => {
                let mut err = anyhow!("```");
                for line in lines.lines().rev() {
                    err = err.context(anyhow!("{}", line))
                }
                err = err.context("```");
                err = err.context("while parsing");
                err = err.context(e);
                Err(err)
            }
        }
    }

    /// A list of errors.
    pub fn new_list_during(during: impl Into<String>, errs: Vec<Self>) -> Self {
        Self::List {
            during: Some(during.into()),
            errs,
        }
    }
    /// A list of errors.
    pub fn new_list(errs: Vec<Self>) -> Self {
        Self::List { during: None, errs }
    }

    /// Semantic error constructor.
    pub fn new_semantic(e: impl Into<SemanticError>) -> Self {
        Self::Semantic(e.into())
    }

    pub fn extend_locations(&mut self, locations: Vec<source::FileSpan>) -> Res<()> {
        self.try_into_run_error();
        match self {
            Self::Run(e) => e.locations.extend(locations),
            _ => bail!("cannot set locations for error {:?}", self),
        }
        Ok(())
    }
    pub fn set_behavior(&mut self, trace: cex::Cex) -> Res<()> {
        self.try_into_run_error();
        match self {
            Self::Run(e) => {
                if e.behavior.is_some() {
                    bail!("trying to set behavior twice on error {:?}", self);
                }
                e.behavior = Some(trace);
            }
            _ => bail!("cannot set behavior for error {:?}", self),
        }
        Ok(())
    }

    /// Attempts to forces the error's module.
    pub fn force_module(self, module: ModuleOrTop) -> Self {
        match self {
            Self::Parse(e) => Self::Parse(e.force_module(module)),
            Self::Semantic(e) => Self::Semantic(e.force_module(module)),
            Self::Lexical(e) => Self::Lexical(e.force_module(module)),
            Self::Warning(e) => Self::Warning(e),
            Self::List { during, errs } => {
                let errs = errs
                    .into_iter()
                    .map(|err| err.force_module(module.clone()))
                    .collect();
                Self::List { during, errs }
            }
            Self::NoJavaRuntime | Self::Run(_) | Self::Tlc(_) => self,
        }
    }

    pub fn integrate(&mut self, msg: tlc::msg::Msg) -> Res<()> {
        use tlc::code;
        match msg.code {
            Some(code::TopMsg::Err(code::Err::Tlc(code::TlcErr::TlcNestedExpression {
                locations,
            }))) => self.extend_locations(locations)?,
            // Some(code) => panic!("got code `{}`", code),
            // None => {
            //     println!("subs:");
            //     for line in msg.subs.into_string().unwrap().lines() {
            //         println!("    {}", line);
            //     }
            //     panic!("got no code")
            // }
            _ => bail!("unsupported message {:?}", msg),
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Exception {
    pub exc: Exc,
    pub txt: String,
}
impl Exception {
    /// Constructor.
    pub fn new(exc: Exc, txt: impl Into<String>) -> Self {
        Self {
            exc,
            txt: txt.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Exc {
    Abort,
    NullPointer,
}
impl Exc {
    /// String description.
    pub fn desc(self) -> &'static str {
        match self {
            Self::Abort => "tla2sany.semantic.AbortException",
            Self::NullPointer => "java.lang.NullPointerException",
        }
    }

    /// True if equal to [`Self::Abort`].
    pub fn is_abort(self) -> bool {
        match self {
            Self::Abort => true,
            Self::NullPointer => false,
        }
    }
}
implem! {
    for Exc {
        Display { |&self, fmt| self.desc().fmt(fmt) }
    }
}

#[derive(Debug, Clone)]
pub struct TlcErr {
    pub module: Option<String>,
    pub pos: Option<source::Pos>,
    pub exc: Option<Exc>,
    pub txt: String,
}
impl TlcErr {
    /// Plain message constructor.
    pub fn new_msg(txt: impl Into<String>) -> Self {
        Self {
            module: None,
            pos: None,
            exc: None,
            txt: txt.into(),
        }
    }

    /// Generates the relevant TLC failed outcome.
    pub fn to_outcome(&self) -> Option<tlc::FailedOutcome> {
        Some(tlc::FailedOutcome::ParseError)
    }

    /// Sets the module.
    pub fn in_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }
    /// Sets the position.
    pub fn at_pos(mut self, pos: impl Into<source::Pos>) -> Self {
        self.pos = Some(pos.into());
        self
    }
    /// Sets the exception.
    pub fn with_exc(mut self, exc: impl Into<Exc>) -> Self {
        self.exc = Some(exc.into());
        self
    }

    /// Pretty, multi-line representation of the error.
    pub fn pretty(&self, _project: &FullProject, _styles: &conf::Styles) -> Res<Vec<String>> {
        Ok(self.to_string().lines().map(|s| s.to_string()).collect())
    }
}
implem! {
    for TlcErr {
        Display {
            |&self, fmt| {
                let mut something = false;
                if let Some(module) = self.module.as_ref() {
                    something = true;
                    write!(fmt, "module `{}`", module)?
                }
                if let Some(pos) = self.pos.as_ref() {
                    if something {
                        write!(fmt, " ")?
                    }
                    something = true;
                    write!(fmt, " at {}", pos)?
                }
                if let Some(exc) = self.exc.as_ref() {
                    if something {
                        write!(fmt, " ")?
                    }
                    something = true;
                    write!(fmt, "[{}]", exc)?
                }
                if something {
                    write!(fmt, ": ")?
                }
                write!(fmt, "{}", self.txt)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub module: ModuleOrTop,
    pub err: Option<tlc::code::Err>,
    pub blah: String,
    pub pos: Option<source::FileSpan>,
}
impl SemanticError {
    /// Constructor.
    pub fn new(
        module: impl Into<ModuleOrTop>,
        err: Option<tlc::code::Err>,
        blah: impl Into<String>,
        pos: Option<source::FileSpan>,
    ) -> Self {
        Self {
            module: module.into(),
            err: err,
            blah: blah.into(),
            pos,
        }
    }

    //     /// Constructor.
    //     ///
    //     /// Automatically calls [`Self::finalize`].
    //     pub fn from(lines: &str) -> Res<Self> {
    //         tlc::parse::semantic_error(lines)
    //             .with_context(|| format!("on input lines `{}`", lines))
    //             .context("failed to parse semantic error")
    //             .map(Self::finalize)
    //     }

    /// Forces the error's module.
    pub fn force_module(mut self, module: ModuleOrTop) -> Self {
        self.module = module;
        self
    }

    /// Turns itself into a [`RunError`].
    pub fn into_run_error(self) -> RunError {
        let error = RunErrorKind::Plain(self.blah);
        let mut e = RunError::new(error);
        match self.pos {
            None => (),
            Some(span) => e.locations.push(span),
        };
        e
    }

    /// Generates the relevant TLC failed outcome.
    pub fn to_outcome(&self) -> Option<tlc::FailedOutcome> {
        Some(tlc::FailedOutcome::ParseError)
    }

    //     /// Recognizes dumb counter-productive TLC error messages.
    //     ///
    //     /// Automatically called by [`Self::new`].
    //     pub fn finalize(mut self) -> Self {
    //         match self.msgs.len() {
    //             1 => match &self.msgs[0] {
    //                 TlcErr {
    //                     module: None,
    //                     pos: None,
    //                     exc: Some(Exc::NullPointer),
    //                     txt: terrible_message,
    //                 } if terrible_message
    //                     == r#"cannot invoke "String.length()" because "str" is null"#
    //                     || terrible_message == "exception raised" =>
    //                 {
    //                     self.msgs.extend([
    //                         TlcErr::new_msg(
    //                             "this usually means your module opener/closer \
    //                             is ill-formed or inexistent",
    //                         ),
    //                         TlcErr::new_msg(format!(
    //                             "make sure your module starts with `---- MODULE {} ----`",
    //                             self.module
    //                                 .as_ref()
    //                                 .map(|s| s as &str)
    //                                 .unwrap_or("<module_name>"),
    //                         )),
    //                         TlcErr::new_msg("and ends with `====`"),
    //                     ]);
    //                 }

    //                 _ => (),
    //             },
    //             _ => (),
    //         }
    //         self
    //     }

    /// Pretty multi-line string representation.
    pub fn pretty(&self, project: &FullProject, styles: &conf::Styles) -> Res<Vec<String>> {
        let mut res = vec![];

        let mut line_1 = format!("on file ");
        let file = self.module.to_source(project)?;
        line_1.push_str(
            &styles
                .bold
                .paint(format!("`{}`", file.path().display()))
                .to_string(),
        );
        if let Some(file_span) = self.pos.as_ref() {
            line_1.push_str(" (");
            line_1.push_str(&styles.uline.paint(file_span.to_string()));
            line_1.push_str(")");
        }
        res.push(line_1);

        let mut pref = "- ";
        for (idx, line) in self.blah.lines().enumerate() {
            let line = if idx == 0 {
                format!("{}{}", pref, styles.fatal.paint(line))
            } else {
                format!("{}{}", pref, line)
            };
            res.push(line);
            pref = "  ";
        }

        if let Some(file_span) = self.pos.as_ref() {
            if file_span.pos != file_span.end {
                for line in file_span.pretty_span(
                    |module, buf| project.load_module(module, buf),
                    // &content,
                    Some(&styles.bad.paint("here").to_string()),
                    None,
                )? {
                    res.push(line)
                }
            } else {
                let file_path = file.path();
                let content = io::load_file(file_path)?;

                for line in file_span
                    .pos
                    .pretty(&content, Some(&styles.bad.paint("here").to_string()))?
                {
                    res.push(line)
                }
            }
        }

        if let Some(err) = self.err.as_ref() {
            let line = format!("- TLC-level error: {}", styles.fatal.paint(err.to_string()));
            res.push(line);
        }

        Ok(res)
    }
}
implem! {
    for SemanticError {
        Display {
            |&self, fmt| {
                writeln!(fmt, "Semantic error in module `{}`", self.module)?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LexicalError {
    pub module: ModuleOrTop,
    pub encountered: (String, source::Pos),
    pub code: String,
}
impl LexicalError {
    /// Constructor.
    pub fn new(lines: &str) -> Res<Self> {
        tlc::parse::parse_lexical_error(lines)
            .with_context(|| format!("on input lines `{}`", lines))
            .context("failed to parse lexical error")
    }

    /// Forces the error's module.
    pub fn force_module(mut self, module: ModuleOrTop) -> Self {
        self.module = module;
        self
    }

    /// Pretty multi-line string representation.
    pub fn pretty(&self, project: &FullProject, styles: &conf::Styles) -> Res<Vec<String>> {
        let mut res = vec![];

        let file = self.module.to_source(project)?;
        let file_path = file.path();
        let content = io::load_file(file_path)?;

        {
            let mut line_1 = format!("{}", styles.fatal.paint("lexical error"));
            let file = self.module.to_source(project)?;
            line_1.push_str(&format!(
                " on file {} ({})",
                styles
                    .bold
                    .paint(format!("`{}`", file.path().display().to_string())),
                styles.uline.paint(self.encountered.1.to_string()),
            ));
            res.push(line_1);
        }

        {
            let line_2 = format!(
                "- {}: TLC's lexical errors are more art than science",
                styles.bad.paint("warning"),
            );
            res.push(line_2);
            res.push("  don't trust the error position too much".into());
        }

        {
            let encountered = &self.encountered.0;
            let line_3 = format!(
                "- encountered {}",
                styles.bad.paint(format!("`{}`", encountered))
            );
            res.push(line_3);
        }

        {
            let code_line_count = self.code.lines().count();
            if code_line_count > 0 {
                let pos = &self.encountered.1;

                let partial_start_line = self
                    .code
                    .lines()
                    .next()
                    .expect("string with more than one line must have at least one line");
                let start_line = content
                    .lines()
                    .nth(pos.row - code_line_count)
                    .ok_or_else(|| anyhow!("failed retrieve start line of file position"))?;
                let col = if start_line.len() >= partial_start_line.len() {
                    1 + start_line.len() - partial_start_line.len()
                } else {
                    start_line.chars().take_while(|c| c.is_whitespace()).count()
                };

                let start = source::Pos::new(1 + pos.row - code_line_count, col);
                // println!("partial start line: `{}`", partial_start_line);
                // println!("start line: `{}`", start_line);
                // println!("col: {}", col);
                // println!("pos: {}", pos);
                // println!("code_line_count: {}", code_line_count);
                // println!("start: {}", start);
                let start_text = &format!(
                    "{} {}",
                    styles.bad.paint("while TLC was parsing"),
                    styles.bold.paint("this"),
                );
                let end_text = &format!("{}", styles.fatal.paint("error reported here"));
                for line in start.pretty_span(pos, &content, Some(start_text), Some(end_text))? {
                    res.push(format!("  {}", line))
                }
            } else {
                for line in self.encountered.1.pretty(&content, Some("here"))? {
                    res.push(format!("  {}", line))
                }
            }
        }

        // if let Some(first_line) = self.code.lines().next() {
        //     let code_line_count = self.code.lines().count();
        //     let pos = &self.encountered.1;
        //     if pos.row + 1 < code_line_count {
        //         bail!(
        //             "illegal lexical error, error line is {} but string-code given has {} line(s)",
        //             pos.row,
        //             code_line_count,
        //         );
        //     }
        //     let col = first_line.chars().filter(|c| c.is_whitespace()).count();
        //     let pos = source::Pos::new(pos.row - 1 - code_line_count, col);

        //     res.push("- while parsing".into());
        //     for line in pos.pretty(&content, Some("this line"))? {
        //         res.push(format!("  {}", line));
        //     }
        // }

        Ok(res)
    }

    /// Generates the relevant TLC failed outcome.
    pub fn to_outcome(&self) -> Option<tlc::FailedOutcome> {
        Some(tlc::FailedOutcome::ParseError)
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub module: ModuleOrTop,
    pub expected: String,
    pub encountered: (String, source::Pos, Option<String>),
    pub trace: Vec<(String, source::Pos)>,
}
impl ParseError {
    /// Constructor.
    pub fn new(lines: &str) -> Res<Self> {
        tlc::parse::parse_parse_error(lines)
            .with_context(|| format!("on input lines `{}`", lines))
            .context("failed to parse parse error")
    }

    /// Forces the error's module.
    pub fn force_module(mut self, module: ModuleOrTop) -> Self {
        self.module = module;
        self
    }

    /// Generates the relevant TLC failed outcome.
    pub fn to_outcome(&self) -> Option<tlc::FailedOutcome> {
        Some(tlc::FailedOutcome::ParseError)
    }

    /// Pretty multi-line string representation.
    pub fn pretty(&self, project: &FullProject, styles: &conf::Styles) -> Res<Vec<String>> {
        let mut res = vec![];

        let mut line_1 = format!("parse error on ");
        let file = self.module.to_source(project)?;
        line_1.push_str(&format!(
            "file {}",
            styles
                .bold
                .paint(format!("`{}`", file.path().display().to_string()))
        ));
        res.push(line_1);

        let file_path = file.path();
        let content = io::load_file(file_path)?;

        {
            let (encountered, pos, and) = &self.encountered;
            let mut s = format!(
                "- expected {}, encountered {}",
                styles.good.paint(&self.expected),
                styles.bad.paint(format!("`{}`", encountered))
            );
            if let Some(and) = and {
                s.push_str(&format!(" and `{}`", styles.bad.paint(and)));
            }
            res.push(s);

            res.push(format!(
                "  {} at {}",
                styles.uline.paint(file_path.display().to_string()),
                styles.bold.paint(pos.to_string()),
            ));
            for line in pos.pretty(&content, Some("here"))? {
                res.push(format!("  {}", line));
            }
        }

        {
            res.push("- while parsing".into());
            for (desc, pos) in self.trace.iter() {
                res.push(format!(
                    "  {} at {}",
                    styles.uline.paint(file_path.display().to_string()),
                    styles.bold.paint(pos.to_string()),
                ));
                for line in pos.pretty(&content, Some(desc))? {
                    res.push(format!("  {}", line));
                }
            }
        }

        Ok(res)
    }
}
implem! {
    for ParseError {
        Display {
            |&self, fmt| {
                writeln!(fmt, "Parse error in module `{}`", self.module)?;
                writeln!(fmt, "- expected {}", self.expected)?;
                let (got, at, token_opt) = &self.encountered;
                writeln!(fmt, "- encountered `{}` at {}", got, at)?;
                if let Some(token) = token_opt {
                    writeln!(fmt, "  and token `{}`", token)?;
                }
                if !self.trace.is_empty() {
                    writeln!(fmt, "- stack trace:")?;
                    for (desc, at) in self.trace.iter() {
                        writeln!(fmt, "  `{}`, starting at {}", desc, at)?;
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum RunErrorKind {
    AssertFailed {
        /// Failure message.
        msg: Option<String>,
    },
    Plain(String),
}
impl RunErrorKind {
    /// Pretty single-line string representation.
    pub fn pretty(&self, _project: &FullProject) -> Res<String> {
        let styles = conf::Styles::new();
        match self {
            Self::AssertFailed { msg } => {
                let mut res = format!("an {}", styles.fatal.paint("assertion failed"));
                if let Some(msg) = msg {
                    res.push_str(" with ");
                    res.push_str(&format!(
                        "{}{}{}",
                        styles.bad.paint("\""),
                        styles.bad.paint(msg),
                        styles.bad.paint("\""),
                    ));
                }
                Ok(res)
            }
            Self::Plain(msg) => Ok(msg.clone()),
        }
    }

    /// Generates the relevant TLC failed outcome.
    pub fn to_outcome(&self) -> Option<tlc::FailedOutcome> {
        match self {
            Self::AssertFailed { .. } => Some(tlc::FailedOutcome::AssertFailed),
            Self::Plain(s) => Some(tlc::FailedOutcome::Plain(s.clone())),
        }
    }
}
implem! {
    for RunErrorKind {
        From<String> { |s| Self::Plain(s) }
    }
}

#[derive(Debug, Clone)]
pub struct RunError {
    pub error: RunErrorKind,
    pub behavior: Option<cex::Cex>,
    pub locations: Vec<source::FileSpan>,
}
impl RunError {
    /// Constructor.
    pub fn new(kind: impl Into<RunErrorKind>) -> Self {
        Self {
            error: kind.into(),
            behavior: None,
            locations: vec![],
        }
    }

    /// Generates the relevant TLC failed outcome.
    pub fn to_outcome(&self) -> Option<tlc::FailedOutcome> {
        self.error.to_outcome()
    }

    /// Pretty multi-line string representation.
    pub fn pretty(&self, project: &FullProject, styles: &conf::Styles) -> Res<Vec<String>> {
        let mut res = vec![self.error.pretty(project)?];

        if !self.locations.is_empty() {
            // Most revelant part is the last file-span on a file that's not the matla module. We
            // want to print the relevant part in full, and show everything else more concisely.
            res.push("".into());
            res.push("- triggered at".into());

            // This flag indicates whether we have already handled the relevant part of the list.
            let mut handled_relevant = false;

            let load_module = |module: &str, buf: &mut String| project.load_module(module, buf);

            // Skip elements that start on the same line of the same module.
            let mut last: Option<(&String, usize)> = None;

            for span in self.locations.iter().rev() {
                let pref = "  ";
                if &span.file == crate::matla::MATLA_MODULE_NAME {
                    continue;
                }

                if !handled_relevant {
                    res.push(format!("  module {}", styles.good.paint(span.to_string())));
                    handled_relevant = true;
                    let lines = span.pretty_span(
                        load_module,
                        None,
                        None, // Some(&styles.bold.paint("triggered here").to_string()),
                    )?;
                    for line in lines {
                        res.push(format!("{}{}", pref, line));
                    }
                } else if project.tlc_cla.print_callstack {
                    let nu_last = Some((&span.file, span.pos.row));
                    if last == nu_last {
                        continue;
                    }
                    last = nu_last;
                    res.push(format!("  module {}", styles.good.paint(span.to_string())));
                    let lines = span.start.pretty(load_module, None)?;
                    for line in lines {
                        res.push(format!("{}{}", pref, line));
                    }
                }
            }
        }

        if let Some(behavior) = self.behavior.as_ref() {
            res.push("".into());
            res.push(format!("- while exploring this trace"));
            let spec = cex::pretty::Spec::default();
            let mut buf = String::new();
            spec.cex_to_ml_string(behavior, &mut buf);
            for line in buf.lines() {
                res.push(line.into());
            }
        }

        Ok(res)
    }
}
