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

// PHASE 1: Incremental Position Tracking
// WHY: Eliminate O(N²) behavior from repeated byte_to_char_pos and char_to_line_col scans
#[derive(Debug)]
pub struct PositionTracker {
    current_byte_pos: usize,
    current_char_pos: usize,
    current_line: usize,
    current_col: usize,
    text_bytes: Vec<u8>,
}

impl PositionTracker {
    pub fn new(text: &str) -> Self {
        Self {
            current_byte_pos: 0,
            current_char_pos: 0,
            current_line: 1,
            current_col: 1,
            text_bytes: text.as_bytes().to_vec(),
        }
    }
    
    /// Advance incrementally to target byte position, updating char/line/col counters
    pub fn advance_to_byte(&mut self, target_byte_pos: BytePos) -> Result<(CharPos, OneBasedLine, OneBasedCol), String> {
        if target_byte_pos.0 < self.current_byte_pos {
            return Err(format!("Cannot seek backwards: current {} > target {}", self.current_byte_pos, target_byte_pos.0));
        }
        
        if target_byte_pos.0 > self.text_bytes.len() {
            return Err(format!("Target byte position {} exceeds text length {}", target_byte_pos.0, self.text_bytes.len()));
        }
        
        // Advance incrementally from current position to target
        while self.current_byte_pos < target_byte_pos.0 {
            let byte = self.text_bytes[self.current_byte_pos];
            
            // Check if this byte starts a UTF-8 character
            if (byte & 0x80) == 0 || (byte & 0xC0) == 0xC0 {
                // This is either ASCII (0xxxxxxx) or start of multi-byte (11xxxxxx)
                self.current_char_pos += 1;
                
                if byte == b'\n' {
                    self.current_line += 1;
                    self.current_col = 1;
                } else {
                    self.current_col += 1;
                }
            }
            // Continuation bytes (10xxxxxx) don't increment char_pos or col
            
            self.current_byte_pos += 1;
        }
        
        Ok((
            CharPos::new(self.current_char_pos),
            OneBasedLine::new(self.current_line).unwrap(),
            OneBasedCol::new(self.current_col).unwrap(),
        ))
    }
    
    /// Get current position without advancing
    #[allow(dead_code)]
    pub fn current_position(&self) -> (CharPos, OneBasedLine, OneBasedCol) {
        (
            CharPos::new(self.current_char_pos),
            OneBasedLine::new(self.current_line).unwrap(),
            OneBasedCol::new(self.current_col).unwrap(),
        )
    }
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
    #[allow(dead_code)]
    pub start_pos: CharPos,
    #[allow(dead_code)]
    pub end_pos: CharPos,
    pub content: String,
    #[allow(dead_code)]
    pub start_line: OneBasedLine,
    #[allow(dead_code)]
    pub start_col: OneBasedCol,
    #[allow(dead_code)]
    pub end_line: OneBasedLine,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    quote_starts: Regex,
    #[allow(dead_code)]
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
        
        // Dialog ending patterns: HARD_END (sentence boundary) vs SOFT_END (just dialog close)
        // HARD_END: sentence_end + close + separator + sentence_start (creates sentence boundary)
        // SOFT_END: just close (needs state transition logic)
        let dialog_hard_double_end = format!("{}{}{}{}", sentence_end_punct, double_quote_close, soft_separator, sentence_start_chars);
        let dialog_soft_double_end = format!("{}", double_quote_close);
        let dialog_double_end = format!("(?:{})|(?:{})", dialog_hard_double_end, dialog_soft_double_end);
        
        let dialog_hard_single_end = format!("{}{}{}{}", sentence_end_punct, single_quote_close, soft_separator, sentence_start_chars);
        let dialog_soft_single_end = format!("{}", single_quote_close);
        let dialog_single_end = format!("(?:{})|(?:{})", dialog_hard_single_end, dialog_soft_single_end);
        
