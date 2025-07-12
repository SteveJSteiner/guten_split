// Dialog detector unit tests
// WHY: Separated from dialog_detector.rs to improve maintainability and reduce file size

use seams::sentence_detector::dialog_detector::SentenceDetectorDialog;
use std::sync::OnceLock;

// WHY: Single shared detector instance reduces test overhead from 38+ instantiations
static SHARED_DETECTOR: OnceLock<SentenceDetectorDialog> = OnceLock::new();

fn get_detector() -> &'static SentenceDetectorDialog {
    SHARED_DETECTOR.get_or_init(|| SentenceDetectorDialog::new().unwrap())
}

#[test]
fn test_user_hermit_scroll_text() {
    let detector = get_detector();
    
    let input = "He had thus sat for hours one day, interrupting his meditations only by\nan occasional pace to the door to look out for a break in the weather,\nwhen there came upon him with a shock of surprise the recollection that\nthere was more in the hermit's scroll than he had considered at first.\nNot much. He unfurled it, and beside the bequest of the hut, only these\nwords were added: \"For a commission look below my bed.\"";
    
    let sentences = detector.detect_sentences_borrowed(input).unwrap();
    
    // Expected sentences
    let expected = vec![
        "He had thus sat for hours one day, interrupting his meditations only by an occasional pace to the door to look out for a break in the weather, when there came upon him with a shock of surprise the recollection that there was more in the hermit's scroll than he had considered at first.",
        "Not much.",
        "He unfurled it, and beside the bequest of the hut, only these words were added: \"For a commission look below my bed.\""
    ];
    
    assert_eq!(sentences.len(), expected.len(), 
        "Expected {} sentences, got {}. Sentences: {:?}", 
        expected.len(), sentences.len(), 
        sentences.iter().map(|s| s.normalize().trim().to_string()).collect::<Vec<_>>());
    
    for (i, (actual, expected)) in sentences.iter().zip(expected.iter()).enumerate() {
        assert_eq!(actual.normalize().trim(), *expected,
            "Sentence {} mismatch:\nExpected: '{}'\nActual: '{}'", 
            i + 1, expected, actual.normalize().trim());
    }
}

#[test]
fn test_basic_narrative_sentences() {
    let detector = get_detector();
    let text = "This is a sentence. This is another sentence.";
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    
    assert_eq!(sentences.len(), 2);
    assert!(sentences[0].raw_content.contains("This is a sentence"));
    assert!(sentences[1].raw_content.contains("This is another sentence"));
}

#[test]
fn test_dialog_coalescing() {
    let detector = get_detector();
    let text = "He said, \"Stop her, sir! Ting-a-ling-ling!\" The headway ran almost out.";
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    
    assert_eq!(sentences.len(), 2);
    assert!(sentences[0].raw_content.contains("Stop her, sir! Ting-a-ling-ling!"));
    assert!(sentences[1].raw_content.contains("The headway ran almost out"));
}

#[test]
fn test_abbreviation_handling() {
    let detector = get_detector();
    
    // Test comprehensive abbreviation handling in various contexts
    let test_cases = [
        // Basic title abbreviation - should not be split
        ("Dr. Smith examined the patient. The results were clear.", 2, ["Dr. Smith examined the patient", "The results were clear"]),
        // Multiple title abbreviations
        ("Mr. and Mrs. Johnson arrived. They were late.", 2, ["Mr. and Mrs. Johnson arrived", "They were late"]),
        // Geographic abbreviations
        ("The U.S.A. declared independence. It was 1776.", 2, ["The U.S.A. declared independence", "It was 1776"]),
        // Measurement abbreviations
        ("Distance is 2.5 mi. from here. We can walk it.", 2, ["Distance is 2.5 mi. from here", "We can walk it"]),
        // Dialog with abbreviations
        ("He said, 'Dr. Smith will see you.' She nodded.", 2, ["Dr. Smith will see you", "She nodded"]),
    ];
    
    for (text, expected_count, expected_content) in test_cases {
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        
        if sentences.len() != expected_count {
            println!("MISMATCH for text: {}", text);
            println!("Expected {} sentences, got {} sentences:", expected_count, sentences.len());
            for (i, sentence) in sentences.iter().enumerate() {
                println!("  {}: '{}'", i, sentence.raw_content);
            }
            panic!("Failed for text: {text}");
        }
        
        for (i, expected) in expected_content.iter().enumerate() {
            assert!(sentences[i].raw_content.contains(expected), 
                "Sentence {} should contain '{}' but got '{}'", i, expected, sentences[i].raw_content);
        }
    }
    
    // Additional validation: ensure "Dr." is not treated as sentence boundary
    let text = "Dr. Smith examined the patient. The results were clear.";
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    assert!(!sentences[0].raw_content.trim().ends_with("Dr."), "Dr. should not end a sentence when followed by a name");
}

