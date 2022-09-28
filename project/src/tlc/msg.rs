//! Handles messages from the TLC thread.
//!
//! Messages are constructed using everything in [`code`], in particular [`TopMsg`]'s `parse_start`
//! and `parse_end` functions. Since TLC messages can be nested, main message type [`Msg`] stores
//! the top-level message and its [`Elms`], *i.e.* a list of sub-messages. An element of this list
//! is either a proper [`Msg`] or a plain `String`, the latter corresponding to content (text) for
//! the top-level [`Msg`].
//!
//! [`code`]: tlc::code (TLC message code module)
//! [`TopMsg`]: tlc::code::TopMsg (TopMsg in the TLC message code module)

prelude!(
    thread::{ChildCmd, ChildCmdCom},
);

/// Elements of a message: a list of plain strings or sub-messages.
///
/// [`Msg`] attaches `Elms` to a top-level message. Plain strings in the `elms` indicate content
/// (text) for that top-level message, while [`Msg`]s are actual sub-messages.
///
/// I think content can only come before sub-messages, but I'm really not sure about this.
#[derive(Debug, Clone)]
pub struct Elms {
    pub elms: Vec<Either<String, Msg>>,
}
implem! {
    for Elms {
        Deref<Target = Vec<Either<String, Msg>>> {
            |&self| &self.elms,
            |&mut self| &mut self.elms,
        }
        From<Vec<Either<String, Msg>>> {
            |elms| Self { elms }
        }
    }
}
impl Elms {
    /// Empty elements.
    pub const EMPTY: Self = Self { elms: Vec::new() };

    /// Constructor.
    pub fn new() -> Self {
        Self::EMPTY.clone()
    }

    /// Iterator over the plain-string elements.
    pub fn plain_str_slices(&self) -> impl Iterator<Item = &str> {
        self.elms
            .iter()
            .filter_map(|either| either.as_ref().left().map(|s| s as &str))
    }

    /// Iterator over the plain-string elements, error on sub-messages.
    pub fn only_plain_str_slices(&self) -> impl Iterator<Item = Res<&str>> {
        self.elms.iter().map(|either| match either {
            Left(s) => Ok(s as &str),
            Right(msg) => {
                let desc: &'static str = msg
                    .code
                    .as_ref()
                    .map(|code| code.desc())
                    .unwrap_or("plain text");
                Err(anyhow!("unexpected sub-message `{}`", desc))
            }
        })
    }

    /// Removes some plain text lines at the beginning of the elements.
    ///
    /// True if lines were removed.
    pub fn starts_with_rm<I, BStr>(&mut self, lines: I) -> bool
    where
        I: IntoIterator<Item = BStr>,
        BStr: std::borrow::Borrow<str>,
    {
        let mut count = 0;
        for (idx, line) in lines.into_iter().enumerate() {
            count += 1;
            let line = line.borrow();
            match self.elms.get(idx) {
                None | Some(Either::Right(_)) => return false,
                Some(Either::Left(l)) => {
                    if l == line {
                        continue;
                    } else {
                        return false;
                    }
                }
            }
        }
        // all `count` first lines correspond to `lines`
        let capa = self.elms.len() - count;
        let old = mem::replace(&mut self.elms, Vec::with_capacity(capa));
        self.elms.extend(old.into_iter().skip(count));
        true
    }

    /// Turns itself into a single string, fails on nested messages.
    pub fn into_string(self) -> Res<String> {
        let mut acc: Option<String> = None;
        for line in self.elms.into_iter() {
            match line {
                Left(line) => {
                    if let Some(acc) = acc.as_mut() {
                        acc.push('\n');
                        acc.push_str(&line);
                    } else {
                        acc = Some(line)
                    }
                }
                Right(msg) => {
                    let desc: &'static str = msg
                        .code
                        .as_ref()
                        .map(|code| code.desc())
                        .unwrap_or("plain text");
                    return Err(anyhow!("unexpected sub-message `{}`", desc));
                }
            }
        }
        Ok(acc.unwrap_or_else(String::new))
    }
}

