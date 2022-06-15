//! TLC warnings.

prelude!();

#[derive(Debug, Clone)]
pub enum TlcWarning {
    Redef(Redef),
}
impl TlcWarning {
    /// Pretty, multi-line representation.
    pub fn pretty(&self, project: &crate::FullProject, styles: &conf::Styles) -> Res<Vec<String>> {
        match self {
            Self::Redef(w) => w.pretty(project, styles),
        }
    }

    /// Static string description.
    pub fn desc(&self) -> &'static str {
        match self {
            Self::Redef(w) => w.desc(),
        }
    }
}
implem! {
    for TlcWarning {
        From<Redef> {
            |w| Self::Redef(w)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Redef {
    pub pos: source::FileSpan,
    pub sym: String,
    pub prev: source::FileSpan,
}

impl Redef {
    /// Static string description.
    pub fn desc(&self) -> &'static str {
        "warning"
    }

    /// Pretty, multi-line representation.
    pub fn pretty(&self, project: &crate::FullProject, styles: &conf::Styles) -> Res<Vec<String>> {
        let mut res = vec![];

        {
            let line = format!(
                "module {} ({})",
                styles.bold.paint(&self.pos.file),
                styles.uline.paint(self.pos.to_string()),
            );
            res.push(line);
        }

        {
            let line = format!(
                "{} or {} of symbol {}",
                styles.fatal.paint("Redefinition"),
                styles.fatal.paint("redeclaration"),
                styles.bold.paint(&self.sym),
            );
            res.push(line);
        }

        res.extend(self.pos.pretty_span(
            |module, buf| project.load_module(module, buf),
            Some(&styles.bad.paint("here").to_string()),
            Some(&format!("ending {}", styles.bad.paint("here"))),
        )?);

        res.extend(
            self.prev.pretty_span(
                |module, buf| project.load_module(module, buf),
                Some(
                    &styles
                        .bad
                        .paint("previous declaration/definition")
                        .to_string(),
                ),
                Some(&format!("ending {}", styles.bad.paint("here"))),
            )?,
        );

        Ok(res)
    }
}
