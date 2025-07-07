// WHY: Centralized abbreviation handling for sentence boundary detection
// Extracted from working test strategies to prevent false sentence splits

use std::collections::HashSet;

/// Title abbreviations that cause false sentence boundaries when followed by proper nouns
/// These are the first part of 2-segment identifiers like "Dr. Smith", "Mr. Johnson"
pub const TITLE_ABBREVIATIONS: &[&str] = &[
    "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr."
];

/// All abbreviations that should not cause sentence splits  
#[cfg(test)]
pub const ABBREVIATIONS: &[&str] = &[
    "Dr.", "Mr.", "Mrs.", "Ms.", "Prof.", "Sr.", "Jr.",
    "U.S.A.", "U.K.", "N.Y.C.", "L.A.", "D.C.",
    "ft.", "in.", "lbs.", "oz.", "mi.", "km.",
    "a.m.", "p.m.", "etc.", "vs.", "ea.", "deg.", "et al."
];

/// Efficient abbreviation lookup using HashSet for O(1) performance
pub struct AbbreviationChecker {
    #[cfg(test)]
    abbreviations: HashSet<&'static str>,
    title_abbreviations: HashSet<&'static str>,
}

impl AbbreviationChecker {
    /// Create new abbreviation checker with default abbreviation sets
    pub fn new() -> Self {
        Self {
            #[cfg(test)]
            abbreviations: ABBREVIATIONS.iter().copied().collect(),
            title_abbreviations: TITLE_ABBREVIATIONS.iter().copied().collect(),
        }
    }

    /// Check if a word is a known abbreviation
    #[cfg(test)]
    pub fn is_abbreviation(&self, word: &str) -> bool {
        self.abbreviations.contains(word)
    }

    /// Check if a word is a title abbreviation (causes false positives with proper nouns)
    pub fn is_title_abbreviation(&self, word: &str) -> bool {
        self.title_abbreviations.contains(word)
    }

    /// Check if text ends with an abbreviation that should not split sentences
    /// WHY: Examines the last word in context to determine if punctuation is part of abbreviation
    #[cfg(test)]
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
    use std::sync::OnceLock;
    
    // WHY: Single shared checker instance reduces test overhead
    static SHARED_CHECKER: OnceLock<AbbreviationChecker> = OnceLock::new();
    
    fn get_checker() -> &'static AbbreviationChecker {
        SHARED_CHECKER.get_or_init(|| AbbreviationChecker::new())
    }

    #[test]
    fn test_abbreviation_detection_comprehensive() {
        let checker = get_checker();
        
        // Test basic abbreviation detection
        let abbreviations = ["Dr.", "U.S.A.", "p.m.", "Prof.", "mi."];
        for abbr in &abbreviations {
            assert!(checker.is_abbreviation(abbr), "Should detect {} as abbreviation", abbr);
        }
        assert!(!checker.is_abbreviation("Hello"));
        
        // Test title vs non-title abbreviation classification
        let title_abbreviations = ["Dr.", "Prof.", "Mr.", "Mrs."];
        for abbr in &title_abbreviations {
            assert!(checker.is_title_abbreviation(abbr), "Should detect {} as title abbreviation", abbr);
        }
        assert!(!checker.is_title_abbreviation("U.S.A."));
        assert!(!checker.is_title_abbreviation("p.m."));
        
        // Test text ending detection
        let ending_tests = [
            ("Meeting at 5 p.m.", true),
            ("He lives in the U.S.A.", true),
            ("This is a sentence", false),
            ("Call Dr.", true),
            ("See Prof.", true),
        ];
        for (text, should_end_with_abbr) in &ending_tests {
            assert_eq!(checker.ends_with_abbreviation(text), *should_end_with_abbr, 
                "ends_with_abbreviation failed for: {}", text);
        }
        
        // Test punctuation handling with quotes (only quotes are stripped, not other punctuation)
        let punctuation_tests = [
            ("He said 'Dr.'", true),
            ("She mentioned \"Prof.\"", true),
            ("Meeting at 5 p.m.?", false), // Question mark is not stripped, so "p.m.?" != "p.m."
        ];
        for (text, should_detect) in &punctuation_tests {
            assert_eq!(checker.ends_with_abbreviation(text), *should_detect,
                "Punctuation handling failed for: {}", text);
        }
    }
}