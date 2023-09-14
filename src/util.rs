use crate::peg::rvsdg_to_peg;
use crate::Optimizer;
use hashbrown::HashMap;
use petgraph::dot::Dot;
use std::{
    ffi::OsStr,
    fmt::{Display, Formatter},
    io,
    path::PathBuf,
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
    /// The PEG, rendered in DOT format.
    pub(crate) peg: String,
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
        let cfg = Optimizer::program_to_cfg(&program);
        let rvsdg = Optimizer::program_to_rvsdg(&program).unwrap();

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

        DebugVisualizations {
            input_cfgs,
            restructured_cfgs,
            rvsdg: rvsdg.to_svg(),
            peg: rvsdg_to_peg(&rvsdg).graph(),
        }
    }

    /// Write the visualizations to output files in the given directory.
    /// If the directory does not exist, it creates it.
    /// If the directory contains any files whose names conflict with the
    /// output files, it replaces them.
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

        // make the directory if it doesn't exist
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        for (name, content) in
            self.input_cfgs
                .iter()
                .map(|(name, content)| (format!("{name}_cfg.dot"), content.clone()))
                .chain(self.restructured_cfgs.iter().map(|(name, content)| {
                    (format!("{name}_restructured_cfg.dot"), content.clone())
                }))
                .chain([("rvsdg.svg".to_string(), self.rvsdg.clone())])
                .chain([("peg.dot".to_string(), self.peg.clone())])
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

#[derive(Clone, Copy)]
pub enum RunType {
    StructuredConversion,
    RvsdgConversion,
    NaiiveOptimization,
}

impl Display for RunType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RunType::StructuredConversion => write!(f, "structured"),
            RunType::RvsdgConversion => write!(f, "rvsdg"),
            RunType::NaiiveOptimization => write!(f, "naiive"),
        }
    }
}

impl RunType {
    pub fn produces_bril(&self) -> bool {
        match self {
            RunType::StructuredConversion => false,
            RunType::RvsdgConversion => false,
            RunType::NaiiveOptimization => true,
        }
    }
}

#[derive(Clone)]
pub struct Run {
    pub path: PathBuf,
    pub test_type: RunType,
    // Also interpret the resulting program
    pub interp: bool,
}

#[derive(Clone)]
pub struct RunOutput {
    // a visualization of the result
    pub visualization: String,
    // a viable file extension for the visualization
    pub visualization_file_extension: String,
    // if the result was interpreted, the stdout of interpreting it
    pub result_interpreted: Option<String>,
    pub original_interpreted: String,
}

impl Run {
    pub fn all_configurations_for(path: PathBuf) -> Vec<Run> {
        let mut res = vec![];
        for test_type in [
            RunType::StructuredConversion,
            //RunType::RvsdgConversion,
            RunType::NaiiveOptimization,
        ] {
            let default = Run {
                path: path.clone(),
                test_type,
                interp: false,
            };
            res.push(default.clone());
            if test_type.produces_bril() {
                let interp = Run {
                    interp: true,
                    ..default
                };
                res.push(interp);
            }
        }
        res
    }

    pub fn name(&self) -> String {
        let mut name = self.path.file_stem().unwrap().to_str().unwrap().to_string();
        name = format!("{}-{}", name, self.test_type);
        if self.interp {
            name = format!("{}-interp", name);
        }
        name
    }

    pub fn run(&self) -> RunOutput {
        let program_read = std::fs::read_to_string(self.path.clone()).unwrap();
        let args = Optimizer::parse_bril_args(&program_read);
        let original_interpreted = Optimizer::interp(&program_read, args.clone(), None);
        let (visualization, visualization_file_extension) = match self.test_type {
            RunType::StructuredConversion => {
                let structured = Optimizer::parse_to_structured(&program_read).unwrap();
                (structured.to_string(), ".txt")
            }
            RunType::RvsdgConversion => {
                let parsed = Optimizer::parse_bril(&program_read).unwrap();
                let rvsdg = Optimizer::program_to_rvsdg(&parsed).unwrap();
                let svg = rvsdg.to_svg();
                (svg, ".svg")
            }
            RunType::NaiiveOptimization => {
                let parsed = Optimizer::parse_bril(&program_read).unwrap();

                let mut optimizer = Optimizer::default();
                let res = optimizer.optimize(&parsed).unwrap();

                (format!("{}", res), ".bril")
            }
        };

        let result_interpreted = if self.interp {
            Some(Optimizer::interp(&program_read, args.clone(), None))
        } else {
            None
        };

        RunOutput {
            visualization,
            visualization_file_extension: visualization_file_extension.to_string(),
            result_interpreted,
            original_interpreted,
        }
    }
}
