pub mod pinned;
pub use self::pinned::*;

/// Pin the provided value to its location. Panic if safety constraints are violated.
pub fn pinned<T>(t: T) -> Pinned<T> {
    self::pinned::Pinned::new(t)
}
