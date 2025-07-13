# Debug Investigation Notes

## Dialog Quote Transition Regression

**Issue**: Text `we read of an "Azacari (or Toucan) of Brazil; has his beak four inches long, almost two thick, like a Turk's sword" (A.D. 1656). From this description Tradescant knew the nature of the bird, if he had not seen it.` produces 1 sentence instead of expected 2.

**Finding**: Both old detector (dialog_detector.rs) and new detector (dialog_detector2.rs) produce 1 sentence for this test case.

**Next Steps**: Need to move backwards through git history to find when this behavior was correct. The baseline expectation of 2 sentences may come from an earlier version of the detector that handled this case differently.

**Key Pattern**: The critical seam is at `sword" (A.D. 1656)` where dialog should close and create sentence boundary.

**Investigation Command**: `git log --oneline src/sentence_detector/dialog_detector.rs` to find historical versions that handled this correctly.

**Unit Test that compares both with Debug state transitions**
cargo test test_dialog_quote_transition_comparison --features debug-states -- --nocapture 