/// This macro generates functions that extract a precise number of string elements of an [`Elms`].
///
/// These functions fail if the [`Elms`] do not contain **exactly** the number of lines expected,
/// and no sub-[`Msg`].
///
/// That's convenient because **many** messages (errors in particular) are not structured besides
/// a message-code opener, a message-code closer, and plain text in between. For these messages,
/// parsing works on arbitrary natural language between the opener/closer.
macro_rules! msg_elms_getter {
    // this macro is recursive, that's the termination case
    () => {};
    (
        $(#[$meta:meta])*
        $name:ident -> Res<$out:ty> {
            $count:literal => ( $($sub:ident),+ $(,)? )
        }
        // tail, this is a recursive macro
        $($tail:tt)*
    ) => {
        $(#[$meta])*
        pub fn $name(&self) -> Res<$out> {
            let mut iter = self.only_plain_str_slices();
            $(
                let $sub = iter.next()
                    .ok_or_else(
                        || anyhow!(
                            "expected {} plain text elements, got {}",
                            $count,
                            self.elms.len(),
                        )
                    )??;
            )+
            if iter.next().is_some() {
                bail!("expected {} plain text elements, got {}", $count, self.elms.len())
            } else {
                Ok(( $($sub),+ ))
            }
        }

        // recursive call on the tail
        msg_elms_getter! { $($tail)* }
    };
}
impl Elms {
    msg_elms_getter! {
        /// Unpacks an [`Elms`] containing exactly one plain string element.
        get_1_plain_str -> Res<&str> {
            1 => (str1)
        }
        /// Unpacks an [`Elms`] containing exactly two plain string elements.
        get_2_plain_str -> Res<(&str, &str)> {
            2 => (str1, str2)
        }
        /// Unpacks an [`Elms`] containing exactly three plain string elements.
        get_3_plain_str -> Res<(&str, &str, &str)> {
            3 => (str1, str2, str3)
        }
        /// Unpacks an [`Elms`] containing exactly four plain string elements.
        get_4_plain_str -> Res<(&str, &str, &str, &str)> {
            4 => (str1, str2, str3, str4)
        }
    }
}

/// A qualified message from TLC.
///
/// The message code is optional because TLC only wraps messages in message-code opener/closer when
/// it feels like it. When it does not, we do whatever we can to recognize what it's trying to say.
#[derive(Debug, Clone)]
pub struct Msg {
    /// Message code.
    ///
    /// Absence of code encodes a print statement.
    pub code: Option<tlc::code::TopMsg>,
    /// Lines of the message or sub-messages.
    pub subs: tlc::msg::Elms,
    /// True if the message was produced on `stderr`.
    pub from_stderr: bool,
}
impl Msg {
    /// Constructor.
    pub fn new(
        code: Option<tlc::code::TopMsg>,
        mut subs: tlc::msg::Elms,
        from_stderr: bool,
    ) -> Self {
        if code
            .as_ref()
            .map(tlc::code::TopMsg::is_general)
            .unwrap_or(false)
            && subs.len() == 1
            && subs.last().map(Either::is_right).unwrap_or(false)
        {
            subs.pop()
                .expect("pop on vector of length 1")
                .expect_right("right variant of either")
        } else {
            Self {
                code,
                subs,
                from_stderr,
            }
            .simplify_try_flatten()
        }
    }

    /// True if the message starts with some plain text lines.
    pub fn starts_with_rm<I, BStr>(&mut self, lines: I) -> bool
    where
        I: IntoIterator<Item = BStr>,
        BStr: std::borrow::Borrow<str>,
    {
        self.subs.starts_with_rm(lines)
    }

    /// If the message is just a single sub-message, return that.
    pub fn try_flatten(mut self) -> Msg {
        if self.subs.len() == 1 {
            match self.subs.pop() {
                Some(msg @ Either::Left(_)) => {
                    self.subs.push(msg);
                }
                Some(Either::Right(sub)) => return sub,
                None => unreachable!(),
            }
        }
        self
    }

    /// Tries to flatten a message.
    ///
    /// Recognizes obfuscation patterns such as a general message with useless plain text
    /// information containing an error.
    pub fn simplify_try_flatten(mut self) -> Msg {
        if self.starts_with_rm(
            "\
TLC threw an unexpected exception.
This was probably caused by an error in the spec or model.
See the User Output or TLC Console for clues to what happened.
The exception was a java.lang.RuntimeException\
            "
            .lines(),
        ) {
            let msg = self.try_flatten();
            msg
        } else {
            self
        }
    }

    /// Turns itself into an error.
    pub fn into_err(self) -> Res<(tlc::code::Err, tlc::msg::Elms)> {
        let code = self
            .code
            .ok_or_else(|| anyhow!("cannot turn codeless message into an error"))?;
        match code {
            tlc::code::TopMsg::Msg(_) => bail!("cannot turn non-error message into an error"),
            tlc::code::TopMsg::Err(e) => Ok((e, self.subs)),
        }
    }

    /// Code accessor.
    pub fn code(&self) -> Option<&tlc::code::TopMsg> {
        self.code.as_ref()
    }

    /// Retrieves all the lines, as a vector.
    pub fn lines<'a>(&'a self) -> Vec<&'a str> {
        fn doit<'b>(slf: &'b Msg, vec: &mut Vec<&'b str>) {
            for sub in slf.subs.iter() {
                match sub {
                    Either::Left(line) => vec.push(line),
                    Either::Right(sub) => doit(sub, vec),
                }
            }
        }
        let mut vec = Vec::with_capacity(7);
        doit(self, &mut vec);
        vec.shrink_to_fit();
        vec
    }

    /// Text lines of a codeless message.
    ///
    /// Fails if the code is not `None`, or there are sub-messages.
    pub fn lines_of_codeless<'a>(&'a self) -> Res<Vec<&'a str>> {
        if let Some(code) = self.code.as_ref() {
            bail!("message is not codeless: {}", code)
        }
        if self.has_sub_msgs() {
            bail!("cannot extract lines of message with sub-messages")
        }
        Ok(self.lines())
    }

    /// True if the message or any of its sub-messages is an error.
    pub fn has_err(&self) -> bool {
        if self.code().map(tlc::code::TopMsg::is_err).unwrap_or(false) {
            return true;
        }
        self.subs.iter().any(|sub| {
            sub.as_ref()
                .right()
                .map(|msg| msg.has_err())
                .unwrap_or(false)
        })
    }

    /// True if the message has sub-messages.
    pub fn has_sub_msgs(&self) -> bool {
        self.subs
            .iter()
            .any(|sub| if sub.is_right() { true } else { false })
    }

    /// Message source, `stdout` or `stderr`.
    pub fn source(&self) -> &'static str {
        if self.from_stderr {
            "`stderr`"
        } else {
            "`stdout`"
        }
    }

    /// True if the message is a the result of a TLC-level print.
    pub fn is_print(&self) -> bool {
        self.code.is_none()
    }
}

