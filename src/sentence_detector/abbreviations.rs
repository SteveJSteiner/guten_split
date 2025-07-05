// WHY: Centralized abbreviation handling for sentence boundary detection
// Extracted from working test strategies to prevent false sentence splits

use std::collections::HashSet;

/// Title abbreviations that cause false sentence boundaries when followed by proper nouns
/// These are the first part of 2-segment identifiers like "Dr. Smith", "Mr. Johnson"
pub const TITLE_ABBREVIATIONS: &[&str] = &[
    "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr."
];

/// All abbreviations that should not cause sentence splits
pub const ABBREVIATIONS: &[&str] = &[
    "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr.",
    "U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C.",
    "ft.", "in.", "lbs.", "oz.", "mi.", "km.",
    "a.m.", "p.m.", "etc.", "vs.", "ea.", "deg.", "et al."
];

/// Efficient abbreviation lookup using HashSet for O(1) performance
pub struct AbbreviationChecker {
    abbreviations: HashSet<&'static str>,
    title_abbreviations: HashSet<&'static str>,
}

impl AbbreviationChecker {
    /// Create new abbreviation checker with default abbreviation sets
    pub fn new() -> Self {
        Self {
            abbreviations: ABBREVIATIONS.iter().copied().collect(),
            title_abbreviations: TITLE_ABBREVIATIONS.iter().copied().collect(),
        }
    }

    /// Check if a word is a known abbreviation
    pub fn is_abbreviation(&self, word: &str) -> bool {
        self.abbreviations.contains(word)
    }

    /// Check if a word is a title abbreviation (causes false positives with proper nouns)
    pub fn is_title_abbreviation(&self, word: &str) -> bool {
        self.title_abbreviations.contains(word)
    }

    /// Check if text ends with an abbreviation that should not split sentences
    /// WHY: Examines the last word in context to determine if punctuation is part of abbreviation
    pub fn ends_with_abbreviation(&self, text: &str) -> bool {
        if let Some(last_word) = text.split_whitespace().last() {
            // Remove quotes and other punctuation to get clean word
            let clean_word = last_word.trim_matches(|c: char| {
                matches!(c, '"' | '\'' | '\u{201C}' | '\u{201D}' | '\u{2018}' | '\u{2019}')
            });
            self.is_abbreviation(clean_word)
        } else {
            false
        }
    }

    /// Check if text ends with a title abbreviation that could cause false positive
    /// WHY: Title abbreviations like "Dr." often precede proper nouns and should not split sentences
    pub fn ends_with_title_abbreviation(&self, text: &str) -> bool {
        if let Some(last_word) = text.split_whitespace().last() {
            // Remove quotes and other punctuation to get clean word
            let clean_word = last_word.trim_matches(|c: char| {
                matches!(c, '"' | '\'' | '\u{201C}' | '\u{201D}' | '\u{2018}' | '\u{2019}')
            });
            self.is_title_abbreviation(clean_word)
        } else {
            false
        }
    }
}

impl Default for AbbreviationChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abbreviation_detection() {
        let checker = AbbreviationChecker::new();
        
        // Test basic abbreviation detection
        assert!(checker.is_abbreviation("Dr."));
        assert!(checker.is_abbreviation("U.S.A."));
        assert!(checker.is_abbreviation("p.m."));
        assert!(!checker.is_abbreviation("Hello"));
        
        // Test title abbreviation detection
        assert!(checker.is_title_abbreviation("Dr."));
        assert!(checker.is_title_abbreviation("Prof."));
        assert!(!checker.is_title_abbreviation("U.S.A."));
    }

    #[test]
    fn test_text_ending_detection() {
        let checker = AbbreviationChecker::new();
        
        // Test ends_with_abbreviation
        assert!(checker.ends_with_abbreviation("Meeting at 5 p.m."));
        assert!(checker.ends_with_abbreviation("He lives in the U.S.A."));
        assert!(!checker.ends_with_abbreviation("This is a sentence"));
        
        // Test ends_with_title_abbreviation
        assert!(checker.ends_with_title_abbreviation("Call Dr."));
        assert!(checker.ends_with_title_abbreviation("See Prof."));
        assert!(!checker.ends_with_title_abbreviation("Meeting at 5 p.m."));
    }

    #[test]
    fn test_punctuation_handling() {
        let checker = AbbreviationChecker::new();
        
        // Test with quotes - should still detect abbreviation
        assert!(checker.ends_with_abbreviation("He said 'Dr.'"));
        assert!(checker.ends_with_abbreviation("She mentioned \"Prof.\""));
        assert!(checker.ends_with_title_abbreviation("He said 'Dr.'"));
    }
}