        let dialog_hard_smart_double_end = format!("{}{}{}{}", sentence_end_punct, smart_double_close, soft_separator, sentence_start_chars);
        let dialog_soft_smart_double_end = format!("{}", smart_double_close);
        let dialog_smart_double_end = format!("(?:{})|(?:{})", dialog_hard_smart_double_end, dialog_soft_smart_double_end);
        
        let dialog_hard_smart_single_end = format!("{}{}{}{}", sentence_end_punct, smart_single_close, soft_separator, sentence_start_chars);
        let dialog_soft_smart_single_end = format!("{}", smart_single_close);
        let dialog_smart_single_end = format!("(?:{})|(?:{})", dialog_hard_smart_single_end, dialog_soft_smart_single_end);
        
        let dialog_hard_paren_round_end = format!("{}{}{}{}", sentence_end_punct, round_paren_close, soft_separator, sentence_start_chars);
        let dialog_soft_paren_round_end = format!("{}", round_paren_close);
        let dialog_paren_round_end = format!("(?:{})|(?:{})", dialog_hard_paren_round_end, dialog_soft_paren_round_end);
        
        let dialog_hard_paren_square_end = format!("{}{}{}{}", sentence_end_punct, square_bracket_close, soft_separator, sentence_start_chars);
        let dialog_soft_paren_square_end = format!("{}", square_bracket_close);
        let dialog_paren_square_end = format!("(?:{})|(?:{})", dialog_hard_paren_square_end, dialog_soft_paren_square_end);
        
        let dialog_hard_paren_curly_end = format!("{}{}{}{}", sentence_end_punct, curly_brace_close, soft_separator, sentence_start_chars);
        let dialog_soft_paren_curly_end = format!("{}", curly_brace_close);
        let dialog_paren_curly_end = format!("(?:{})|(?:{})", dialog_hard_paren_curly_end, dialog_soft_paren_curly_end);
        
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
        
        // PHASE 1: Use incremental position tracker instead of O(N) position conversions
        let mut position_tracker = PositionTracker::new(text);
        
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
                                // PHASE 1: Use incremental position tracker instead of O(N) conversions
                                let (start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)?;
                                let (end_char, end_line, end_col) = position_tracker.advance_to_byte(sentence_end_byte)?;
                                
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
                                // PHASE 1: Use incremental position tracker instead of O(N) conversions
                                let (start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)?;
                                let (end_char, end_line, end_col) = position_tracker.advance_to_byte(sentence_end_byte)?;
                                
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
                                // PHASE 1: Use incremental position tracker instead of O(N) conversions
                                let (start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)?;
                                let (end_char, end_line, end_col) = position_tracker.advance_to_byte(match_start_byte)?;
                                
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
                        // PHASE 1: Use incremental position tracker instead of O(N) conversions
                        let (start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)?;
                        let (end_char, end_line, end_col) = position_tracker.advance_to_byte(BytePos::new(text.len()))?;
                        
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
                let has_sentence_punct = matched_text.chars().any(|c| ".!?".contains(c));
                let has_whitespace = matched_text.chars().any(char::is_whitespace);
                