implem! {
    impl(T: Into<tlc::code::TopMsg>) for Msg {
        From<T> {
            |t| Msg {
                code: Some(t.into()),
                subs: tlc::msg::Elms::new(),
                from_stderr: false,
            }
        }
    }
    for Msg {
        Display {
            |&self, fmt| {
                if let Some(code) = self.code.as_ref() {
                    write!(fmt, "|===[{}] ", code)?;
                } else {
                    for (idx, sub) in self.subs.iter().enumerate() {
                        if idx > 0 { writeln!(fmt)? }
                        match sub {
                            Either::Left(line) => write!(fmt, "> {}", line.trim())?,
                            Either::Right(msg) => {
                                for line in msg.to_string().lines() {
                                    write!(fmt, "> {}", line.trim())?
                                }
                            }
                        }
                    }
                    return Ok(())
                }
                if self.from_stderr {
                    writeln!(fmt, "from stderr")?
                } else {
                    writeln!(fmt, "from stdout")?
                }
                for sub in self.subs.iter() {
                    match sub {
                        Either::Left(line) => writeln!(fmt, "| {}", line.trim())?,
                        Either::Right(sub) => {
                    let sub = sub.to_string();
                    for line in sub.lines() {
                        writeln!(fmt, "| {}", line)?;
                    }
                }}}
                write!(fmt, "|===|")?;

                Ok(())
            }
        }
    }
}

/// A TLC communication channel.
///
/// Stores a two-way communication channel [`ChildCmdCom`] with the actual TLC process.
///
/// Distinguishes between `stdout` and `stderr` messages by storing a separate message stack for
/// each. Message stacks are needed because TLC messages can be nested.
///
/// Also stores a `handle` on the child process so that it can kill it, and a list of all the errors
/// that happened during the run.
pub struct TlcHandler {
    /// Child channel.
    com: ChildCmdCom,
    /// Message from `stdout` under construction.
    stdout_msg: Vec<(Code, tlc::msg::Elms)>,
    /// Message from `stderr` under construction.
    stderr_msg: Vec<(Code, tlc::msg::Elms)>,
    /// Join handle for the child process.
    handle: thread::JoinHandle<Res<io::ExitStatus>>,
    /// Errors that happened during the run.
    errors: Vec<tlc::code::Err>,
}
impl TlcHandler {
    /// Constructor.
    ///
    /// - `cmd`: assumed to be a full TLC call, with `-tool`.
    pub fn new(cmd: io::Command) -> Self {
        let (child, com) = ChildCmd::new(cmd);
        let handle = child.spawn();
        Self {
            com,
            stdout_msg: vec![],
            stderr_msg: vec![],
            handle,
            errors: Vec::new(),
        }
    }

    /// Destroys the process regarless of its state.
    pub fn destroy(self) -> Res<tlc::ProcessOutcome> {
        self.com.destroy();
        self.handle
            .join()
            .map_err(|err| anyhow!("TLC-process panic: {:?}", err))?
            .and_then(|out| tlc::ProcessOutcome::new(out.code().unwrap_or(-1)))
    }

