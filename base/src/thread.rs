//! Thread-related things.

use crate::*;

pub use std::thread::{sleep, spawn, JoinHandle};

/// Messages sent by a [`ChildCmd`].
#[derive(Debug, Clone)]
pub enum OutMsg {
    /// A line from stdout.
    Stdout(String),
    /// A line from stderr.
    Stderr(String),
    /// An error occurred.
    Fail(String),
    /// Child is done.
    Done,
}
impl OutMsg {
    /// Stdout or stderr constructor.
    pub fn msg(s: impl Into<String>, is_stdout: bool) -> Self {
        if is_stdout {
            Self::Stdout(s.into())
        } else {
            Self::Stderr(s.into())
        }
    }
    /// Fail constructor.
    pub fn fail(s: impl Into<String>) -> Self {
        Self::Fail(s.into())
    }
    /// Done constructor.
    pub fn done() -> Self {
        Self::Done
    }
}
/// Messages sent to a [`ChildCmd`].
#[derive(Debug, Clone, Copy)]
pub enum InMsg {
    /// Kill the child.
    Kill,
}

/// Communicates with a [`ChildCmd`].
#[derive(Debug)]
pub struct ChildCmdCom {
    /// [`InMsg`] sender.
    pub send: mpsc::Sender<InMsg>,
    /// [`OutMsg`] receiver.
    pub recv: mpsc::Receiver<OutMsg>,
}
impl ChildCmdCom {
    /// Constructor.
    fn new(send: mpsc::Sender<InMsg>, recv: mpsc::Receiver<OutMsg>) -> Self {
        Self { send, recv }
    }
    /// Destroys itself and sends a kill order to the child process.
    pub fn destroy(self) {
        let res = self.send.send(InMsg::Kill);
        if res.is_err() {
            log::debug!("failed to send kill order to child, disconnected")
        }
    }
    /// Retrieves the next line, `bool` flag is true for lines from `stderr`.
    pub fn next(&mut self) -> Option<Res<(String, bool)>> {
        match self.recv.recv() {
            Ok(OutMsg::Stdout(line)) => Some(Ok((line, false))),
            Ok(OutMsg::Stderr(line)) => Some(Ok((line, true))),
            Ok(OutMsg::Fail(err)) => Some(Err(anyhow::Error::msg(err))),
            Ok(OutMsg::Done) => None,
            Err(_) => None,
        }
    }
}

/// A separate child running a command.
#[derive(Debug)]
pub struct ChildCmd {
    /// Command to run.
    pub cmd: io::Command,
    /// [`InMsg`] receiver.
    pub recv: mpsc::Receiver<InMsg>,
    /// [`OutMsg`] sender.
    pub send: mpsc::Sender<OutMsg>,
}
impl ChildCmd {
    /// Constructor.
    pub fn new(cmd: io::Command) -> (Self, ChildCmdCom) {
        let (in_send, in_recv) = mpsc::channel();
        let (out_send, out_recv) = mpsc::channel();
        (
            Self {
                cmd,
                recv: in_recv,
                send: out_send,
            },
            ChildCmdCom::new(in_send, out_recv),
        )
    }

    /// Spawns the child and changes its working directory.
    pub fn spawn(mut self) -> JoinHandle<Res<io::ExitStatus>> {
        thread::spawn(move || {
            self.run()
                .with_context(|| format!("child process for command `{:?}` failed", self.cmd))
        })
    }

    /// Runs the child.
    fn run(&mut self) -> Res<io::ExitStatus> {
        let mut child = self
            .cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| anyhow!("failed to start command {:?}", self.cmd,))?;
        let (stdout, stderr) = (
            mem::replace(&mut child.stdout, None)
                .ok_or_else(|| anyhow!(msg::child!("cannot access stdout")))?,
            mem::replace(&mut child.stderr, None)
                .ok_or_else(|| anyhow!(msg::child!("cannot access stderr")))?,
        );

        // Spawn thread that just reads `stdout`.
        let (stdout_send, stdout_recv) = mpsc::channel();
        Self::launch_kid(true, stdout_send, io::BufReader::new(stdout));
        let (stderr_send, stderr_recv) = mpsc::channel();
        Self::launch_kid(false, stderr_send, io::BufReader::new(stderr));

        let (mut stdout_recv, mut stderr_recv) = (Some(stdout_recv), Some(stderr_recv));

        // Receive message, try read line, send if any, repeat.
        loop {
            use mpsc::TryRecvError::*;

            // Message from master?
            match self.recv.try_recv() {
                // Nothing to do.
                Err(Empty) => (),
                // Kill order.
                Ok(InMsg::Kill) => {
                    log::debug!("{}", msg::child!("received `Kill` order"));
                    break;
                }
                // Connection to master is broken.
                Err(Disconnected) => {
                    log::debug!("{}", msg::child!("lost connection to master"));
                    break;
                }
            }

            // Check stdout.
            match stdout_recv.as_ref().map(mpsc::Receiver::try_recv) {
                // Stdout is done, unset `stdout_recv`.
                Some(Err(Disconnected)) | Some(Ok(OutMsg::Done)) => stdout_recv = None,
                // Message to pass.
                Some(Ok(msg)) => {
                    let res = self.send.send(msg);
                    if res.is_err() {
                        log::debug!("{}", msg::child!("lost connection with master"))
                    }
                }
                // Nothing to do.
                None => {
                    // log::trace!("  none");
                }
                Some(Err(Empty)) => {
                    // log::trace!("  empty");
                }
            }
            // Check stderr.
            match stderr_recv.as_ref().map(mpsc::Receiver::try_recv) {
                // Stdout is done, unset `stderr_recv`.
                Some(Err(Disconnected)) | Some(Ok(OutMsg::Done)) => stderr_recv = None,
                // Message to pass.
                Some(Ok(msg)) => {
                    let res = self.send.send(msg);
                    if res.is_err() {
                        log::debug!("{}", msg::child!("lost connection with master"))
                    }
                }
                // Nothing to do.
                None => {
                    // log::trace!("  none");
                }
                Some(Err(Empty)) => {
                    // log::trace!("  empty");
                }
            }

            if stdout_recv.is_none() && stderr_recv.is_none() {
                let res = self.send.send(OutMsg::done());
                if res.is_err() {
                    log::trace!("{}", msg::child!("lost connection with master"));
                }
                break;
            }
        }
        child.wait().with_context(|| "child processe panicked")
    }

    /// Launches `stdout` or `stderr` kid.
    fn launch_kid(
        is_stdout: bool,
        sender: mpsc::Sender<OutMsg>,
        br: impl io::BufRead + Send + 'static,
    ) {
        thread::spawn(move || {
            let desc = if is_stdout { "stdout" } else { "stderr" };
            for line in br.lines() {
                match line {
                    Ok(line) => {
                        let res = sender.send(OutMsg::msg(line, is_stdout));
                        if res.is_err() {
                            log::debug!("{}", msg::child!("error sending line from {}", desc));
                            // Connection ended.
                            break;
                        }
                    }
                    Err(e) => {
                        let e = Error::from(e).context("[child] failed to read a line");
                        let res = sender.send(OutMsg::fail(format!("{:?}", e)));
                        if res.is_err() {
                            log::debug!("{}", msg::child!("error sending read-line error"));
                        }
                        break;
                    }
                }
            }
            let res = sender.send(OutMsg::done());
            if res.is_err() {
                log::debug!(msg::child!("error sending final `Done` message"));
            }
        });
    }
}
