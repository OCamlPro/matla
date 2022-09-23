//! Type used by modes to control the [`Runtime`][tlc::runtime].
//!
//! When handling a message, modes issue a [`Control`] to let the runtime know what it must do.

use super::*;

/// Controls the [`Runtime`], issued by modes when asked to deal with a message.
///
/// Note that the mode takes ownership of itself when dealing with a message. This is because, in
/// general, the mode might replace itself with a new one.
#[derive(Debug, Clone)]
pub enum Control {
    /// Tells the runtime that the mode ignores this message.
    ///
    /// The mode stored is the original mode that was queried.
    Ignored(TlcMode),
    /// Replaces the current mode with another one, *i.e.* the active mode changes.
    Replace(TlcMode),
    /// Keeps the current mode and optionally activate a sub-mode.
    Keep(TlcMode, Option<TlcMode>),
    /// Done, with an outcome.
    Finalize(ModeOutcome),
}
impl Control {
    /// Ignore-mode message constructor.
    pub fn ignored(out: impl Into<TlcMode>) -> Self {
        Self::Ignored(out.into())
    }
    /// Finalization constructor.
    pub fn finalize(out: impl Into<ModeOutcome>) -> Self {
        Self::Finalize(out.into())
    }
    /// Normal finalization (no error).
    pub fn done(safe: bool) -> Self {
        Self::finalize(ModeOutcome::new_success(safe))
    }
    /// Mode replacement constructor.
    pub fn replace(mode: impl Into<TlcMode>) -> Self {
        Self::Replace(mode.into())
    }
    /// Single next-state constructor.
    pub fn keep(mode: impl Into<TlcMode>) -> Self {
        Self::Keep(mode.into(), None)
    }
    /// Double next-state constructor.
    pub fn keep_and(fst: impl Into<TlcMode>, snd: impl Into<TlcMode>) -> Self {
        Self::Keep(fst.into(), Some(snd.into()))
    }

    /// Wraps itself in a `Some`.
    pub fn some(self) -> Option<Self> {
        Some(self)
    }
    /// Wraps itself in a `Ok`.
    pub fn ok(self) -> Res<Self> {
        Ok(self)
    }
    /// Wraps itself in an `Ok(Some(_))`.
    pub fn ok_some(self) -> Res<Option<Self>> {
        Ok(Some(self))
    }
}
implem! {
    for Control {
        From<ModeOutcome> {
            |outcome| Self::Finalize(outcome)
        }
    }
}
