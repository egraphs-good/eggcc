use std::{
    ffi::OsStr,
    fs::{self, File},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

/// Invokes some program with the given arguments, piping the given input to the program.
/// Returns an error if the program returns a non-zero exit code.
/// Code adapted from https://github.com/egraphs-good/egg/blob/e7845c5ae34267256b544c8e6b5bc36d91d096d2/src/dot.rs#L127
pub fn run_cmd_line<S1, S2, I>(program: S1, args: I, input: &str) -> std::io::Result<String>
where
    S1: AsRef<OsStr>,
    S2: AsRef<OsStr>,
    I: IntoIterator<Item = S2>,
{
    // Write the input to a temporary file so the child can read it directly.
    let mut temp_path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    temp_path.push(format!(
        "eggcc-run-cmd-{}-{}.tmp",
        std::process::id(),
        unique
    ));

    fs::write(&temp_path, input)?;
    let input_file = File::open(&temp_path)?;

    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::from(input_file))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let output = match output {
        Ok(out) => {
            let _ = fs::remove_file(&temp_path);
            out
        }
        Err(err) => {
            let _ = fs::remove_file(&temp_path);
            return Err(err);
        }
    };

    match output.status.code() {
        Some(0) => Ok(String::from_utf8(output.stdout)
            .map_err(|e| std::io::Error::other(format!("utf8 error: {}", e)))?),
        Some(exit) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(std::io::Error::other(format!(
                "program returned error code {exit}; stderr: {stderr}"
            )))
        }
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(std::io::Error::other(format!(
                "program was terminated by signal; stderr: {stderr}"
            )))
        }
    }
}
