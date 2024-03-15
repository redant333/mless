//! Hint generation.
mod pool_generator;
pub use pool_generator::HintPoolGenerator;

/// The trait that defines structs that can generate hints.
#[cfg_attr(test, mockall::automock)]
pub trait HintGenerator {
    /// Ask the generator to create the requested number of hints.
    ///
    /// Note that the generator is allowed to return a different
    /// number of hints than requested.
    fn create_hints(&self, hint_count: usize) -> Vec<String>;
}