    /// Joins with the underlying child process.
    ///
    /// Returns an error if `self.next() == Ok(Some(_))`.
    pub fn join(mut self) -> Res<tlc::ProcessOutcome> {
        log::trace!("joining child process");
        if let Some(msg) = self.next().context("retrieving message of child process")? {
            bail!(
                "trying to join child process but it's not done ({} message available)",
                if let Some(code) = msg.code {
                    format!(" {}", code)
                } else {
                    "".into()
                }
            );
        }
        self.handle
            .join()
            .map_err(|_| anyhow!("child process panicked"))
            .and_then(|res| match res {
                Ok(exit_status) => {
                    let code = exit_status
                        .code()
                        .ok_or_else(|| anyhow!("exit code not available from child exit status"))?;
                    Ok(tlc::ProcessOutcome::new(code)?)
                }
                Err(e) => Err(e),
            })
    }

    /// Returns the next message, if any.
    pub fn next(&mut self) -> Res<Option<Msg>> {
        macro_rules! next {
            { ($line:pat, $from_stderr:pat) => $($action:tt)* } => {
                match self.com.next() {
                    None => return Ok(None),
                    Some(Err(e)) => return Err(e),
                    Some(Ok(($line, $from_stderr))) => {
                        $($action)*
                    }
                }
            };
        }

        let msg = loop {
            next! { (line, from_stderr) =>
                log::trace!("handling line from TLC");
                log::trace!("`{}`", line);
                if let Some((code, _)) = tlc::code::TopMsg::parse_start(&line)? {
                    self.new_msg(code, from_stderr)?;
                } else if let Some(code) = tlc::code::TopMsg::parse_end(&line)? {
                    if let Some(msg) = self.end_msg(code, from_stderr)? {
                        break msg;
                    }
                } else {
                    let line = line.trim();
                    if !line.is_empty() {
                        if self.is_building_msg(from_stderr) {
                            self.msg_push(line, from_stderr)?;
                        } else {
                            break Msg::new(
                                None,
                                vec![Either::Left(line.into())].into(),
                                from_stderr,
                            );
                        }
                    }
                }
            }
        };

        Ok(Some(msg))
    }

    /// True if there is a message under construction.
    pub fn is_building_msg(&self, from_stderr: bool) -> bool {
        if from_stderr {
            !self.stderr_msg.is_empty()
        } else {
            !self.stdout_msg.is_empty()
        }
    }

    /// Creates a message to construct.
    fn new_msg(&mut self, code: Code, from_stderr: bool) -> Res<()> {
        let target = if from_stderr {
            &mut self.stderr_msg
        } else {
            &mut self.stdout_msg
        };
        target.push((code, tlc::msg::Elms::new()));
        Ok(())
    }

    /// Finalizes a message under construction.
    fn end_msg(&mut self, code: Code, from_stderr: bool) -> Res<Option<Msg>> {
        let target = if from_stderr {
            &mut self.stderr_msg
        } else {
            &mut self.stdout_msg
        };
        let source = if from_stderr { "`stderr`" } else { "`stdout`" };

        if let Some((msg_code, subs)) = target.pop() {
            if code == msg_code {
                let msg = tlc::code::TopMsg::from_code(msg_code, &subs)?;
                if let Some(err) = msg.as_ref().and_then(tlc::code::TopMsg::as_err) {
                    self.errors.push(err.clone());
                }
                let msg = Msg::new(msg, subs, from_stderr);
                if let Some((_, subs)) = target.last_mut() {
                    subs.push(Either::Right(msg));
                    Ok(None)
                } else {
                    Ok(Some(msg))
                }
            } else {
                bail!(
                    "trying to end `#{}` message from {}, but message under construction is `#{}`",
                    code,
                    source,
                    msg_code,
                )
            }
        } else {
            bail!(
                "trying to end message from {}, but no message is under construction",
                source,
            )
        }
    }

    /// Pushes a line to a message under construction.
    fn msg_push(&mut self, line: impl Into<String>, from_stderr: bool) -> Res<()> {
        let target = if from_stderr {
            &mut self.stderr_msg
        } else {
            &mut self.stdout_msg
        };
        if let Some((_, subs)) = target.last_mut() {
            let line = line.into();
            if !line.trim().is_empty() {
                subs.push(Either::Left(line.into()));
            }
            Ok(())
        } else {
            bail!(
                "trying to push line to {} message, but no message is under construction",
                if from_stderr { "`stderr`" } else { "`stdout`" },
            )
        }
    }
}
