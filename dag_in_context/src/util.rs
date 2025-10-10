use std::{
    error::Error,
    ffi::OsStr,
    fmt,
    io::{Seek, SeekFrom, Write},
    process::{Command, ExitStatus, Stdio},
};

use tempfile::tempfile;

struct CommandFailure {
    status: ExitStatus,
    stderr: String,
}

impl CommandFailure {
    fn new(status: ExitStatus, stderr: String) -> Self {
        Self { status, stderr }
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

/// Invokes some program with the given arguments, piping the given input to the program.
/// Returns an error if the program returns a non-zero exit code.
/// Code adapted from https://github.com/egraphs-good/egg/blob/e7845c5ae34267256b544c8e6b5bc36d91d096d2/src/dot.rs#L127
pub fn run_cmd_line<S1, S2, I>(program: S1, args: I, input: &str) -> std::io::Result<String>
where
    S1: AsRef<OsStr>,
    S2: AsRef<OsStr>,
    I: IntoIterator<Item = S2>,
{
    // Write the input to a temporary file so the child can read it directly without
    // relying on manually managed filesystem paths.
    let mut temp_file = tempfile()?;
    temp_file.write_all(input.as_bytes())?;
    temp_file.seek(SeekFrom::Start(0))?;

    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::from(temp_file))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    match output.status.success() {
        true => Ok(String::from_utf8(output.stdout)
            .map_err(|e| std::io::Error::other(format!("utf8 error: {e}")))?),
        false => {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            Err(std::io::Error::other(CommandFailure::new(
                output.status,
                stderr,
            )))
        }
    }
}
