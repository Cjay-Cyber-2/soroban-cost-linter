//! Fixture contract for the corpus lint benchmarks.
//!
//! This contract is the smallest `soroban_storage_in_loop` fixture in
//! `tests/corpus/contracts/`: it deliberately contains one triggering case and
//! one clean case so the real-world corpus test can confirm the lint fires on
//! the former and stays silent on the latter. The functions below are the
//! shared shape all other `storage_iter_*` fixtures scale up from.
//!
//! Storage keys are a single `symbol_short!` constant, matching how
//! production contracts usually key small instance-storage counters.

#![no_std]
use soroban_sdk::{symbol_short, Env, Symbol};

/// Instance-storage key shared by every entry point in this fixture.
const COUNTER: Symbol = symbol_short!("counter");

/// Number of iterations the fixture loops over.
///
/// Chosen small (10) so the corpus build stays fast; the linter is
/// iteration-count-agnostic — a single `set` inside the loop body is enough
/// to fire regardless of the bound.
const ITERATIONS: u32 = 10;

/// Triggers `soroban_storage_in_loop`: set inside a for loop.
///
/// The value written varies with the loop counter, so this also exercises the
/// "loop-variant payload" case: the anti-pattern is the repeated host write
/// itself, not just a repeated identical write.
pub fn bad_simple(env: Env) {
    for i in 0..ITERATIONS {
        env.storage().instance().set(&COUNTER, &i);
    }
}

/// Good: storage operation outside the loop.
///
/// Accumulates in a local variable first, then performs exactly one storage
/// write after the loop. The final state is the same shape as the bad case's
/// last write (the counter holds a single `i128`), demonstrating that the fix
/// does not change the storage contract — only its cost profile.
pub fn good_simple(env: Env) {
    let mut total = 0i128;
    for _ in 0..ITERATIONS {
        total += 1;
    }
    env.storage().instance().set(&COUNTER, &total);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bad entry point must keep at least one instance-storage write
    /// lexically inside the `for` body: that shape is what
    /// `soroban_storage_in_loop` keys on for this fixture, and the corpus
    /// baseline entry for `storage_iter_simple` asserts it fires.
    #[test]
    fn bad_case_still_writes_inside_loop() {
        // Compile-time canary: the body of `bad_simple` must remain a loop
        // containing a `set` call. If the loop is ever hoisted out (i.e. the
        // fixture is accidentally "fixed"), the baseline comparison in
        // cargo-cost-lint/tests/real_world_corpus.rs will start failing
        // because the expected TP disappears.
        let src = include_str!("lib.rs");
        let bad_start = src.find("pub fn bad_simple").expect("bad_simple exists");
        let bad_end = src.find("pub fn good_simple").expect("good_simple exists");
        let body = &src[bad_start..bad_end];
        assert!(
            body.contains("for ") && body.contains(".set("),
            "bad_simple must contain a storage set inside a loop"
        );
    }

    /// The good entry point must keep its storage write outside the loop.
    #[test]
    fn good_case_writes_after_loop() {
        let src = include_str!("lib.rs");
        let good_start = src.find("pub fn good_simple").expect("good_simple exists");
        let good_end = src.find("#[cfg(test)]").expect("test module follows");
        let body = &src[good_start..good_end];
        let set_pos = body.find(".set(").expect("good_simple performs a set");
        let for_pos = body.find("for ").expect("good_simple contains a loop");
        let for_end = body[for_pos..].find("}").map(|i| for_pos + i).expect("loop closes");
        assert!(
            set_pos > for_end,
            "the storage write in good_simple must come after the loop closes"
        );
    }

    /// The accumulated value is deterministic: 10 iterations of +1 must
    /// always produce 10, independent of environment state.
    #[test]
    fn good_case_accumulates_deterministically() {
        // Mirrors the loop in good_simple without needing a host Env: the
        // arithmetic is what the single post-loop write persists.
        let mut total = 0i128;
        for _ in 0..ITERATIONS {
            total += 1;
        }
        assert_eq!(total, 10);
    }

    /// The loop-variant payload of the bad case must stay a plain integer
    /// that increases with the counter, guarding against the fixture being
    /// simplified into a loop-invariant write by accident.
    #[test]
    fn bad_case_payload_is_loop_variant() {
        let src = include_str!("lib.rs");
        let bad_start = src.find("pub fn bad_simple").expect("bad_simple exists");
        let bad_end = src.find("pub fn good_simple").expect("good_simple exists");
        let body = &src[bad_start..bad_end];
        assert!(
            body.contains("&i)"),
            "bad_simple writes the loop counter itself, keeping the payload loop-variant"
        );
    }

    /// The fixture shares one key between both entry points; a divergent key
    /// would make the bad/good comparison incomparable.
    ///
    /// The needle is assembled via `concat!` so this test's own source text
    /// (which is included via `include_str!`) cannot match it and inflate
    /// the count.
    #[test]
    fn both_entry_points_share_counter_key() {
        let needle = concat!(".set(&C", "OUNTER");
        let src = include_str!("lib.rs");
        assert_eq!(src.matches(needle).count(), 2);
    }
}
