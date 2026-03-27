use crate::io;
use alloc::string::String;

pub fn exit(_code: i32) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub fn abort() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub fn id() -> u32 {
    0
}

pub struct Command {
    _program: String,
}

impl Command {
    pub fn new<S: Into<String>>(program: S) -> Self {
        Self {
            _program: program.into(),
        }
    }

    pub fn arg<S: Into<String>>(&mut self, _arg: S) -> &mut Self {
        self
    }

    pub fn args<I, S>(&mut self, _args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self
    }

    pub fn env<K: Into<String>, V: Into<String>>(&mut self, _key: K, _val: V) -> &mut Self {
        self
    }

    pub fn spawn(&mut self) -> io::Result<Child> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "no child processes in kernel"))
    }

    pub fn output(&mut self) -> io::Result<Output> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "no child processes in kernel"))
    }

    pub fn status(&mut self) -> io::Result<ExitStatus> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "no child processes in kernel"))
    }
}

pub struct Child {
    _private: (),
}

pub struct Output {
    pub status: ExitStatus,
    pub stdout: alloc::vec::Vec<u8>,
    pub stderr: alloc::vec::Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitStatus {
    code: i32,
}

impl ExitStatus {
    pub fn success(&self) -> bool {
        self.code == 0
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.code)
    }
}

impl core::fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "exit status: {}", self.code)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCode(u8);

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode(0);
    pub const FAILURE: ExitCode = ExitCode(1);
}

impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        ExitCode(code)
    }
}

pub trait Termination {
    fn report(self) -> ExitCode;
}

impl Termination for () {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}

impl Termination for ExitCode {
    fn report(self) -> ExitCode {
        self
    }
}

impl<E: core::fmt::Debug> Termination for Result<(), E> {
    fn report(self) -> ExitCode {
        match self {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        }
    }
}
