//! Type used by modes to control the runtime.

use super::*;

#[derive(Debug, Clone)]
pub enum Control {
    Ignored(TlcMode),
    Replace(TlcMode),
    Keep(TlcMode, Option<TlcMode>),
    Finalize(ModeOutcome),
}
impl Control {
    /// Ignored message constructor.
    pub fn ignored(out: impl Into<TlcMode>) -> Self {
        Self::Ignored(out.into())
    }
    /// Finalization constructor.
    pub fn finalize(out: impl Into<ModeOutcome>) -> Self {
        Self::Finalize(out.into())
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
