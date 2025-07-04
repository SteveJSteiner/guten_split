use regex_automata::{meta::Regex, Input};
use std::collections::HashMap;

// Type-safe position wrappers to prevent byte/char and 0/1-based confusion

/// 0-based byte position in source text
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct BytePos(pub usize);

/// 0-based character position in source text
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct CharPos(pub usize);

/// 1-based line number for output spans
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct OneBasedLine(pub usize);

/// 1-based column number for output spans
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct OneBasedCol(pub usize);

// Conversion implementations

impl From<BytePos> for usize {
    fn from(pos: BytePos) -> Self {
        pos.0
    }
}

impl From<CharPos> for usize {
    fn from(pos: CharPos) -> Self {
        pos.0
    }
}

impl From<OneBasedLine> for usize {
    fn from(line: OneBasedLine) -> Self {
        line.0
    }
}

impl From<OneBasedCol> for usize {
    fn from(col: OneBasedCol) -> Self {
        col.0
    }
}

impl BytePos {
    pub fn new(pos: usize) -> Self {
        BytePos(pos)
    }
    
    pub fn advance(&self, offset: usize) -> Self {
        BytePos(self.0 + offset)
    }
}

impl CharPos {
    pub fn new(pos: usize) -> Self {
        CharPos(pos)
    }
}

impl OneBasedLine {
    pub fn new(line: usize) -> Option<Self> {
        if line > 0 {
            Some(OneBasedLine(line))
        } else {
            None
        }
    }
    
    pub fn first() -> Self {
        OneBasedLine(1)
    }
}

impl OneBasedCol {
    pub fn new(col: usize) -> Option<Self> {
        if col > 0 {
            Some(OneBasedCol(col))
        } else {
            None
        }
    }
    
    pub fn first() -> Self {
        OneBasedCol(1)
    }
}

/// Convert byte position to character position in given text
pub fn byte_to_char_pos(text: &str, byte_pos: BytePos) -> Result<CharPos, String> {
    if byte_pos.0 > text.len() {
        return Err(format!("Byte position {} exceeds text length {}", byte_pos.0, text.len()));
    }
    
    // Count characters up to byte position
    let char_count = text[..byte_pos.0].chars().count();
    Ok(CharPos::new(char_count))
}

