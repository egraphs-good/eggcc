use std::{ffi::OsStr, fmt::Display, io, path::PathBuf};

use hashbrown::HashMap;
use petgraph::dot::Dot;

use crate::{
    cfg::{program_to_cfg, to_cfg},
    rvsdg::from_cfg::cfg_func_to_rvsdg,
    Optimizer,
};

pub(crate) struct ListDisplay<'a, TS>(pub TS, pub &'a str);

impl<'a, TS> Display for ListDisplay<'a, TS>
where
    TS: Clone + IntoIterator,
    TS::Item: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut did_something = false;
        for item in self.0.clone().into_iter() {
            if did_something {
                f.write_str(self.1)?;
            }
            Display::fmt(&item, f)?;
            did_something = true;
        }
        Ok(())
    }
}

/// Parse a string containing a bril program (in text format) into a Program.
///
/// This function is intended for use in tests and in ad-hoc debugging.
#[allow(unused)]
pub(crate) fn parse_from_string(input: &str) -> bril_rs::Program {
    use bril2json::parse_abstract_program_from_read;
    use bril_rs::load_program_from_read;
    let abs_program = parse_abstract_program_from_read(input.as_bytes(), true, false, None);
    let mut buf = Vec::new();
    serde_json::to_writer_pretty(&mut buf, &abs_program).unwrap();
    buf.push(b'\n');
    let json_str = String::from_utf8(buf).unwrap();
    load_program_from_read(json_str.as_bytes())
}

/// Visual representations of intermediate representations computed on a bril
/// program.
///
/// This struct is intended for use in debugging eggcc. SVG visualizations can
/// be opened directly in a web browser. Dot visualizations can be converted to
/// (e.g.) PNGs with utilities like graphviz, e.g.
/// `dot -Tpng input_cfg.dot -o input_cfg.png`.
#[allow(unused)]
pub struct DebugVisualizations {
    /// The raw CFG before any restructuring, in Dot format,
    /// for each function in the program.
    pub(crate) input_cfgs: HashMap<String, String>,
    /// The restructured CFG ready for RVSDG conversion, in Dot format.
    pub(crate) restructured_cfgs: HashMap<String, String>,
    /// The RVSDG, rendered in SVG format.
    pub(crate) rvsdg: String,
}

impl DebugVisualizations {
    /// Compute visualizations for the given bril program.
    ///
    /// This function is intended for use in debugging.
    ///
    /// # Panics
    /// Any failures in the conversion to CFG or RVSDG will cause a panic.
    pub fn new(input: &str) -> DebugVisualizations {
        let program = parse_from_string(input);
        let mut cfg = Optimizer::program_to_cfg(&program);

        let input_cfgs = cfg
            .functions
            .iter()
            .map(|f| (f.name.clone(), format!("{:#?}", Dot::new(&f.graph))))
            .collect();
        let restructured_cfgs = cfg
            .functions
            .iter()
            .map(|f| {
                let mut f = f.clone();
                f.restructure();
                (f.name.clone(), format!("{:#?}", Dot::new(&f.graph)))
            })
            .collect();

        let rvsdg = Optimizer::program_to_rvsdg(&program).unwrap().to_svg();
        DebugVisualizations {
            input_cfgs,
            restructured_cfgs,
            rvsdg,
        }
    }

    /// Write the visualizations to output files in the given directory.
    /// If the directory exists, it deletes it compeltely before writing.
    /// Otherwise, it creates the directory (recursively).
    ///
    /// * The `input_cfg` field is written to `input_cfg.dot`.
    /// * The `restructured_cfg` field is written to `function_name` + `restructured.dot`.
    /// * The `rvsdg` field is written to `rvsdg.svg`.
    ///
    /// Like other utilities related to `DebugVisualizations`, this method is
    /// only intended for debugging eggcc.
    pub fn write_output(&self, path: PathBuf) -> io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        // make the directory, clearing it if needed
        if path.exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }
        std::fs::create_dir_all(&path).unwrap();

        for (name, content) in
            self.input_cfgs
                .iter()
                .map(|(name, content)| (format!("{name}_cfg.dot"), content.clone()))
                .chain(self.restructured_cfgs.iter().map(|(name, content)| {
                    (format!("{name}_restructured_cfg.dot"), content.clone())
                }))
                .chain([("rvsdg.svg".to_string(), self.rvsdg.clone())])
        {
            let mut output_path = path.clone();
            assert!(path.is_dir());
            output_path.push(name.clone());
            let mut file = File::create(output_path)?;
            file.write_all(content.as_bytes())?;

            // if it's a dot file, also write
            // an svg using the dot program:
            if name.ends_with(".dot") {
                let output_path = path.join(name.replace(".dot", ".svg"));
                self.run(
                    "dot",
                    [
                        "-Tsvg",
                        "-o",
                        &output_path.into_os_string().into_string().unwrap(),
                    ],
                    &content,
                )?;
            }
        }
        Ok(())
    }

    /// Invokes some program with the given arguments, piping the given input to the program.
    /// Returns an error if the program returns a non-zero exit code.
    /// Code adapted from https://github.com/egraphs-good/egg/blob/e7845c5ae34267256b544c8e6b5bc36d91d096d2/src/dot.rs#L127
    pub fn run<S1, S2, I>(&self, program: S1, args: I, input: &str) -> std::io::Result<()>
    where
        S1: AsRef<OsStr>,
        S2: AsRef<OsStr>,
        I: IntoIterator<Item = S2>,
    {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        write!(stdin, "{}", input)?;
        match child.wait()?.code() {
            Some(0) => Ok(()),
            Some(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("dot program returned error code {}", e),
            )),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "dot program was killed by a signal",
            )),
        }
    }
}
