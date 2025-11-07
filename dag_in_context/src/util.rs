use std::{
    error::Error,
    ffi::OsStr,
    fmt,
    io::{self, Read, Seek, SeekFrom, Write},
    process::{Command, ExitStatus, Stdio},
};

#[cfg(not(unix))]
use std::io::ErrorKind;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

use tempfile::tempfile;

pub struct CommandFailure {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl CommandFailure {
    fn new(status: ExitStatus, stdout: String, stderr: String) -> Self {
        Self {
            status,
            stdout,
            stderr,
        }
    }

    fn exit_code(&self) -> Option<i32> {
        self.status.code()
    }
}

impl fmt::Display for CommandFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.exit_code() {
            Some(code) => writeln!(f, "program returned error code {code}")?,
            None => writeln!(f, "program was terminated by signal")?,
        };

        if self.stdout.is_empty() {
            writeln!(f, "stdout: <empty>")?
        } else {
            writeln!(f, "stdout:\n{}", self.stdout)?;
        }

        if self.stderr.is_empty() {
            write!(f, "stderr: <empty>")
        } else {
            write!(f, "stderr:\n{}", self.stderr)
        }
    }
}

impl fmt::Debug for CommandFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for CommandFailure {}

#[derive(Debug)]
pub struct MemoryLimitExceeded {
    pub limit_bytes: u64,
    pub signal: Option<i32>,
}

impl MemoryLimitExceeded {
    pub fn new(limit_bytes: u64, signal: Option<i32>) -> Self {
        Self {
            limit_bytes,
            signal,
        }
    }

    pub fn signal(&self) -> Option<i32> {
        self.signal
    }
}

impl fmt::Display for MemoryLimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "memory limit of {} bytes exceeded", self.limit_bytes)
    }
}

impl Error for MemoryLimitExceeded {}

/// Invokes some program with the given arguments, piping the given input to the program.
/// Returns an error if the program returns a non-zero exit code.
/// Code adapted from https://github.com/egraphs-good/egg/blob/e7845c5ae34267256b544c8e6b5bc36d91d096d2/src/dot.rs#L127
pub fn run_cmd_line<S1, S2, I>(program: S1, args: I, input: &str) -> std::io::Result<String>
where
    S1: AsRef<OsStr>,
    S2: AsRef<OsStr>,
    I: IntoIterator<Item = S2>,
{
    run_cmd_line_with_memory_limit(program, args, input, None)
}

/// Like [`run_cmd_line`], but enforces a maximum address space size when running on Unix.
pub fn run_cmd_line_with_memory_limit<S1, S2, I>(
    program: S1,
    args: I,
    input: &str,
    memory_limit_bytes: Option<u64>,
) -> std::io::Result<String>
where
    S1: AsRef<OsStr>,
    S2: AsRef<OsStr>,
    I: IntoIterator<Item = S2>,
{
    #[cfg(not(unix))]
    if memory_limit_bytes.is_some() {
        return Err(std::io::Error::new(
            ErrorKind::Unsupported,
            "memory limits are only supported on Unix targets",
        ));
    }

    // Write the input to a temporary file so the child can read it directly without
    // relying on manually managed filesystem paths.
    let mut temp_file = tempfile()?;
    temp_file.write_all(input.as_bytes())?;
    temp_file.seek(SeekFrom::Start(0))?;

    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::from(temp_file))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    if let Some(limit) = memory_limit_bytes {
        let limit = limit as libc::rlim_t;
        unsafe {
            command.pre_exec(move || {
                let target = libc::rlimit {
                    rlim_cur: limit,
                    rlim_max: limit,
                };
                if libc::setrlimit(libc::RLIMIT_AS, &target) != 0 {
                    return Err(io::Error::last_os_error());
                }
                // Attempt to set RLIMIT_DATA as well; ignore failures since not all platforms allow it.
                let _ = libc::setrlimit(libc::RLIMIT_DATA, &target);
                Ok(())
            });
        }
    }

    let mut child = command.spawn()?;
    let stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("failed to capture stdout"))?;
    let stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| std::io::Error::other("failed to capture stderr"))?;

    let stdout_handle = std::thread::spawn(move || -> std::io::Result<String> {
        let mut buf = Vec::new();
        let mut reader = io::BufReader::new(stdout_pipe);
        reader.read_to_end(&mut buf)?;
        String::from_utf8(buf)
            .map_err(|e| std::io::Error::other(format!("utf8 error: {e}")))
    });

    let stderr_handle = std::thread::spawn(move || -> std::io::Result<String> {
        let mut buf = Vec::new();
        let mut reader = io::BufReader::new(stderr_pipe);
        let mut chunk = [0u8; 4096];
        let mut stderr_writer = io::stderr();
        loop {
            let n = reader.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            stderr_writer.write_all(&chunk[..n])?;
            stderr_writer.flush()?;
        }
        Ok(String::from_utf8_lossy(&buf).into_owned())
    });

    let status = child.wait()?;

    let stdout = stdout_handle
        .join()
        .map_err(|_| std::io::Error::other("failed to join stdout reader"))??;
    let stderr = stderr_handle
        .join()
        .map_err(|_| std::io::Error::other("failed to join stderr reader"))??;

    #[cfg(unix)]
    if let Some(limit_bytes) = memory_limit_bytes {
        if let Some(signal) = status.signal() {
            if matches!(signal, libc::SIGKILL | libc::SIGABRT) {
                return Err(std::io::Error::other(MemoryLimitExceeded::new(
                    limit_bytes,
                    Some(signal),
                )));
            }
        }
    }

    if status.success() {
        Ok(stdout)
    } else {
        Err(std::io::Error::other(CommandFailure::new(status, stdout, stderr)))
    }
}
