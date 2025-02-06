#[cfg(feature = "serialize")]
pub mod ser;

#[cfg(feature = "serialize")]
mod serializer;

#[cfg(feature = "serialize")]
pub use serializer::*;

/// Export dummy trait when `serialize` feature is not enabled
#[cfg(not(feature = "serialize"))]
pub trait ESTree {}
