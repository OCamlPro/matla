//! Handles initial states mode.

prelude!(tlc::code, tlc::msg);

/// Result of the initial state phase.
#[derive(Debug, Clone)]
pub struct Init {
    /// Number of initial states generated.
    pub state_count: usize,
    /// Date-time at which the initial state phase ended.
    pub end_time: chrono::NaiveDateTime,
}
impl Init {
    /// Constructor from a message.
    pub fn new(msg: msg::Msg) -> Res<Either<Self, msg::Msg>> {
        match msg.code {
            Some(code::TopMsg::Msg(code::Msg::Status(code::Status::TlcInitGenerated1))) => {
                Self::new_1(msg)
                    .map(Left)
                    .context("failed to handle TlcInitGenerated1 message")
            }
            _ => Ok(Right(msg)),
        }
    }

    fn new_1(mut msg: msg::Msg) -> Res<Self> {
        let (state_count, end_time) = {
            let mut iter = msg.subs.drain(0..);
            let res = match iter.next() {
                Some(Left(line)) => tlc::parse::init_generated_1(&line)
                    .with_context(|| anyhow!("failed to parse message line `{}`", line))?,
                Some(Right(msg)) => {
                    return Err(Error::msg(msg.to_string())
                        .context("expected plain text, found sub-message"))
                }
                None => bail!("expected plain text, found nothing"),
            };
            match iter.next() {
                Some(Left(line)) => {
                    return Err(
                        Error::msg(line).context("expected exactly one line, found at least two")
                    )
                }
                Some(Right(msg)) => {
                    return Err(Error::msg(msg.to_string())
                        .context("expected exactly one line, found additional sub-message"))
                }
                None => (),
            }
            res
        };

        Ok(Self {
            state_count,
            end_time,
        })
    }
}
