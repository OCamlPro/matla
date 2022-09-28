//! TLC warnings.

prelude!();

/// Enumerates all the warnings handled by matla.
#[derive(Debug, Clone)]
pub enum TlcWarning {
    /// A redefinition.
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

/// A redefinition for some `sym`bol at some `pos`ition for a `prev`iously defined symbol.
#[derive(Debug, Clone)]
pub struct Redef {
    /// Position of the redefinition.
    pub pos: source::FileSpan,
    /// Symbol that's being redefined.
    pub sym: String,
    /// Position of the original definition.
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
