use rs_sft_sentences::SentenceDetectorDialog;

#[test]
fn test_dialog_hard_separator_bug_reproduction() {
    // Test case from exploration/BAD_SEP_HANDLING.md
    // Issue: Hard separator between dialog lines should create separate sentences
    // but dialog state machine incorrectly coalesces them
    
    let input = r#"As the
young woman spoke, he rose, and advancing to the bed's head, said, with
more kindness than might have been expected of him:

"Oh, you must not talk about dying yet."

"Lor bless her dear heart, no!" interposed the nurse, hastily
depositing in her pocket a green glass bottle, the contents of which
she had been tasting in a corner with evident satisfaction."#;

    let detector = SentenceDetectorDialog::new().expect("Failed to create detector");
    let sentences: Vec<_> = detector.detect_sentences_borrowed(input).expect("Failed to detect sentences");
    
    // Print actual results for debugging
    for (i, sentence) in sentences.iter().enumerate() {
        println!("{}\\t{}\\t{:?}", i, sentence.normalize(), sentence.span);
    }
    
    // Expected behavior based on BAD_SEP_HANDLING.md:
    // Sentence 28: "As the young woman spoke, he rose, and advancing to the bed's head, said, with more kindness than might have been expected of him: \"Oh, you must not talk about dying yet.\""
    // Sentence 29: "\"Lor bless her dear heart, no!\" interposed the nurse, hastily depositing in her pocket a green glass bottle, the contents of which she had been tasting in a corner with evident satisfaction."
    
    assert_eq!(sentences.len(), 2, "Should detect exactly 2 sentences");
    
    // Check sentence content (normalized)
    assert_eq!(
        sentences[0].normalize().trim(),
        "As the young woman spoke, he rose, and advancing to the bed's head, said, with more kindness than might have been expected of him: \"Oh, you must not talk about dying yet.\""
    );
    
    assert_eq!(
        sentences[1].normalize().trim(),
        "\"Lor bless her dear heart, no!\" interposed the nurse, hastily depositing in her pocket a green glass bottle, the contents of which she had been tasting in a corner with evident satisfaction."
    );
    
    // Check span positioning - this is the key bug
    // Expected: sentence 1 should start at line 7, column 0 (after the blank line)
    // Actual (buggy): sentence 1 starts at line 5, column 40 (continuing from previous)
    
    // First sentence should end before the hard separator
    assert_eq!(sentences[0].span.end_line, 5, "First sentence should end at line 5");
    
    // Second sentence should start after the hard separator on a new line
    assert_eq!(sentences[1].span.start_line, 7, "Second sentence should start at line 7");
    assert_eq!(sentences[1].span.start_col, 1, "Second sentence should start at column 1 (beginning of line)");
}

#[test]
fn test_dialog_hard_separator_minimal_case() {
    // Minimal reproduction case
    let input = "He said:\n\n\"Hello.\"\n\n\"World.\"";
    
    let detector = SentenceDetectorDialog::new().expect("Failed to create detector");
    let sentences: Vec<_> = detector.detect_sentences_borrowed(input).expect("Failed to detect sentences");
    
    // Print for debugging
    for (i, sentence) in sentences.iter().enumerate() {
        println!("{}\\t{}\\t{:?}", i, sentence.normalize(), sentence.span);
    }
    
    // Should be 3 sentences separated by hard separators
    assert_eq!(sentences.len(), 3, "Should detect 3 sentences");
    
    // Check positioning
    assert_eq!(sentences[0].normalize().trim(), "He said:");
    assert_eq!(sentences[1].normalize().trim(), "\"Hello.\"");
    assert_eq!(sentences[2].normalize().trim(), "\"World.\"");
    
    // Verify line positions
    assert_eq!(sentences[0].span.start_line, 1);
    assert_eq!(sentences[1].span.start_line, 3);
    assert_eq!(sentences[2].span.start_line, 5);
}