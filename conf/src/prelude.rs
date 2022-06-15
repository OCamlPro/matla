//! This crate's prelude.

pub use base::*;
pub use serde::{Deserialize, Serialize};
pub use toml;

pub use crate::{
    customl,
    project::{self, Project},
    target::{self, Target},
    toolchain::{self, Toolchain},
    top_cla::{self, TopCla},
    Conf, TlcCla,
};

pub(crate) use crate::glob;

/// Imports this crate's prelude.
#[macro_export]
macro_rules! prelude {
    {$($stuff:tt)*} => {
        use $crate::prelude::{*, $($stuff)*};
    };
}

// /// Provides a static name.
// pub(crate) trait Named {
//     /// A static name.
//     fn name() -> &'static str;
// }
// macro_rules! impl_named {
//     ( $($ty:ident),* $(,)? ) => {
//         $(
//             impl Named for $ty {
//                 fn name() -> &'static str {
//                     stringify!($ty)
//                 }
//             }
//         )*
//     };
// }
// impl_named!(usize, bool, u64, );

// pub(crate) fn deserialize_pseudoption<'de, D, T>(deser: D) -> Result<Option<T>, D::Error>
// where
//     D: serde::Deserializer<'de>,
//     T: serde::Deserialize,
// {

//     match deser.deserialize_str() {
//         Ok("none" | "None" | "default" | "Default") => Ok(None),
//         Ok(unexpected) =>
//     }
//     match T::deserialize(deser) {
//         Ok(t) => Ok(Some(T)),
//         Err()
//     }
// }

// /// A fake option type that can be (de)serialized to/from either `"None"` or the value it contains.
// pub enum PseudOption<T> {
//     /// A value.
//     Some(T),
//     /// No value.
//     None,
// }

// impl<T> serde::Serialize for PseudOption<T>
// where
//     T: serde::Serialize,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         match self {
//             Self::Some(t) => serializer.serialize_newtype_variant("PseudOption", 0, "Some", t),
//             Self::None => serializer.serialize_str("none"),
//         }
//     }
// }
// // impl<T> serde::de::Visitor for PseudOption<T>
// // where T: serde::de::Visitor,
// // {

// //     fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
// //         write!(fmt, "either strings `\"none\"`, `\"None\"`, or ")?;

// //     }
// // }