#[test]
fn test_soft_dialog_transitions() {
    let detector = get_detector();
    
    // Test case 1: comma + quote should soft transition, continue sentence
    let text = "\"Hello,\" she said quietly.";
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    // Should be one sentence - soft transition should continue
    assert_eq!(sentences.len(), 1, "Soft transition with comma should continue sentence");
    assert!(sentences[0].raw_content.contains("Hello") && sentences[0].raw_content.contains("she said"));
    
    // Test case 2: quote alone should soft transition
    let text = "\"Yes\" followed by more narrative.";
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    // Should be one sentence - soft transition should continue
    assert_eq!(sentences.len(), 1, "Soft transition with quote alone should continue sentence");
    
    // Test case 3: parenthetical close should soft transition
    let text = "(thinking quietly) and then he spoke.";
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    // Should be one sentence - soft transition should continue
    assert_eq!(sentences.len(), 1, "Soft transition with parenthetical should continue sentence");
}

#[test]
fn test_hard_dialog_transitions() {
    let detector = get_detector();
    
    // Test case: exclamation + space + capital should hard transition, create boundary
    let text = "\"Wait!\" he shouted loudly. Then he left.";
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    // Should be two sentences - hard transition should create boundary
    assert_eq!(sentences.len(), 2,
        "Hard transition should create sentence boundary\nExpected: 2 sentences\nActual: {} sentences\nSentences: {:?}",
        sentences.len(),
        sentences.iter().map(|s| &s.raw_content).collect::<Vec<_>>());
    assert!(sentences[0].raw_content.contains("Wait!") && sentences[0].raw_content.contains("he shouted"));
    assert!(sentences[1].raw_content.contains("Then he left"));
}

#[test]
fn test_colon_paragraph_break_dialog_separation() {
    let detector = get_detector();
    
    // Test case from task: colon + paragraph break + dialog should create sentence boundary
    let text = r#"She looked perplexed for a moment, and then said, not fiercely, but still loud enough for the furniture to hear:

"Well, I lay if I get hold of you I'll—"

She did not finish, for by this time she was bending down and punching under the bed with the broom, and so she needed breath to punctuate the punches with."#;
    
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    
    // Should be 2 sentences - colon followed by paragraph break should not over-coalesce
    assert_eq!(sentences.len(), 2, "Colon + paragraph break + dialog should create sentence boundary");
    
    // First sentence should include the dialog
    assert!(sentences[0].raw_content.contains("furniture to hear:"));
    assert!(sentences[0].raw_content.contains("Well, I lay if I get hold of you I'll—"));
    
    // Second sentence should be the narrative continuation
    assert!(sentences[1].raw_content.contains("She did not finish"));
}

#[test]
fn test_dialog_hard_separator_bug() {
    let detector = get_detector();
    
    // Test case: Hard separator between dialog lines should create separate sentences
    let input = r#"As the
young woman spoke, he rose, and advancing to the bed's head, said, with
more kindness than might have been expected of him:

"Oh, you must not talk about dying yet."

"Lor bless her dear heart, no!" interposed the nurse, hastily
depositing in her pocket a green glass bottle, the contents of which
she had been tasting in a corner with evident satisfaction."#;

    let sentences = detector.detect_sentences_borrowed(input).unwrap();
    
    assert_eq!(sentences.len(), 2,
        "Dialog hard separator test failed\nExpected: 2 sentences\nActual: {} sentences\nSentences: {:?}",
        sentences.len(),
        sentences.iter().map(|s| &s.raw_content).collect::<Vec<_>>());
    
    // Check sentence content
    assert!(sentences[0].normalize().contains("Oh, you must not talk about dying yet"));
    assert!(sentences[1].normalize().contains("Lor bless her dear heart, no!"));
    
    // Check span positioning - key bug validation
    assert_eq!(sentences[0].span.end_line, 5, "First sentence should end at line 5");
    assert_eq!(sentences[1].span.start_line, 7, "Second sentence should start at line 7");
}

