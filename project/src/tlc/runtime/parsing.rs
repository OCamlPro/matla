//! Parsing mode.

use super::*;

#[derive(Debug, Clone)]
pub struct Parsing {
    pub current_file: Option<ModuleOrTop>,
    pub error_msg: Option<String>,
}

impl Parsing {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            current_file: None,
            error_msg: None,
        }
    }
}

impl Parsing {
    /// Reports an error (if any) to `out`.
    pub fn try_report_error(&self, out: &mut impl tlc::Out) -> Res<bool> {
        if let Some(error) = self.error_msg.as_ref() {
            let top = ModuleOrTop::TopTla;
            let module = self.current_file.as_ref().unwrap_or(&top);
            let error = tlc::err::TlcError::new_parse(&error, module)?;
            out.handle_error(error)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl IsMode for Parsing {
    fn desc(&self) -> &'static str {
        "parsing"
    }
    fn handle_plain(
        mut self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
    ) -> Res<Option<Control>> {
        debug_assert!(msg.code.is_none());
        // println!("plain message:");
        // for line in msg.lines() {
        //     println!("| {}", line);
        // }
        let mut lines = msg.lines().into_iter();

        let line = match (msg.subs.len(), lines.next()) {
            (1, Some(line @ "Semantic errors:")) => {
                if self.error_msg.is_none() {
                    self.error_msg = Some("".into())
                }
                line
            }
            (1, Some(line)) => line,
            (count, _) => {
                out.handle_message(msg, log::Level::Info);
                bail!("expected exactly one plain message, got {}", count);
            }
        };

        match self.error_msg.as_mut() {
            None => tlc::parse::parsing(line, &mut self)
                .with_context(|| anyhow!("while parsing plain message `{}`", line))?,
            Some(error) => {
                for line in msg.lines() {
                    if !error.is_empty() {
                        error.push('\n');
                    }
                    error.push_str(line);
                }
            }
        }

        Control::Keep(self.into(), None).ok_some()
    }
    fn handle_error(
        self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        err: &code::Err,
        _reported: bool,
    ) -> Res<Option<Control>> {
        self.try_report_error(out)?;
        tlc::runtime::IsMode::handle_error(self, out, msg, err, true)
    }
    fn handle_msg(
        self,
        out: &mut impl tlc::Out,
        msg: &tlc::msg::Msg,
        tlc_msg: &code::Msg,
    ) -> Res<Option<Control>> {
        use code::*;
        match tlc_msg {
            // Parsing ends.
            Msg::Status(Status::TlcSanyEnd) => {
                out.handle_message(&msg, log::Level::Debug);
                Control::keep(self).ok_some()
            }
            // Going to analysis mode.
            Msg::Status(Status::TlcStarting) => {
                out.handle_message(&msg, log::Level::Debug);
                let reported = self.try_report_error(out)?;
                if !reported {
                    Control::replace(Starting).ok_some()
                } else {
                    Control::replace(Error::empty(true)).ok_some()
                }
            }
            _ => Control::ignored(self).ok_some(),
        }
    }
}
