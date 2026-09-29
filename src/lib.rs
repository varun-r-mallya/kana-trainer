//! kanatrain: a mistake-driven hiragana / katakana trainer.
//!
//! Learning loop:
//!   * cards you miss get heavier weights and are drawn more often (persisted between runs)
//!   * a missed card comes back a few questions later as a "second chance"
//!   * every miss shows a side-by-side comparison of what you typed vs the answer,
//!     look-alike kana, and (for combos) how the pieces blend
//!   * you must type the right answer once before moving on
//!   * the end-of-session report lists every confusion you made

pub mod data;
pub mod feedback;
pub mod rng;
pub mod session;
pub mod stats;
pub mod term;
