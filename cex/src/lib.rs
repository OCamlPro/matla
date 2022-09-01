//! Counterexample management.

#![forbid(rustdoc::broken_intra_doc_links)]

/// This crate's prelude.
pub mod prelude {
    pub use base::*;
    pub use conf;

    pub use crate::{
        idx, pretty,
        value::{self, Value},
        Cex, State, StateInfo,
    };
}
/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
    {$($stuff:tt)*} => {
        use $crate::prelude::{*, $($stuff)*};
    };
}

pub mod idx {
    prelude!();

    safe_index::new! {
        /// Index for a state in a cex.
        State,
        /// Map from state indices to something.
        map: States,
    }
}

pub mod pretty;
pub mod value;

pub use value::Value;

prelude!();

/// Some state info.
#[derive(Debug, Clone)]
pub struct StateInfo {
    /// Action name.
    pub action: String,
    /// Action span in the source module.
    pub span: (source::Pos, source::Pos),
    /// Module the action is from.
    pub module: String,
}
impl StateInfo {
    /// Constructor.
    pub fn new(
        action: impl Into<String>,
        span: (source::Pos, source::Pos),
        module: impl Into<String>,
    ) -> Self {
        Self {
            action: action.into(),
            span,
            module: module.into(),
        }
    }
}

/// A state in a counterexample.
#[derive(Debug, Clone)]
pub struct State {
    /// State info, none if the state is initial.
    pub info: Option<StateInfo>,
    /// Maps state variables to values.
    pub values: Map<String, Value>,
}
implem! {
    for State {
        Deref<Target = Map<String, Value>> {
            |&self| &self.values,
            |&mut self| &mut self.values,
        }
    }
}
impl State {
    /// Empty constructor.
    pub fn new(info: Option<StateInfo>) -> Self {
        Self {
            info,
            values: Map::new(),
        }
    }
}

/// Shape of a counterexample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Finite trace.
    ///
    /// For invariant (safety property) counterexamples.
    Finite,
    /// Counterexample stutters on its final state.
    ///
    /// For property (liveness property) counterexamples.
    Stuttering,
    /// Counterexample loops back to this state after its final states.
    ///
    /// For property (liveness property) counterexamples.
    Loop(idx::State),
}

/// A counterexample.
#[derive(Debug, Clone)]
pub struct Cex {
    pub falsified: Option<String>,
    pub states: idx::States<State>,
    pub shape: Shape,
}
impl Cex {
    /// Constructor with default [`Shape::Finite`] shape.
    pub fn new() -> Self {
        Self {
            falsified: None,
            states: idx::States::new(),
            shape: Shape::Finite,
        }
    }

    /// Optional name of the invariant/property falsified, and a flag indicating it's a temporal
    /// property.
    pub fn falsified(&self) -> (Option<&str>, bool) {
        (
            self.falsified.as_ref().map(|s| s as &str),
            self.shape != Shape::Finite,
        )
    }

    /// Sets the `falsified` field.
    pub fn set_falsified(mut self, falsified: impl Into<String>) -> Res<Self> {
        let _prev = mem::replace(&mut self.falsified, Some(falsified.into()));
        if let Some(_prev) = _prev {
            bail!(
                "trying to set `falsified` twice (`{}`, `{}`)",
                _prev,
                self.falsified.as_ref().unwrap()
            )
        }
        Ok(self)
    }

    /// Yields the [`idx::State`] corresponding to `usize`-index.
    ///
    /// Fails if the input index is illegal.
    pub fn idx_of(&self, i: usize) -> Res<idx::State> {
        if i < self.states.len() {
            Ok(i.into())
        } else {
            Err(anyhow!(
                "illegal state index `{}`, CEX only contains `{}` state(s)",
                i,
                self.states.len()
            ))
        }
    }

    /// Sets the shape of a cex.
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }
}
implem! {
    for Cex {
        Deref<Target = idx::States<State>> {
            |&self| &self.states,
            |&mut self| &mut self.states,
        }
    }
}
