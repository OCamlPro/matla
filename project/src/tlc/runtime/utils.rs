//! TLC runtime helpers.

use super::*;

/// Reports messages with unexpected sub-messages.
pub fn report_subs(desc: &str, msg: &tlc::msg::Msg) {
    if msg.has_sub_msgs() {
        log::error!(
            "unexpected TLC output with nested messages in `{}` mode",
            desc
        );
        for line in msg.to_string().lines() {
            log::error!("{}", line);
        }
    }
}
/// Fails on messages that have no code.
pub fn code_of<'msg>(desc: &str, msg: &'msg tlc::msg::Msg) -> Res<&'msg code::Msg> {
    msg.code
        .as_ref()
        .ok_or_else(|| anyhow!("unexpected plain message in TLC run mode `{}`", desc))
        .and_then(|top| top.into_res(msg))
}
/// Reports unexpected messages and returns itself.
pub fn report_unexpected(desc: &str, msg: &tlc::msg::Msg) {
    log::error!("unexpected TLC output in `{}` mode", desc);
    for line in msg.to_string().lines() {
        log::error!("{}", line);
    }
}