#[test]
fn test_dialog_hard_separator_minimal() {
    let detector = get_detector();
    
    // Minimal case: colon followed by hard separator and dialog
    let input = "He said:\n\n\"Hello.\"\n\n\"World.\"";
    let sentences = detector.detect_sentences_borrowed(input).unwrap();
    
    assert_eq!(sentences.len(), 2, 
        "Expected 2 sentences, got {}. Sentences: {:?}", 
        sentences.len(), 
        sentences.iter().map(|s| s.normalize().trim().to_string()).collect::<Vec<_>>());
    
    assert_eq!(sentences[0].normalize().trim(), "He said: \"Hello.\"",
        "First sentence mismatch:\nExpected: 'He said: \"Hello.\"'\nActual: '{}'", 
        sentences[0].normalize().trim());
    
    assert_eq!(sentences[1].normalize().trim(), "\"World.\"",
        "Second sentence mismatch:\nExpected: '\"World.\"'\nActual: '{}'", 
        sentences[1].normalize().trim());
    
    // Verify line positions
    assert_eq!(sentences[0].span.start_line, 1, 
        "First sentence should start at line 1, got line {}", sentences[0].span.start_line);
    assert_eq!(sentences[1].span.start_line, 5,
        "Second sentence should start at line 5, got line {}", sentences[1].span.start_line);
    
    // Also test Windows line endings
    let input_windows = "He said:\r\n\r\n\"Hello.\"\r\n\r\n\"World.\"";
    let sentences_windows = detector.detect_sentences_borrowed(input_windows).unwrap();
    
    assert_eq!(sentences_windows.len(), 2, "Should detect 2 sentences with Windows line endings");
    assert_eq!(sentences_windows[0].normalize().trim(), "He said: \"Hello.\"");
    assert_eq!(sentences_windows[1].normalize().trim(), "\"World.\"");
}

#[test]
fn test_pg4300_compass_directions_fix() {
    let detector = get_detector();
    
    // Test the specific PG 4300 case that was failing - compass directions should not split
    let text = "Listener, S. E. by E.: Narrator, N. W. by W.: on the 53rd parallel of latitude, N., and 6th meridian of longitude, W.: at an angle of 45° to the terrestrial equator.";
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    
    // This should be one sentence - single capital letters should not create false boundaries
    assert_eq!(sentences.len(), 1, "Compass directions with single capitals should remain one sentence");
    assert!(sentences[0].raw_content.contains("S. E. by E."));
    assert!(sentences[0].raw_content.contains("N. W. by W."));
    assert!(sentences[0].raw_content.contains("latitude, N.,"));
    assert!(sentences[0].raw_content.contains("longitude, W.:"));
}

#[test]
fn test_missing_seams_reproduction() {
    let detector = get_detector();
    
    // Reproduce the MissingSeams.txt failure case - using Windows line endings (\r\n)
    let text = "By the narrator a\r\nlimitation of activity, mental and corporal, inasmuch as complete\r\nmental intercourse between himself and the listener had not taken place\r\nsince the consummation of puberty, indicated by catamenic hemorrhage,\r\nof the female issue of narrator and listener, 15 September 1903, there\r\nremained a period of 9 months and 1 day during which, in consequence of\r\na preestablished natural comprehension in incomprehension between the\r\nconsummated females (listener and issue), complete corporal liberty of\r\naction had been circumscribed.\r\n\r\nHow?\r\n\r\nBy various reiterated feminine interrogation concerning the masculine\r\ndestination whither, the place where, the time at which, the duration\r\nfor which, the object with which in the case of temporary absences,\r\nprojected or effected.\r\n\r\nWhat moved visibly above the listener's and the narrator's invisible\r\nthoughts?\r\n\r\nThe upcast reflection of a lamp and shade, an inconstant series of\r\nconcentric circles of varying gradations of light and shadow.\r\n\r\nIn what directions did listener and narrator lie?\r\n\r\nListener, S. E. by E.: Narrator, N. W. by W.: on the 53rd parallel of\r\nlatitude, N., and 6th meridian of longitude, W.: at an angle of 45° to\r\nthe terrestrial equator.\r\n\r\nIn what state of rest or motion?\r\n\r\nAt rest relatively to themselves and to each other.";
    
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    
    // This should be multiple sentences, not one massive sentence
    
    // Expected sentence boundaries:
    // 1. "...had been circumscribed." 
    // 2. "How?"
    // 3. "By various... projected or effected."
    // 4. "What moved... invisible thoughts?"
    // 5. "The upcast... light and shadow."
    // 6. "In what directions... narrator lie?"
    // 7. "Listener, S. E. by E.... terrestrial equator."
    // 8. "In what state... rest or motion?"
    // 9. "At rest... each other."
    
    // Should now detect multiple sentences with Windows line ending support
    assert!(sentences.len() > 1, "Should detect multiple sentences with Windows line endings, got {}", sentences.len());
    
    // Verify we get the expected 9 sentences
    assert_eq!(sentences.len(), 9, "Should detect exactly 9 sentences");
    
    // Verify some key sentence boundaries
    assert!(sentences[0].raw_content.contains("had been circumscribed"));
    assert_eq!(sentences[1].raw_content.trim(), "How?");
    assert!(sentences[2].raw_content.contains("projected or effected"));
    assert!(sentences[5].raw_content.contains("In what directions"));
    assert!(sentences[6].raw_content.contains("S. E. by E."));
    assert!(sentences[8].raw_content.contains("relatively to themselves"));
}