                if has_sentence_punct && has_whitespace {
                    // This is a narrative gesture boundary (. A, ! B, ? C pattern)
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
                // In dialog state - this must be a dialog end, analyze punctuation context
                self.classify_dialog_end(matched_text)
            }
        }
    }
    
    fn classify_dialog_end(&self, matched_text: &str) -> (MatchType, DialogState) {
        // Check if this is a HARD_END (sentence punctuation + close + separator) or SOFT_END (just close)
        let has_sentence_punct = matched_text.chars().any(|c| ".!?".contains(c));
        let has_separator = matched_text.chars().any(char::is_whitespace);
        
        if has_sentence_punct && has_separator {
            // HARD_END: This creates a sentence boundary and transitions to Narrative
            (MatchType::DialogEnd, DialogState::Narrative)
        } else {
            // SOFT_END: Just dialog close, creates soft transition, not hard boundary
            // Return DialogEnd but maintain dialog state for soft transition
            (MatchType::DialogEnd, DialogState::Narrative)
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
        
        for (i, ch) in matched_boundary.char_indices() {
            if ch.is_whitespace() {
                if !in_whitespace {
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
    
    #[test]
    fn test_false_positive_case_7_dialog_attribution() {
        let machine = DialogStateMachine::new().unwrap();
        
        // From FALSE_POSITIVE_examples.txt #7 - Dialog attribution should be coalesced
        // This should be ONE sentence, not split at the period before the quote
        let text = r#"They had been strangers too long. "It's all over, Mrs. Thingummy!" said the surgeon at last."#;
        
        let sentences = machine.detect_sentences(text).unwrap();
        
        println!("DEBUG: FALSE_POSITIVE #7 test found {} sentences", sentences.len());
        for (i, sentence) in sentences.iter().enumerate() {
            println!("DEBUG: Sentence {}: '{}'", i, sentence.content);
        }
        
        // Expected: Should be 2 sentences properly split
        // 1. "They had been strangers too long."
        // 2. "It's all over, Mrs. Thingummy!" said the surgeon at last.
        assert_eq!(sentences.len(), 2, 
            "Expected 2 sentences but got {}. Dialog state machine is not splitting correctly!", 
            sentences.len());
        
        // Check the correct sentence boundaries
        assert_eq!(sentences[0].content, "They had been strangers too long.");
        assert_eq!(sentences[1].content, r#""It's all over, Mrs. Thingummy!" said the surgeon at last."#);
    }

    #[test]
    fn test_false_negative_dialog_over_coalescing() {
        let machine = DialogStateMachine::new().unwrap();
        
        // From FALSE_NEGATIVE_examples.txt - Oliver Twist conversation
        // This should be split into multiple sentences, not treated as one massive narrative gesture
        let text = r#"(He stirred the gin-and-water.) "I—I drink your health with cheerfulness, Mrs. Mann"; and he swallowed half of it. "And now about business," said the beadle, taking out a leathern pocket-book. "The child that was half-baptized Oliver Twist, is nine year old today." "Bless him!" interposed Mrs. Mann, inflaming her left eye with the corner of her apron."#;
        
        let sentences = machine.detect_sentences(text).unwrap();
        
        println!("DEBUG: FALSE_NEGATIVE test found {} sentences", sentences.len());
        for (i, sentence) in sentences.iter().enumerate() {
            println!("DEBUG: Sentence {}: '{}'", i, sentence.content);
        }
        
        // Expected: Should split this into at least 5 separate sentences:
        // 1. "(He stirred the gin-and-water.)"
        // 2. "I—I drink your health with cheerfulness, Mrs. Mann"; and he swallowed half of it.
        // 3. "And now about business," said the beadle, taking out a leathern pocket-book.
        // 4. "The child that was half-baptized Oliver Twist, is nine year old today."
        // 5. "Bless him!" interposed Mrs. Mann, inflaming her left eye with the corner of her apron.
        
        // FAILING TEST: Currently over-coalesces into 1-2 sentences instead of proper splitting
        assert!(sentences.len() >= 5, 
            "Expected at least 5 sentences but got {}. Dialog state machine is over-coalescing!", 
            sentences.len());
    }
    
    #[test]
    fn test_generated_boundary_cases() {
        run_boundary_validation_workflow(false, 10);
    }
    
    #[test]
    fn test_populate_baseline_behavior() {
        run_boundary_validation_workflow(true, 162);
    }
    
    fn run_boundary_validation_workflow(populate_baseline: bool, limit: usize) {
        use serde::{Deserialize, Serialize};
        use regex_automata::Input;
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct GeneratedTestCase {
            id: String,
            pattern: String,
            current_state: String,
            full_text: String,
            context_before: String,
            context_after: String,
            expected_match_type: String,
            expected_next_state: String,
            creates_sentence_boundary: bool,
            validated: bool,
            source_rule: String,
            source_category: String,
            notes: String,
        }
        
        #[derive(Debug, Serialize, Deserialize)]
        struct GeneratedTestData {
            schema_version: String,
            description: String,
            generated_from: Vec<String>,
            test_cases: Vec<GeneratedTestCase>,
        }
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct ValidationStateEntry {
            expected_match_type: String,
            expected_next_state: String,
            validated: bool,
        }
        
        let machine = DialogStateMachine::new().unwrap();
        
        // Generate test cases to temp directory
        let temp_dir = std::env::temp_dir();
        let temp_test_file = temp_dir.join("boundary_tests_generated.json");
        
        // Run generator to create fresh test cases
        let generate_result = std::process::Command::new("cargo")
            .args(&["run", "--bin", "generate_boundary_tests"])
            .env("BOUNDARY_TEST_OUTPUT", &temp_test_file)
            .output();
            
        match generate_result {
            Ok(output) if output.status.success() => {
                println!("Generated fresh test cases to temp directory");
            },
            Ok(output) => {
                println!("Generator failed: {}", String::from_utf8_lossy(&output.stderr));
                return;
            },
            Err(e) => {
                println!("Could not run generator: {}. Using existing file if available.", e);
                // Fall back to existing file
            }
        }
        
        // Load generated test cases (try temp first, fall back to existing)
        let test_file = if temp_test_file.exists() {
            temp_test_file.to_str().unwrap()
        } else {
            "tests/generated_boundary_tests.json"
        };
        
        let json_content = match std::fs::read_to_string(test_file) {
            Ok(content) => content,
            Err(_) => {
                println!("Skipping test - no test file found. Run: cargo run --bin generate_boundary_tests");
                return;
            }
        };
        
        let mut test_data: GeneratedTestData = serde_json::from_str(&json_content).unwrap();
        
        // Load validation state if it exists
        let validation_state_file = "tests/boundary_validation_state.json";
        if let Ok(validation_content) = std::fs::read_to_string(validation_state_file) {
            if let Ok(validation_state) = serde_json::from_str::<std::collections::HashMap<String, ValidationStateEntry>>(&validation_content) {
                // Merge validation state into test cases
                for test_case in &mut test_data.test_cases {
                    if let Some(state) = validation_state.get(&test_case.id) {
                        test_case.expected_match_type = state.expected_match_type.clone();
                        test_case.expected_next_state = state.expected_next_state.clone();
                        test_case.validated = state.validated;
                    }
                }
                println!("Merged validation state from {}", validation_state_file);
            }
        }
        
        println!("Running {} generated boundary tests (limit: {})...", test_data.test_cases.len(), limit);
        if populate_baseline {
            println!("BASELINE MODE: Will update test cases with current behavior");
        }
        
        let mut passed = 0;
        let mut failed = 0;
        let mut no_match = 0;
        let mut baseline_recorded = 0;
        let mut attention_required = 0;
        let mut errors = Vec::new();
        let mut modified = false;
        
        // Category-wise tracking
        let mut category_stats: std::collections::HashMap<String, (usize, usize, usize, usize, usize)> = std::collections::HashMap::new();
        
        for test_case in test_data.test_cases.iter_mut().take(limit) {
            // Convert string state to DialogState enum
            let current_state = match test_case.current_state.as_str() {
                "Narrative" => DialogState::Narrative,
                "DialogDoubleQuote" => DialogState::DialogDoubleQuote,
                "DialogSingleQuote" => DialogState::DialogSingleQuote,
                "DialogSmartDoubleOpen" => DialogState::DialogSmartDoubleOpen,
                "DialogSmartSingleOpen" => DialogState::DialogSmartSingleOpen,
                "DialogParenthheticalRound" => DialogState::DialogParenthheticalRound,
                "DialogParenthheticalSquare" => DialogState::DialogParenthheticalSquare,
                "DialogParenthheticalCurly" => DialogState::DialogParenthheticalCurly,
                "Unknown" => DialogState::Unknown,
                _ => {
                    println!("SKIP: Unknown state '{}'", test_case.current_state);
                    continue;
                }
            };
            
            // Get the regex pattern for the current state
            let pattern_regex = match machine.patterns.get(&current_state) {
                Some(regex) => regex,
                None => {
                    println!("SKIP: No pattern for state {:?}", current_state);
                    continue;
                }
            };
            
            // Test if the pattern matches in the full text
            let pattern_pos = test_case.context_before.len();
            
            if pattern_pos >= test_case.full_text.len() {
                println!("SKIP: Pattern position {} >= text length {}", pattern_pos, test_case.full_text.len());
                continue;
            }
            
            let input_from_pattern = Input::new(&test_case.full_text[pattern_pos..]);
            
            let test_result = match pattern_regex.find(input_from_pattern) {
                Some(mat) => {
                    let matched_text = &test_case.full_text[pattern_pos + mat.start()..pattern_pos + mat.end()];
                    
                    // Check if the matched text contains our test pattern
                    if matched_text.contains(&test_case.pattern) {
                        let (actual_match_type, actual_next_state) = machine.classify_match(matched_text, &current_state);
                        
                        let actual_match_str = format!("{:?}", actual_match_type);
                        let actual_state_str = format!("{:?}", actual_next_state);
                        
                        Some((actual_match_str, actual_state_str, matched_text.to_string()))
                    } else {
                        None // Pattern not found in matched text
                    }
                },
                None => None // No regex match
            };
            
            // Apply validation workflow
            let result_type = match test_result {
                Some((actual_match_str, actual_state_str, matched_text)) => {
                    println!("TEST: {} | Pattern: '{}' | Matched: '{}'", 
                        test_case.id, test_case.pattern, matched_text);
                    println!("  Actual: {} -> {}", actual_match_str, actual_state_str);
                    
                    if test_case.expected_match_type == "UNKNOWN" {
                        // BASELINE RECORDING
                        if populate_baseline {
                            test_case.expected_match_type = actual_match_str.clone();
                            test_case.expected_next_state = actual_state_str.clone();
                            modified = true;
                            println!("  BASELINE_RECORDED: {} -> {}", actual_match_str, actual_state_str);
                            baseline_recorded += 1;
                            "baseline_recorded"
                        } else {
                            println!("  BASELINE: Current behavior {} -> {}", actual_match_str, actual_state_str);
                            passed += 1;
                            "passed"
                        }
                    } else {
                        // VALIDATION AGAINST EXPECTED
                        let behavior_changed = actual_match_str != test_case.expected_match_type || 
                                             actual_state_str != test_case.expected_next_state;
                        
                        if test_case.validated {
                            if behavior_changed {
                                println!("  ERROR: Validated test behavior changed!");
                                println!("    Expected: {} -> {}", test_case.expected_match_type, test_case.expected_next_state);
                                println!("    Actual:   {} -> {}", actual_match_str, actual_state_str);
                                errors.push(format!("{}: Validated behavior changed", test_case.id));
                                failed += 1;
                                "failed"
                            } else {
                                println!("  PASS: Validated behavior unchanged");
                                passed += 1;
                                "passed"
                            }
                        } else {
                            if behavior_changed {
                                println!("  ATTENTION_REQUIRED: Unvalidated behavior changed!");
                                println!("    Expected: {} -> {}", test_case.expected_match_type, test_case.expected_next_state);
                                println!("    Actual:   {} -> {}", actual_match_str, actual_state_str);
                                attention_required += 1;
                                "attention_required"
                            } else {
                                println!("  PASS: Unvalidated behavior unchanged");
                                passed += 1;
                                "passed"
                            }
                        }
                    }
                },
                None => {
                    if test_case.expected_match_type == "NO_MATCH" {
                        println!("PASS: {} | No match as expected", test_case.id);
                        passed += 1;
                        "passed"
                    } else {
                        println!("NO_MATCH: {} | Pattern '{}' not found", test_case.id, test_case.pattern);
                        no_match += 1;
                        "no_match"
                    }
                }
            };
            
            // Update category stats
            let category = &test_case.source_category;
            let stats = category_stats.entry(category.clone()).or_insert((0, 0, 0, 0, 0));
            match result_type {
                "passed" => stats.0 += 1,
                "failed" => stats.1 += 1,
                "no_match" => stats.2 += 1,
                "baseline_recorded" => stats.3 += 1,
                "attention_required" => stats.4 += 1,
                _ => {}
            }
            println!();
        }
        
        // Write back validation state if we populated baselines or made changes
        if populate_baseline && modified {
            // Extract validation state
            let mut validation_state = std::collections::HashMap::new();
            for test_case in &test_data.test_cases {
                validation_state.insert(test_case.id.clone(), ValidationStateEntry {
                    expected_match_type: test_case.expected_match_type.clone(),
                    expected_next_state: test_case.expected_next_state.clone(),
                    validated: test_case.validated,
                });
            }
            
            // Write validation state to separate file
            let validation_state_file = "tests/boundary_validation_state.json";
            let validation_json = serde_json::to_string_pretty(&validation_state).unwrap();
            std::fs::write(validation_state_file, validation_json).expect("Failed to write validation state");
            println!("Updated {} with baseline behavior", validation_state_file);
        }
        
        println!("\n=== BOUNDARY VALIDATION SUMMARY ===");
        let total_tested = passed + failed + no_match + baseline_recorded + attention_required;
        
        // Overall results
        println!("Total Tests: {}", total_tested);
        println!("Passed: {} ({:.1}%)", passed, if total_tested > 0 { (passed as f64 / total_tested as f64) * 100.0 } else { 0.0 });
        if failed > 0 {
            println!("Failed: {} ({:.1}%)", failed, (failed as f64 / total_tested as f64) * 100.0);
        }
        if no_match > 0 {
            println!("No Match: {} ({:.1}%)", no_match, (no_match as f64 / total_tested as f64) * 100.0);
        }
        if baseline_recorded > 0 {
            println!("Baseline Recorded: {} ({:.1}%)", baseline_recorded, (baseline_recorded as f64 / total_tested as f64) * 100.0);
        }
        if attention_required > 0 {
            println!("Attention Required: {} ({:.1}%)", attention_required, (attention_required as f64 / total_tested as f64) * 100.0);
        }
        
        // Category breakdown
        println!("\n=== RESULTS BY CATEGORY ===");
        let mut categories: Vec<_> = category_stats.keys().collect();
        categories.sort();
        
        for category in categories {
            let (cat_passed, cat_failed, cat_no_match, cat_baseline, cat_attention) = category_stats.get(category).unwrap();
            let cat_total = cat_passed + cat_failed + cat_no_match + cat_baseline + cat_attention;
            
            if cat_total == 0 { continue; }
            
            let success_rate = if cat_total > 0 { 
                ((cat_passed + cat_baseline) as f64 / cat_total as f64) * 100.0 
            } else { 
                0.0 
            };
            
            println!("{}: {:.1}% success ({}/{} tests)", 
                category, success_rate, cat_passed + cat_baseline, cat_total);
            
            if *cat_failed > 0 || *cat_attention > 0 || *cat_no_match > 0 {
                print!("  ");
                if *cat_failed > 0 {
                    print!("Failed: {} ", cat_failed);
                }
                if *cat_attention > 0 {
                    print!("Attention: {} ", cat_attention);
                }
                if *cat_no_match > 0 {
                    print!("No Match: {} ", cat_no_match);
                }
                println!();
            }
        }
        
        // Error details
        if !errors.is_empty() {
            println!("\n=== ERRORS (Validated behavior changed) ===");
            for error in errors {
                println!("  {}", error);
            }
        }
        
        // Attention items
        if attention_required > 0 {
            println!("\n=== ATTENTION REQUIRED ===");
            println!("{} unvalidated tests changed behavior", attention_required);
            println!("Review these changes and mark as validated if correct");
        }
        
        // Overall status
        println!("\n=== OVERALL STATUS ===");
        if failed > 0 {
            println!("❌ FAILED: {} validated tests changed behavior", failed);
        } else if attention_required > 0 {
            println!("⚠️  ATTENTION: {} unvalidated tests need review", attention_required);
        } else {
            println!("✅ PASSED: All tests behaving as expected");
        }
    }
}