//! Integration header parse stuff.

// prelude!();

use super::*;

/// Integration test configuration.
#[derive(Debug, Clone)]
pub struct Conf {
    /// Test configuration.
    pub test: TestConf,
    /// True for test-libraries.
    pub is_lib: bool,
}
implem! {
    for Conf {
        Deref<Target = TestConf> {
            |&self| &self.test,
            |&mut self| &mut self.test,
        }
    }
}
impl Default for Conf {
    fn default() -> Self {
        Self {
            test: TestConf::default(),
            is_lib: false,
        }
    }
}
impl Conf {
    /// Turns itself into an actual configuration.
    pub fn resolve(self) -> Either<TestConf, LibConf> {
        if self.is_lib {
            Right(LibConf)
        } else {
            Left(self.test)
        }
    }
}

pub fn parse(s: &str) -> PegRes<Either<TestConf, LibConf>> {
    let mut conf = Conf::default();
    conf_parser::test_header(s, &mut conf)?;
    Ok(conf.resolve())
}

peg::parser! {
    grammar conf_parser() for str {
        rule ws() = quiet! { [ ' ' | '\t' | '\n' ]+ }
        rule cmt() = quiet!{
            "#" (&[_] [^'\n'])* ("\n" / ![_])
            / "\\*" (&[_] [^'\n'])* ("\n" / ![_])
        }
        rule _() = quiet! { (ws() / cmt())* }
        rule bool() -> bool = quiet! {
            "true" { true } / "false" { false }
        } / expected!("boolean value")

        rule violation() -> Violation
        = ("Assumption" / "assumption") { Violation::Assumption }
        / ("Deadlock" / "deadlock") { Violation::Deadlock }
        / ("Safety" / "safety") { Violation::Safety }
        / ("Liveness" / "liveness") { Violation::Liveness }
        / ("Assert" / "assert") { Violation::Assert }

        rule failure() -> Failure
        = ("SpecEval" / "spec") { Failure::SpecEval }
        / ("SafetyEval" / "safety") { Failure::SafetyEval }
        / ("LivenessEval" / "liveness") { Failure::LivenessEval }

        rule error() -> ITestError
        = ("SpecParse" / "spec_parse") { ITestError::SpecParse }
        / ("ConfigParse" / "config_parse") { ITestError::ConfigParse }
        / ("StatespaceTooLarge" / "statespace_too_large") { ITestError::StatespaceTooLarge }
        / ("System" / "system") { ITestError::System }

        rule outcome() -> ITestOutcome
        = "expected" _ "=" _ res:(
            quiet! { "\"" / "'" }?
            res:(
                "success" { ITestOutcome::Success }
                / ("violation" / "Violation") _ "(" _ v:violation() _ ")" {
                    ITestOutcome::Violation(v)
                }
                / ("failure" / "Failure") _ "(" _ f:failure() _ ")" {
                    ITestOutcome::Failure(f)
                }
                / ("error" / "Error") _ "(" _ e:error() _ ")" {
                    ITestOutcome::Error(e)
                }
            )
            quiet! { "\"" / "'" }?
            { res }
        ) { res }

        rule conf_apply(conf: &mut Conf)
        = o:outcome() {?
            if conf.expected.is_some() {
                Err("trying to set expected outcome twice")
            } else {
                conf.expected = Some(o);
                Ok(())
            }
        }
        / "only_in" _ "=" _ only_in:(
            "release" { Some(true) }
            / "debug" { Some(false) }
            / "none" { None }
        ) {?
            if conf.only_in.is_some() {
                Err("trying to set `only_in` twice")
            } else {
                conf.only_in = only_in;
                Ok(())
            }
        }

        // pub rule test_conf(conf: &mut Conf)
        // = _ ("[" _ "test" _ "]" (_ conf_apply(conf))*)? _ "----" [_]*
        // pub rule lib_header()
        // = _ "[" _ "lib" _ "]" _ "----" [_]*

        pub rule test_header(conf: &mut Conf)
        = _ (
            ("[" _ "test" _ "]" ( _ conf_apply(conf))* ) { conf.is_lib = false }
            / ("[" _ "lib" _ "]") { conf.is_lib = true }
        )? _ "----" [_]*
    }
}
