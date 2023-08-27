use std::{fmt::Display, io};

use petgraph::dot::Dot;

use crate::{cfg::to_cfg, rvsdg::from_cfg::to_rvsdg};

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
pub(crate) struct DebugVisualizations {
    /// The raw CFG before any restructuring, in Dot format.
    pub(crate) input_cfg: String,
    /// The restructured CFG ready for RVSDG conversion, in Dot format.
    pub(crate) restructured_cfg: String,
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
    #[allow(unused)]
    pub(crate) fn new(input: &str) -> DebugVisualizations {
        let program = parse_from_string(input);
        let mut cfg = to_cfg(&program.functions[0]);
        let input_cfg = format!("{:#?}", Dot::new(&cfg.graph));
        let restructured_cfg = {
            let mut cfg = cfg.clone();
            cfg.restructure();
            format!("{:#?}", Dot::new(&cfg.graph))
        };
        let rvsdg = to_rvsdg(&mut cfg).unwrap().to_svg();
        DebugVisualizations {
            input_cfg,
            restructured_cfg,
            rvsdg,
        }
    }

    /// Write the visualizations to output files with the given prefix.
    ///
    /// * The `input_cfg` field is written to `prefix` + `input_cfg.dot`.
    /// * The `restructured_cfg` field is written to `prefix` + `restructured.dot`.
    /// * The `rvsdg` field is written to `prefix` + `rvsdg.svg`.
    ///
    /// Like other utilities related to `DebugVisualizations`, this method is
    /// only intended for debugging eggcc.
    #[allow(unused)]
    pub(crate) fn write_output(&self, prefix: impl AsRef<str>) -> io::Result<()> {
        use std::fs::File;
        use std::io::Write;
        for (name, content) in &[
            ("input_cfg.dot", &self.input_cfg),
            ("restructured_cfg.dot", &self.restructured_cfg),
            ("rvsdg.svg", &self.rvsdg),
        ] {
            let mut file = File::create(format!("{}{}", prefix.as_ref(), name))?;
            file.write_all(content.as_bytes())?;
        }
        Ok(())
    }
}