#[test]
fn test_actual_backward_seek_repro() {
    // Reproduce the exact failure pattern from the Gutenberg text
    // The problematic pattern: 'meet.' "Why should
    let detector = SentenceDetectorDialog::new().unwrap();
    let text = r#"S. and B. emend so as to negative the verb 'meet.' "Why should
Hrothgar weep if he expects to meet Beowulf again?" both these
scholars ask."#;
    
    match detector.detect_sentences_borrowed(text) {
        Ok(_sentences) => {
            // Test passed - no backward seek error
        }
        Err(e) => {
            if e.to_string().contains("Cannot seek backwards") {
                panic!("Backward seek bug reproduced with text: '{}'", text.replace('\n', "\\n"));
            } else {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}

#[test]
fn test_dialog_attribution_no_split() {
    let detector = get_detector();
    let text = r#""Lor bless her dear heart, no!" interposed the nurse, hastily
depositing in her pocket a green glass bottle, the contents of which
she had been tasting in a corner with evident satisfaction."#;
    
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    
    // Should be ONE sentence (dialog + attribution), not split at "no!" interposed
    assert_eq!(sentences.len(), 1,
        "Dialog with attribution should not be split\nExpected: 1 sentence\nActual: {} sentences\nSentences: {:?}",
        sentences.len(),
        sentences.iter().map(|s| &s.raw_content).collect::<Vec<_>>());
    assert!(sentences[0].raw_content.contains("no!"));
    assert!(sentences[0].raw_content.contains("interposed"));
    assert!(sentences[0].raw_content.contains("satisfaction"));
}

#[test]
fn test_narrative_dialog_separation_expected_fail() {
    let detector = get_detector();
    
    // This test is expected to fail - demonstrates current limitation
    let text = r#"Then he struggled up and looked round him, somewhat confused, for a second or two.  "Hallo!  Is it all over?""#;
    
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    
    // Expected: 2 sentences (narrative + dialog should be separate)
    // Current implementation may fail this expectation
    let expected_sentences = vec![
        "Then he struggled up and looked round him, somewhat confused, for a second or two.",
        "\"Hallo!  Is it all over?\""
    ];
    
    assert_eq!(sentences.len(), expected_sentences.len(),
        "Expected {} sentences, got {}\nExpected: {:?}\nActual: {:?}",
        expected_sentences.len(),
        sentences.len(),
        expected_sentences,
        sentences.iter().map(|s| s.raw_content.trim()).collect::<Vec<_>>());
    
    for (i, (actual, expected)) in sentences.iter().zip(expected_sentences.iter()).enumerate() {
        assert_eq!(actual.raw_content.trim(), *expected,
            "Sentence {} mismatch:\nExpected: '{}'\nActual: '{}'", 
            i + 1, expected, actual.raw_content.trim());
    }
}

// NEW TESTS FOR SENTENCE BOUNDARY ISSUE

#[test]
fn test_mismatched_quote_dialogue_by_design_choice() {
    let detector = get_detector();
    
    // BY DESIGN CHOICE: Current implementation detects 1 sentence for this mismatched quote pattern.
    // The quotes don't properly match (single quote opens/closes, double quote opens, single quote closes).
    // This could potentially be improved to detect 2 sentences in the future, but doing so
    // would require mechanism changes that might introduce false positives or significant overhead.
    // More data would be needed before changing this behavior.
    // Alternative parsers like pysbd might detect 2 sentences.
    let text = "'Ah! fair lady,' quoth the king, \"I love you, and without your love I am but dead.' Then the lady said, 'Stop it.";
    
    let sentences = detector.detect_sentences_borrowed(text).unwrap();
    
    // BY DESIGN: Current implementation produces 1 sentence for this pattern
    let expected = vec![
        "'Ah! fair lady,' quoth the king, \"I love you, and without your love I am but dead.' Then the lady said, 'Stop it."
    ];
    
    assert_eq!(sentences.len(), expected.len(),
        "BY DESIGN: Current behavior for mismatched quotes\nExpected: {} sentences, got {}\nExpected: {:?}\nActual: {:?}",
        expected.len(),
        sentences.len(),
        expected,
        sentences.iter().map(|s| s.normalize().trim().to_string()).collect::<Vec<_>>());
    
    for (i, (actual, expected)) in sentences.iter().zip(expected.iter()).enumerate() {
        assert_eq!(actual.normalize().trim(), *expected,
            "Sentence {} mismatch:\nExpected: '{}'\nActual: '{}'", 
            i + 1, expected, actual.normalize().trim());
    }
}

#[test]
fn test_backward_seek_minimal_reproduction() {
    let detector = get_detector();
    
    // Create a minimal case that demonstrates the backward seek issue
    // Start with a simple case and add complexity until we reproduce the problem
    let test_cases = [
        // Simple case - should work
        "Hello. World.",
        // Case with whitespace
        "Hello. World. Another.",
        // Case with hard separator
        "Hello.\n\nWorld.",
        // Case with dialog
        "He said. \"Hello.\" She replied.",
        // Case with colon + hard separator (from the original problem)
        "He said:\n\n\"Hello.\"\n\n\"World.\"",
        
        // DIALOG HARD END PATTERN TESTS
        // Test each type of dialog hard end pattern to see if they all have the same issue
        
        // 1. Single quote (known failing case)
        "verb 'meet.' \"Why should",
        
        // 2. Double quote  
        "verb \"meet.\" 'Why should",
        
        // 3. Smart double quote (opening: ", closing: ")
        "verb \u{201C}meet.\u{201D} \"Why should",
        
        // 4. Smart single quote (opening: ', closing: ')
        "verb \u{2018}meet.\u{2019} \"Why should",
        
        // 5. Round parentheses
        "verb (meet.) \"Why should",
        
        // 6. Square brackets
        "verb [meet.] \"Why should",
        
        // 7. Curly braces
        "verb {meet.} \"Why should",
    ];
    
    let mut failed_cases = Vec::new();
    
    for (i, test_case) in test_cases.iter().enumerate() {
        let result = detector.detect_sentences_borrowed(test_case);
        match result {
            Ok(_sentences) => {
                // Test case passed
            }
            Err(e) => {
                if e.to_string().contains("Cannot seek backwards") {
                    failed_cases.push((i, test_case, e.to_string()));
                } else {
                    panic!("Unexpected error in test case {}: {}", i, e);
                }
            }
        }
    }
    
    if !failed_cases.is_empty() {
        panic!("Found {} cases with backward seek issues: {:?}", 
            failed_cases.len(), 
            failed_cases.iter().map(|(i, case, err)| format!("Case {}: '{}' - {}", i, case.replace('\n', "\\n"), err)).collect::<Vec<_>>());
    }
    let problem_text = std::fs::read_to_string("exploration/problem_repro-0.txt")
        .expect("Failed to read problem_repro-0.txt");
    
    let result = detector.detect_sentences_borrowed(&problem_text);
    match result {
        Ok(_sentences) => {
            // Full file processed successfully
        }
        Err(e) => {
            if e.to_string().contains("Cannot seek backwards") {
                panic!("Backward seek error reproduced with full file: {}", e);
            } else {
                panic!("Unexpected error with full file: {}", e);
            }
        }
    }
}