/// Convert character position to line/column in given text
pub fn char_to_line_col(text: &str, char_pos: CharPos) -> Result<(OneBasedLine, OneBasedCol), String> {
    let mut line = 1;
    let mut col = 1;
    let mut char_count = 0;
    
    for ch in text.chars() {
        if char_count == char_pos.0 {
            break;
        }
        
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        char_count += 1;
    }
    
    if char_count != char_pos.0 && char_pos.0 != text.chars().count() {
        return Err(format!("Character position {} exceeds text length", char_pos.0));
    }
    
    Ok((
        OneBasedLine::new(line).unwrap(),
        OneBasedCol::new(col).unwrap(),
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DialogState {
    Narrative,
    DialogDoubleQuote,
    DialogSingleQuote,
    DialogSmartDoubleOpen,
    DialogSmartSingleOpen,
    DialogParenthheticalRound,
    DialogParenthheticalSquare,
    DialogParenthheticalCurly,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DetectedSentence {
    pub start_pos: CharPos,
    pub end_pos: CharPos,
    pub content: String,
    pub start_line: OneBasedLine,
    pub start_col: OneBasedCol,
    pub end_line: OneBasedLine,
    pub end_col: OneBasedCol,
}

#[derive(Debug, Clone)]
pub enum MatchType {
    NarrativeGestureBoundary,
    DialogOpen,
    DialogEnd,
    HardSeparator,
}

pub struct DialogStateMachine {
    patterns: HashMap<DialogState, Regex>,
    quote_starts: Regex,
    paren_starts: Regex,
}

impl DialogStateMachine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut patterns = HashMap::new();
        
        // Compositional pattern components
        let sentence_end_punct = r"[.!?]";
        let soft_separator = r"[ \t]+";  // spaces and tabs only
        let hard_separator = r"\n\n";   // double newline
        let sentence_start_chars = r"[A-Z\x22\x27\u{201C}\u{2018}\(\[\{]";
        let dialog_open_chars = r"[\x22\x27\u{201C}\u{2018}\(\[\{]";
        
        // Composed patterns with visible logic
        let narrative_soft_boundary = format!("{}{}{}", sentence_end_punct, soft_separator, sentence_start_chars);
        let narrative_hard_boundary = format!(r"{}\s*{}\s*{}", sentence_end_punct, hard_separator, sentence_start_chars);
        let pure_hard_sep = hard_separator.to_string();  // standalone hard separator (will need context check)
        
        // Dialog closing characters
        let double_quote_close = r"\x22";      // "
        let single_quote_close = r"\x27";      // '
        let smart_double_close = r"\u{201D}";  // "
        let smart_single_close = r"\u{2019}";  // '
        let round_paren_close = r"\)";         // )
        let square_bracket_close = r"\]";      // ]
        let curly_brace_close = r"\}";         // }
        
        // Composed dialog ending patterns: PUNCT + CLOSE + SEP + START
        let dialog_double_end = format!("{}{}{}{}", sentence_end_punct, double_quote_close, soft_separator, sentence_start_chars);
        let dialog_single_end = format!("{}{}{}{}", sentence_end_punct, single_quote_close, soft_separator, sentence_start_chars);
        let dialog_smart_double_end = format!("{}{}{}{}", sentence_end_punct, smart_double_close, soft_separator, sentence_start_chars);
        let dialog_smart_single_end = format!("{}{}{}{}", sentence_end_punct, smart_single_close, soft_separator, sentence_start_chars);
        let dialog_paren_round_end = format!("{}{}{}{}", sentence_end_punct, round_paren_close, soft_separator, sentence_start_chars);
        let dialog_paren_square_end = format!("{}{}{}{}", sentence_end_punct, square_bracket_close, soft_separator, sentence_start_chars);
        let dialog_paren_curly_end = format!("{}{}{}{}", sentence_end_punct, curly_brace_close, soft_separator, sentence_start_chars);
        
        // Build state-specific patterns with visible composition
        let narrative_pattern = format!(
            "(?:{})|(?:{})|(?:{})|(?:{})",
            narrative_soft_boundary, narrative_hard_boundary, pure_hard_sep, dialog_open_chars
        );
        
        let dialog_double_pattern = format!(
            "(?:{})|(?:{})",
            dialog_double_end, pure_hard_sep
        );
        
        let dialog_single_pattern = format!(
            "(?:{})|(?:{})",
            dialog_single_end, pure_hard_sep
        );
        
        let dialog_smart_double_pattern = format!(
            "(?:{})|(?:{})",
            dialog_smart_double_end, pure_hard_sep
        );
        
        let dialog_smart_single_pattern = format!(
            "(?:{})|(?:{})",
            dialog_smart_single_end, pure_hard_sep
        );
        
        let dialog_paren_round_pattern = format!(
            "(?:{})|(?:{})",
            dialog_paren_round_end, pure_hard_sep
        );
        
        let dialog_paren_square_pattern = format!(
            "(?:{})|(?:{})",
            dialog_paren_square_end, pure_hard_sep
        );
        
        let dialog_paren_curly_pattern = format!(
            "(?:{})|(?:{})",
            dialog_paren_curly_end, pure_hard_sep
        );
        
        // Compile patterns
        patterns.insert(DialogState::Narrative, Regex::new(&narrative_pattern)?);
        patterns.insert(DialogState::DialogDoubleQuote, Regex::new(&dialog_double_pattern)?);
        patterns.insert(DialogState::DialogSingleQuote, Regex::new(&dialog_single_pattern)?);
        patterns.insert(DialogState::DialogSmartDoubleOpen, Regex::new(&dialog_smart_double_pattern)?);
        patterns.insert(DialogState::DialogSmartSingleOpen, Regex::new(&dialog_smart_single_pattern)?);
        patterns.insert(DialogState::DialogParenthheticalRound, Regex::new(&dialog_paren_round_pattern)?);
        patterns.insert(DialogState::DialogParenthheticalSquare, Regex::new(&dialog_paren_square_pattern)?);
        patterns.insert(DialogState::DialogParenthheticalCurly, Regex::new(&dialog_paren_curly_pattern)?);
        
        // Helper patterns for state transitions
        let quote_starts = Regex::new(r"[\x22\x27\u{201C}\u{2018}]")?;
        let paren_starts = Regex::new(r"[\(\[\{]")?;
        
        Ok(DialogStateMachine {
            patterns,
            quote_starts,
            paren_starts,
        })
    }
    
    pub fn detect_sentences(&self, text: &str) -> Result<Vec<DetectedSentence>, String> {
        let mut sentences = Vec::new();
        let mut current_state = DialogState::Narrative;
        let mut sentence_start_byte = BytePos::new(0);
        let mut position_byte = BytePos::new(0);
        
        while position_byte.0 < text.len() {
            let pattern = match self.patterns.get(&current_state) {
                Some(p) => p,
                None => {
                    // Fallback to narrative pattern for unknown states
                    self.patterns.get(&DialogState::Narrative).unwrap()
                }
            };
            
            let input = Input::new(&text[position_byte.0..]);
            
            if let Some(mat) = pattern.find(input) {
                let match_start_byte = position_byte.advance(mat.start());
                let match_end_byte = position_byte.advance(mat.end());
                
                // Determine what type of match this is and next state
                let matched_text = &text[match_start_byte.0..match_end_byte.0];
                let (match_type, next_state) = self.classify_match(matched_text, &current_state);
                
                match match_type {
                    MatchType::NarrativeGestureBoundary => {
                        // This creates a sentence boundary - record the sentence
                        let sentence_end_byte = self.find_sent_sep_start(matched_text)
                            .map(|sep_offset| match_start_byte.advance(sep_offset))
                            .unwrap_or(match_start_byte);
                        
                        if sentence_end_byte.0 > sentence_start_byte.0 {
                            let content = text[sentence_start_byte.0..sentence_end_byte.0].trim().to_string();
                            if !content.is_empty() {
                                // Convert byte positions to character positions
                                let start_char = byte_to_char_pos(text, sentence_start_byte)?;
                                let end_char = byte_to_char_pos(text, sentence_end_byte)?;
                                
                                // Convert character positions to line/col
                                let (start_line, start_col) = char_to_line_col(text, start_char)?;
                                let (end_line, end_col) = char_to_line_col(text, end_char)?;
                                
                                sentences.push(DetectedSentence {
                                    start_pos: start_char,
                                    end_pos: end_char,
                                    content,
                                    start_line,
                                    start_col,
                                    end_line,
                                    end_col,
                                });
                            }
                        }
                        
                        // Next sentence starts after the separator
                        let next_sentence_start_byte = self.find_sent_sep_end(matched_text)
                            .map(|sep_end_offset| match_start_byte.advance(sep_end_offset))
                            .unwrap_or(match_end_byte);
                        
                        sentence_start_byte = next_sentence_start_byte;
                    }
                    MatchType::DialogOpen => {
                        // State transition only - no sentence boundary created
                        // Continue from match start (include the opening punctuation in dialog)
                        // No sentence recorded, just state change
                    }
                    MatchType::DialogEnd => {
                        // Dialog end creates a sentence boundary
                        let sentence_end_byte = self.find_sent_sep_start(matched_text)
                            .map(|sep_offset| match_start_byte.advance(sep_offset))
                            .unwrap_or(match_start_byte);
                        
                        if sentence_end_byte.0 > sentence_start_byte.0 {
                            let content = text[sentence_start_byte.0..sentence_end_byte.0].trim().to_string();
                            if !content.is_empty() {
                                let start_char = byte_to_char_pos(text, sentence_start_byte)?;
                                let end_char = byte_to_char_pos(text, sentence_end_byte)?;
                                
                                let (start_line, start_col) = char_to_line_col(text, start_char)?;
                                let (end_line, end_col) = char_to_line_col(text, end_char)?;
                                
                                sentences.push(DetectedSentence {
                                    start_pos: start_char,
                                    end_pos: end_char,
                                    content,
                                    start_line,
                                    start_col,
                                    end_line,
                                    end_col,
                                });
                            }
                        }
                        
                        let next_sentence_start_byte = self.find_sent_sep_end(matched_text)
                            .map(|sep_end_offset| match_start_byte.advance(sep_end_offset))
                            .unwrap_or(match_end_byte);
                        
                        sentence_start_byte = next_sentence_start_byte;
                    }
                    MatchType::HardSeparator => {
                        // Hard separator - record sentence and transition to Unknown
                        if sentence_start_byte.0 < match_start_byte.0 {
                            // For hard separators, preserve punctuation but trim trailing whitespace carefully
                            let raw_content = &text[sentence_start_byte.0..match_start_byte.0];
                            let content = raw_content.trim_start().trim_end_matches(char::is_whitespace).to_string();
                            if !content.is_empty() {
                                let start_char = byte_to_char_pos(text, sentence_start_byte)?;
                                let end_char = byte_to_char_pos(text, match_start_byte)?;
                                
                                let (start_line, start_col) = char_to_line_col(text, start_char)?;
                                let (end_line, end_col) = char_to_line_col(text, end_char)?;
                                
                                sentences.push(DetectedSentence {
                                    start_pos: start_char,
                                    end_pos: end_char,
                                    content,
                                    start_line,
                                    start_col,
                                    end_line,
                                    end_col,
                                });
                            }
                        }
                        
                        sentence_start_byte = match_end_byte;
                    }
                }
                
                // Update position and state
                position_byte = match_end_byte;
                current_state = next_state;
            } else {
                // No more boundaries found, handle remaining text
                if sentence_start_byte.0 < text.len() {
                    let content = text[sentence_start_byte.0..].trim().to_string();
                    if !content.is_empty() {
                        let start_char = byte_to_char_pos(text, sentence_start_byte)?;
                        let end_char = byte_to_char_pos(text, BytePos::new(text.len()))?;
                        
                        let (start_line, start_col) = char_to_line_col(text, start_char)?;
                        let (end_line, end_col) = char_to_line_col(text, end_char)?;
                        
                        sentences.push(DetectedSentence {
                            start_pos: start_char,
                            end_pos: end_char,
                            content,
                            start_line,
                            start_col,
                            end_line,
                            end_col,
                        });
                    }
                }
                break;
            }
        }
        
        Ok(sentences)
    }
    
    fn classify_match(&self, matched_text: &str, current_state: &DialogState) -> (MatchType, DialogState) {
        // Check for pure hard separator (exactly \n\n)
        if matched_text == "\n\n" {
            return (MatchType::HardSeparator, DialogState::Unknown);
        }
        
        // Check for narrative hard boundary (contains punctuation + \n\n + letter)
        if matched_text.contains("\n\n") {
            let has_punct = matched_text.chars().any(|c| ".!?".contains(c));
            let has_letter = matched_text.chars().any(|c| c.is_alphabetic());
            if has_punct && has_letter {
                return (MatchType::NarrativeGestureBoundary, DialogState::Narrative);
            }
        }
        
        match current_state {
            DialogState::Narrative => {
                // In narrative state, determine if this is a boundary or dialog open
                if matched_text.contains("[.!?]") && matched_text.contains(char::is_whitespace) {
                    // This is a narrative gesture boundary
                    (MatchType::NarrativeGestureBoundary, DialogState::Narrative)
                } else {
                    // This must be a dialog open - determine which type
                    if matched_text.contains('"') {
                        (MatchType::DialogOpen, DialogState::DialogDoubleQuote)
                    } else if matched_text.contains('\'') {
                        (MatchType::DialogOpen, DialogState::DialogSingleQuote)
                    } else if matched_text.contains('\u{201C}') {
                        (MatchType::DialogOpen, DialogState::DialogSmartDoubleOpen)
                    } else if matched_text.contains('\u{2018}') {
                        (MatchType::DialogOpen, DialogState::DialogSmartSingleOpen)
                    } else if matched_text.contains('(') {
                        (MatchType::DialogOpen, DialogState::DialogParenthheticalRound)
                    } else if matched_text.contains('[') {
                        (MatchType::DialogOpen, DialogState::DialogParenthheticalSquare)
                    } else if matched_text.contains('{') {
                        (MatchType::DialogOpen, DialogState::DialogParenthheticalCurly)
                    } else {
                        // Fallback
                        (MatchType::NarrativeGestureBoundary, DialogState::Narrative)
                    }
                }
            }
            DialogState::Unknown => {
                // After hard separator, determine next state based on what we found
                self.determine_state_from_context(matched_text)
            }
            _ => {
                // In dialog state - this must be a dialog end
                (MatchType::DialogEnd, DialogState::Narrative)
            }
        }
    }
    
    fn determine_state_from_context(&self, text: &str) -> (MatchType, DialogState) {
        // Check for dialog opens
        if text.contains('"') {
            (MatchType::DialogOpen, DialogState::DialogDoubleQuote)
        } else if text.contains('\'') {
            (MatchType::DialogOpen, DialogState::DialogSingleQuote)
        } else if text.contains('\u{201C}') {
            (MatchType::DialogOpen, DialogState::DialogSmartDoubleOpen)
        } else if text.contains('\u{2018}') {
            (MatchType::DialogOpen, DialogState::DialogSmartSingleOpen)
        } else if text.contains('(') {
            (MatchType::DialogOpen, DialogState::DialogParenthheticalRound)
        } else if text.contains('[') {
            (MatchType::DialogOpen, DialogState::DialogParenthheticalSquare)
        } else if text.contains('{') {
            (MatchType::DialogOpen, DialogState::DialogParenthheticalCurly)
        } else {
            // Default to narrative boundary
            (MatchType::NarrativeGestureBoundary, DialogState::Narrative)
        }
    }
    
    fn find_sent_sep_start(&self, matched_boundary: &str) -> Option<usize> {
        // Find where SENT_SEP starts within a SENT_END + SENT_SEP + SENT_START match
        // Look for the first whitespace character or \n\n
        if let Some(hard_sep_pos) = matched_boundary.find("\n\n") {
            return Some(hard_sep_pos);
        }
        
        // Find first whitespace after punctuation
        let mut found_punct = false;
        for (i, ch) in matched_boundary.char_indices() {
            if ".!?".contains(ch) || "\"'".contains(ch) || ")]}>".contains(ch) {
                found_punct = true;
            } else if found_punct && ch.is_whitespace() {
                return Some(i);
            }
        }
        
        None
    }
    
    fn find_sent_sep_end(&self, matched_boundary: &str) -> Option<usize> {
        // Find where SENT_SEP ends within a SENT_END + SENT_SEP + SENT_START match
        // This is where the next sentence should start
        if let Some(hard_sep_pos) = matched_boundary.find("\n\n") {
            return Some(hard_sep_pos + 2); // After the \n\n
        }
        
        // Find the end of whitespace sequence
        let mut in_whitespace = false;
        let mut whitespace_start = 0;
        
        for (i, ch) in matched_boundary.char_indices() {
            if ch.is_whitespace() {
                if !in_whitespace {
                    whitespace_start = i;
                    in_whitespace = true;
                }
            } else if in_whitespace {
                // Found non-whitespace after whitespace - this is start of SENT_START
                return Some(i);
            }
        }
        
        // If we end in whitespace, return end of string
        if in_whitespace {
            Some(matched_boundary.len())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_narrative_sentences() {
        let machine = DialogStateMachine::new().unwrap();
        let text = "This is a sentence. This is another sentence.";
        let sentences = machine.detect_sentences(text).unwrap();
        
        println!("DEBUG: Found {} sentences", sentences.len());
        for (i, sentence) in sentences.iter().enumerate() {
            println!("DEBUG: Sentence {}: '{}'", i, sentence.content);
        }
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].content, "This is a sentence.");
        assert_eq!(sentences[1].content, "This is another sentence.");
    }
    
    #[test]
    fn test_dialog_coalescing() {
        let machine = DialogStateMachine::new().unwrap();
        let text = "He said, \"Stop her, sir! Ting-a-ling-ling!\" The headway ran almost out.";
        let sentences = machine.detect_sentences(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].content, "He said, \"Stop her, sir! Ting-a-ling-ling!\"");
        assert_eq!(sentences[1].content, "The headway ran almost out.");
    }
    
    #[test]
    fn test_hard_separator() {
        let machine = DialogStateMachine::new().unwrap();
        let text = "First sentence.\n\nSecond sentence.";
        let sentences = machine.detect_sentences(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].content, "First sentence.");
        assert_eq!(sentences[1].content, "Second sentence.");
    }
    
    #[test]
    fn test_parenthetical_boundaries() {
        let machine = DialogStateMachine::new().unwrap();
        let text = "He left (quietly.) She followed.";
        let sentences = machine.detect_sentences(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].content, "He left (quietly.)");
        assert_eq!(sentences[1].content, "She followed.");
    }
    
    #[test]
    fn test_false_positive_examples() {
        let machine = DialogStateMachine::new().unwrap();
        
        // Example #1: Dialog coalescing
        let text = "The switch hovered in the air—the peril was desperate—\n\n\"My! Look behind you, aunt!\" The old lady whirled round.";
        let sentences = machine.detect_sentences(text).unwrap();
        
        // Should have 3 sentences: narrative, dialog, narrative
        assert_eq!(sentences.len(), 3);
        assert!(sentences[1].content.contains("My! Look behind you, aunt!"));
        
        // Example #5: Dialog with multiple exclamations
        let text = "He was boat and captain: \"Stop her, sir! Ting-a-ling-ling!\" The headway ran almost out.";
        let sentences = machine.detect_sentences(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].content, "He was boat and captain: \"Stop her, sir! Ting-a-ling-ling!\"");
        assert_eq!(sentences[1].content, "The headway ran almost out.");
    }
}