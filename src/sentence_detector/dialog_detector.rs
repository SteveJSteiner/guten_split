// WHY: Dialog-aware sentence detection with state machine for coalescing and sophisticated boundaries
// Moved from tests to official implementation with dual API support

use anyhow::Result;
use regex_automata::{meta::Regex, Input};
use std::collections::HashMap;
use tracing::{debug, info};

use super::{DetectedSentenceBorrowed, Span, AbbreviationChecker};

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
            text_bytes: text.as_bytes().to_vec(), // TODO: Should use &[u8] to avoid copy, but requires lifetime parameter
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

/// Internal representation for dialog state machine
#[derive(Debug, Clone)]
pub struct DialogDetectedSentence {
    pub start_byte: BytePos,  // Added for O(1) borrowed API
    pub end_byte: BytePos,    // Added for O(1) borrowed API
    pub start_line: OneBasedLine,
    pub start_col: OneBasedCol,
    pub end_line: OneBasedLine,
    pub end_col: OneBasedCol,
}

#[derive(Debug, Clone)]
pub enum MatchType {
    NarrativeGestureBoundary,
    NarrativeToDialog,  // New: N→D transition that creates sentence boundary AND enters dialog
    DialogOpen,
    DialogEnd,
    DialogSoftEnd,
    HardSeparator,
}

pub struct DialogStateMachine {
    state_patterns: HashMap<DialogState, Regex>,
    state_pattern_mappings: HashMap<DialogState, Vec<(MatchType, DialogState)>>,
    abbreviation_checker: AbbreviationChecker,
}

impl DialogStateMachine {
    
    /// Check if hard separator should be rejected due to preceding internal punctuation
    /// O(1) operation - scans backward to find meaningful punctuation (typically 1-5 bytes)
    fn should_reject_hard_separator(&self, text_bytes: &[u8], separator_start_byte: usize) -> bool {
        if separator_start_byte == 0 {
            return false;
        }
        
        // WHY: Walk backward to find meaningful punctuation, skipping whitespace and closing delimiters
        // Expected to exit early (1-5 iterations) in typical cases
        let mut pos = separator_start_byte;
        let scan_limit = separator_start_byte.saturating_sub(20); // Safety guard
        
        while pos > scan_limit {
            pos -= 1;
            let byte = text_bytes[pos];
            
            // Skip whitespace bytes - continue scanning
            if matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
                continue;
            }
            
            // Found non-whitespace - check if it's a UTF-8 continuation byte
            if (byte & 0xC0) == 0x80 {
                // This is a continuation byte (10xxxxxx), keep going to find start byte
                continue;
            }
            
            // This is either ASCII (0xxxxxxx) or multi-byte start (11xxxxxx)
            if byte < 0x80 {
                // ASCII character - check what type it is
                match byte {
                    // Terminal punctuation - allow hard separator (don't reject)
                    b'.' | b'?' | b'!' => return false,
                    
                    // Closing delimiters - accept hard separator (they are terminal)
                    b'"' | b'\'' | b')' | b']' | b'}' => return false,
                    
                    // Internal punctuation - reject hard separator (coalesce)
                    b',' | b';' | b':' | b'-' | b'/' | 
                    b'(' | b'[' | b'{' => return true,
                    
                    // Other characters (letters, digits, etc.) - treat as continuation, reject separator
                    _ => return true,
                }
            } else {
                // Multi-byte UTF-8 character - decode from start byte
                let remaining_bytes = &text_bytes[pos..separator_start_byte.min(pos + 4)];
                if let Ok(utf8_str) = std::str::from_utf8(remaining_bytes) {
                    if let Some(ch) = utf8_str.chars().next() {
                        match ch {
                            // Terminal punctuation - allow hard separator
                            '.' | '?' | '!' => return false,
                            
                            // Smart closing quotes - accept hard separator (they are terminal)
                            '\u{201D}' | '\u{2019}' => return false,
                            
                            // Internal punctuation (em/en dash, smart opening quotes) - reject separator
                            '\u{2014}' | '\u{2013}' | '\u{201C}' | '\u{2018}' => return true,
                            
                            // Ellipsis - treat as continuation, reject separator
                            '\u{2026}' => return true,
                            
                            // Other Unicode - treat as continuation, reject separator
                            _ => return true,
                        }
                    }
                }
                // If decode fails, conservatively reject (coalesce)
                return true;
            }
        }
        
        // No meaningful punctuation found in scan window - treat as continuation, reject separator
        true
    }

    /// Helper function to negate a character class
    fn negate_char_class(char_class: &str) -> String {
        format!("[^{}]", &char_class[1..char_class.len()-1])
    }

    pub fn new() -> Result<Self> {
        let mut state_patterns = HashMap::new();
        let mut state_pattern_mappings = HashMap::new();
        
        // Compositional pattern components
        let sentence_end_punct = r"[.!?]";
        let soft_separator = r"[ \t]+";  // Only spaces/tabs within same line  
        let line_boundary = r"(?:\r?\n)";  // single newline for line-end patterns
        let hard_separator = r"(?:\r\n\r\n|\n\n)";   // double newline (Windows or Unix)
        
        // SOLUTION: Split sentence start chars to prevent overlaps
        let non_dialog_sentence_start_chars = r"[A-Z]";  // Only capital letters for narrative boundaries
        let dialog_open_chars = r"[\x22\x27\u{201C}\u{2018}\(\[\{]";  // All dialog opening characters
        
        let dialog_prefix_whitespace = r"[ \t\n]";  // Space, tab, or newline
        
        // NEW APPROACH: Explicit patterns for each dialog character type
        // Distinguish between sentence-ending and non-sentence-ending punctuation
        let sentence_ending_punct = r"[.!?]";  // These create sentence boundaries
        let non_sentence_ending_punct = r"[,:;]";  // These do NOT create sentence boundaries
        
        // Individual dialog characters
        let double_quote_char = r"\x22";         // "
        let single_quote_char = r"\x27";         // '
        let smart_double_open = r"\u{201C}";     // “
        let smart_single_open = r"\u{2018}";     // ‘
        let round_paren_open = r"\(";            // (
        let square_bracket_open = r"\[";         // [
        let curly_brace_open = r"\{";            // {
        
        // N→N narrative sentence boundaries
        let narrative_sentence_boundary = format!("{sentence_ending_punct}({soft_separator}){non_dialog_sentence_start_chars}");
        let narrative_line_boundary = format!("{sentence_ending_punct}{line_boundary}{non_dialog_sentence_start_chars}");
        let narrative_hard_boundary = format!(r"{sentence_ending_punct}\s*{hard_separator}\s*{non_dialog_sentence_start_chars}");
        
        // N→D transitions WITH sentence boundary (sentence punct + space + specific dialog char)
        let narrative_to_double_quote_boundary = format!("{sentence_ending_punct}({soft_separator}){double_quote_char}");
        let narrative_to_single_quote_boundary = format!("{sentence_ending_punct}({soft_separator}){single_quote_char}");
        let narrative_to_smart_double_boundary = format!("{sentence_ending_punct}({soft_separator}){smart_double_open}");
        let narrative_to_smart_single_boundary = format!("{sentence_ending_punct}({soft_separator}){smart_single_open}");
        let narrative_to_round_paren_boundary = format!("{sentence_ending_punct}({soft_separator}){round_paren_open}");
        let narrative_to_square_bracket_boundary = format!("{sentence_ending_punct}({soft_separator}){square_bracket_open}");
        let narrative_to_curly_brace_boundary = format!("{sentence_ending_punct}({soft_separator}){curly_brace_open}");
        
        // N→D transitions WITHOUT sentence boundary (non-sentence punct + space + specific dialog char)
        let narrative_to_double_quote_no_boundary = format!("{non_sentence_ending_punct}({soft_separator}){double_quote_char}");
        let narrative_to_single_quote_no_boundary = format!("{non_sentence_ending_punct}({soft_separator}){single_quote_char}");
        let narrative_to_smart_double_no_boundary = format!("{non_sentence_ending_punct}({soft_separator}){smart_double_open}");
        let narrative_to_smart_single_no_boundary = format!("{non_sentence_ending_punct}({soft_separator}){smart_single_open}");
        let narrative_to_round_paren_no_boundary = format!("{non_sentence_ending_punct}({soft_separator}){round_paren_open}");
        let narrative_to_square_bracket_no_boundary = format!("{non_sentence_ending_punct}({soft_separator}){square_bracket_open}");
        let narrative_to_curly_brace_no_boundary = format!("{non_sentence_ending_punct}({soft_separator}){curly_brace_open}");
        
        // Independent dialog starts (whitespace + dialog char, not after punctuation)
        let double_quote_independent = format!("{dialog_prefix_whitespace}{double_quote_char}");
        let single_quote_independent = format!("{dialog_prefix_whitespace}{single_quote_char}");
        let smart_double_independent = format!("{dialog_prefix_whitespace}{smart_double_open}");
        let smart_single_independent = format!("{dialog_prefix_whitespace}{smart_single_open}");
        let round_paren_independent = format!("{dialog_prefix_whitespace}{round_paren_open}");
        let square_bracket_independent = format!("{dialog_prefix_whitespace}{square_bracket_open}");
        let curly_brace_independent = format!("{dialog_prefix_whitespace}{curly_brace_open}");
        
        let pure_hard_sep = hard_separator.to_string();
        
        
        // Dialog closing characters
        let double_quote_close = r"\x22";      // "
        let single_quote_close = r"\x27";      // '
        let smart_double_close = r"\u{201D}";  // "
        let smart_single_close = r"\u{2019}";  // '
        let round_paren_close = r"\)";         // )
        let square_bracket_close = r"\]";      // ]
        let curly_brace_close = r"\}";         // }
        
        
        // Dialog ending patterns: HARD_END (sentence boundary) vs SOFT_END (continue sentence)
        // Use same approach - separate dialog and non-dialog sentence starts
        let all_sentence_start_chars = format!("{non_dialog_sentence_start_chars}|{dialog_open_chars}");
        let not_all_sentence_start_chars = Self::negate_char_class(&all_sentence_start_chars);
        
        let dialog_hard_double_end = format!("{sentence_end_punct}{double_quote_close}({soft_separator})[{}{}]", &non_dialog_sentence_start_chars[1..non_dialog_sentence_start_chars.len()-1], &dialog_open_chars[1..dialog_open_chars.len()-1]);
        let dialog_soft_double_end = format!("{sentence_end_punct}{double_quote_close}({soft_separator}){not_all_sentence_start_chars}");
        
        // NARRATIVE STATE - Mutually exclusive patterns
        let narrative_patterns = vec![
            // N→D with sentence boundary (highest priority - create sentence break)
            narrative_to_double_quote_boundary.as_str(),       // PatternID 0
            narrative_to_single_quote_boundary.as_str(),       // PatternID 1
            narrative_to_smart_double_boundary.as_str(),       // PatternID 2
            narrative_to_smart_single_boundary.as_str(),       // PatternID 3
            narrative_to_round_paren_boundary.as_str(),        // PatternID 4
            narrative_to_square_bracket_boundary.as_str(),     // PatternID 5
            narrative_to_curly_brace_boundary.as_str(),        // PatternID 6
            
            // N→D without sentence boundary (continue current sentence)
            narrative_to_double_quote_no_boundary.as_str(),    // PatternID 7
            narrative_to_single_quote_no_boundary.as_str(),    // PatternID 8
            narrative_to_smart_double_no_boundary.as_str(),    // PatternID 9
            narrative_to_smart_single_no_boundary.as_str(),    // PatternID 10
            narrative_to_round_paren_no_boundary.as_str(),     // PatternID 11
            narrative_to_square_bracket_no_boundary.as_str(),  // PatternID 12
            narrative_to_curly_brace_no_boundary.as_str(),     // PatternID 13
            
            // Independent dialog starts
            double_quote_independent.as_str(),                 // PatternID 14
            single_quote_independent.as_str(),                 // PatternID 15
            smart_double_independent.as_str(),                 // PatternID 16
            smart_single_independent.as_str(),                 // PatternID 17
            round_paren_independent.as_str(),                  // PatternID 18
            square_bracket_independent.as_str(),               // PatternID 19
            curly_brace_independent.as_str(),                  // PatternID 20
            
            // N→N narrative boundaries
            narrative_line_boundary.as_str(),                  // PatternID 21
            narrative_sentence_boundary.as_str(),              // PatternID 22
            narrative_hard_boundary.as_str(),                  // PatternID 23
            pure_hard_sep.as_str(),                           // PatternID 24
        ];
        let narrative_mappings = vec![
            // N→D with sentence boundary (create sentence break + enter dialog)
            (MatchType::NarrativeToDialog, DialogState::DialogDoubleQuote),        // PatternID 0
            (MatchType::NarrativeToDialog, DialogState::DialogSingleQuote),        // PatternID 1
            (MatchType::NarrativeToDialog, DialogState::DialogSmartDoubleOpen),    // PatternID 2
            (MatchType::NarrativeToDialog, DialogState::DialogSmartSingleOpen),    // PatternID 3
            (MatchType::NarrativeToDialog, DialogState::DialogParenthheticalRound), // PatternID 4
            (MatchType::NarrativeToDialog, DialogState::DialogParenthheticalSquare), // PatternID 5
            (MatchType::NarrativeToDialog, DialogState::DialogParenthheticalCurly), // PatternID 6
            
            // N→D without sentence boundary (continue sentence + enter dialog)
            (MatchType::DialogOpen, DialogState::DialogDoubleQuote),              // PatternID 7
            (MatchType::DialogOpen, DialogState::DialogSingleQuote),              // PatternID 8
            (MatchType::DialogOpen, DialogState::DialogSmartDoubleOpen),          // PatternID 9
            (MatchType::DialogOpen, DialogState::DialogSmartSingleOpen),          // PatternID 10
            (MatchType::DialogOpen, DialogState::DialogParenthheticalRound),      // PatternID 11
            (MatchType::DialogOpen, DialogState::DialogParenthheticalSquare),     // PatternID 12
            (MatchType::DialogOpen, DialogState::DialogParenthheticalCurly),      // PatternID 13
            
            // Independent dialog starts (enter dialog)
            (MatchType::DialogOpen, DialogState::DialogDoubleQuote),              // PatternID 14
            (MatchType::DialogOpen, DialogState::DialogSingleQuote),              // PatternID 15
            (MatchType::DialogOpen, DialogState::DialogSmartDoubleOpen),          // PatternID 16
            (MatchType::DialogOpen, DialogState::DialogSmartSingleOpen),          // PatternID 17
            (MatchType::DialogOpen, DialogState::DialogParenthheticalRound),      // PatternID 18
            (MatchType::DialogOpen, DialogState::DialogParenthheticalSquare),     // PatternID 19
            (MatchType::DialogOpen, DialogState::DialogParenthheticalCurly),      // PatternID 20
            
            // N→N narrative boundaries (stay in narrative)
            (MatchType::NarrativeGestureBoundary, DialogState::Narrative),        // PatternID 21
            (MatchType::NarrativeGestureBoundary, DialogState::Narrative),        // PatternID 22
            (MatchType::NarrativeGestureBoundary, DialogState::Narrative),        // PatternID 23
            (MatchType::HardSeparator, DialogState::Unknown),                     // PatternID 24
        ];
        state_patterns.insert(DialogState::Narrative, Regex::new_many(&narrative_patterns)?);
        state_pattern_mappings.insert(DialogState::Narrative, narrative_mappings);
        
        // DIALOG DOUBLE QUOTE STATE - Multi-pattern with hard separator FIRST (highest priority)
        let dialog_double_patterns = vec![
            pure_hard_sep.as_str(),                  // PatternID 0 = paragraph break (HIGHEST PRIORITY)
            dialog_hard_double_end.as_str(),         // PatternID 1 = sentence boundary
            dialog_soft_double_end.as_str(),         // PatternID 2 = continue sentence  
        ];
        let dialog_double_mappings = vec![
            (MatchType::HardSeparator, DialogState::Unknown),        // Hard separator
            (MatchType::DialogEnd, DialogState::Narrative),          // Hard dialog end
            (MatchType::DialogSoftEnd, DialogState::Narrative),      // Soft dialog end
        ];
        state_patterns.insert(DialogState::DialogDoubleQuote, Regex::new_many(&dialog_double_patterns)?);
        state_pattern_mappings.insert(DialogState::DialogDoubleQuote, dialog_double_mappings);
        
        // DIALOG SINGLE QUOTE STATE - Similar structure
        let dialog_hard_single_end = format!("{sentence_end_punct}{single_quote_close}({soft_separator})[{}{}]", &non_dialog_sentence_start_chars[1..non_dialog_sentence_start_chars.len()-1], &dialog_open_chars[1..dialog_open_chars.len()-1]);
        let dialog_soft_single_end = format!("{sentence_end_punct}{single_quote_close}({soft_separator}){not_all_sentence_start_chars}");
        
        let dialog_single_patterns = vec![
            pure_hard_sep.as_str(),                  // PatternID 0 = paragraph break (HIGHEST PRIORITY)
            dialog_hard_single_end.as_str(),         // PatternID 1 = sentence boundary
            dialog_soft_single_end.as_str(),         // PatternID 2 = continue sentence
        ];
        let dialog_single_mappings = vec![
            (MatchType::HardSeparator, DialogState::Unknown),
            (MatchType::DialogEnd, DialogState::Narrative),
            (MatchType::DialogSoftEnd, DialogState::Narrative),
        ];
        state_patterns.insert(DialogState::DialogSingleQuote, Regex::new_many(&dialog_single_patterns)?);
        state_pattern_mappings.insert(DialogState::DialogSingleQuote, dialog_single_mappings);
        
        // DIALOG SMART DOUBLE QUOTE STATE  
        let dialog_hard_smart_double_end = format!("{sentence_end_punct}{smart_double_close}({soft_separator})[{}{}]", &non_dialog_sentence_start_chars[1..non_dialog_sentence_start_chars.len()-1], &dialog_open_chars[1..dialog_open_chars.len()-1]);
        let dialog_soft_smart_double_end = format!("{sentence_end_punct}{smart_double_close}({soft_separator}){not_all_sentence_start_chars}");
        
        let dialog_smart_double_patterns = vec![
            pure_hard_sep.as_str(),
            dialog_hard_smart_double_end.as_str(),
            dialog_soft_smart_double_end.as_str(),
        ];
        let dialog_smart_double_mappings = vec![
            (MatchType::HardSeparator, DialogState::Unknown),
            (MatchType::DialogEnd, DialogState::Narrative),
            (MatchType::DialogSoftEnd, DialogState::Narrative),
        ];
        state_patterns.insert(DialogState::DialogSmartDoubleOpen, Regex::new_many(&dialog_smart_double_patterns)?);
        state_pattern_mappings.insert(DialogState::DialogSmartDoubleOpen, dialog_smart_double_mappings);
        
        // DIALOG SMART SINGLE QUOTE STATE
        let dialog_hard_smart_single_end = format!("{sentence_end_punct}{smart_single_close}({soft_separator})[{}{}]", &non_dialog_sentence_start_chars[1..non_dialog_sentence_start_chars.len()-1], &dialog_open_chars[1..dialog_open_chars.len()-1]);
        let dialog_soft_smart_single_end = format!("{sentence_end_punct}{smart_single_close}({soft_separator}){not_all_sentence_start_chars}");
        
        let dialog_smart_single_patterns = vec![
            pure_hard_sep.as_str(),
            dialog_hard_smart_single_end.as_str(),
            dialog_soft_smart_single_end.as_str(),
        ];
        let dialog_smart_single_mappings = vec![
            (MatchType::HardSeparator, DialogState::Unknown),
            (MatchType::DialogEnd, DialogState::Narrative),
            (MatchType::DialogSoftEnd, DialogState::Narrative),
        ];
        state_patterns.insert(DialogState::DialogSmartSingleOpen, Regex::new_many(&dialog_smart_single_patterns)?);
        state_pattern_mappings.insert(DialogState::DialogSmartSingleOpen, dialog_smart_single_mappings);
        
        // DIALOG PARENTHETICAL ROUND STATE
        let dialog_hard_paren_round_end = format!("{sentence_end_punct}{round_paren_close}({soft_separator})[{}{}]", &non_dialog_sentence_start_chars[1..non_dialog_sentence_start_chars.len()-1], &dialog_open_chars[1..dialog_open_chars.len()-1]);
        let dialog_soft_paren_round_end = format!("{sentence_end_punct}{round_paren_close}({soft_separator}){not_all_sentence_start_chars}");
        
        let dialog_paren_round_patterns = vec![
            pure_hard_sep.as_str(),
            dialog_hard_paren_round_end.as_str(),
            dialog_soft_paren_round_end.as_str(),
        ];
        let dialog_paren_round_mappings = vec![
            (MatchType::HardSeparator, DialogState::Unknown),
            (MatchType::DialogEnd, DialogState::Narrative),
            (MatchType::DialogSoftEnd, DialogState::Narrative),
        ];
        state_patterns.insert(DialogState::DialogParenthheticalRound, Regex::new_many(&dialog_paren_round_patterns)?);
        state_pattern_mappings.insert(DialogState::DialogParenthheticalRound, dialog_paren_round_mappings);
        
        // DIALOG PARENTHETICAL SQUARE STATE
        let dialog_hard_paren_square_end = format!("{sentence_end_punct}{square_bracket_close}({soft_separator})[{}{}]", &non_dialog_sentence_start_chars[1..non_dialog_sentence_start_chars.len()-1], &dialog_open_chars[1..dialog_open_chars.len()-1]);
        let dialog_soft_paren_square_end = format!("{sentence_end_punct}{square_bracket_close}({soft_separator}){not_all_sentence_start_chars}");
        
        let dialog_paren_square_patterns = vec![
            pure_hard_sep.as_str(),
            dialog_hard_paren_square_end.as_str(),
            dialog_soft_paren_square_end.as_str(),
        ];
        let dialog_paren_square_mappings = vec![
            (MatchType::HardSeparator, DialogState::Unknown),
            (MatchType::DialogEnd, DialogState::Narrative),
            (MatchType::DialogSoftEnd, DialogState::Narrative),
        ];
        state_patterns.insert(DialogState::DialogParenthheticalSquare, Regex::new_many(&dialog_paren_square_patterns)?);
        state_pattern_mappings.insert(DialogState::DialogParenthheticalSquare, dialog_paren_square_mappings);
        
        // DIALOG PARENTHETICAL CURLY STATE
        let dialog_hard_paren_curly_end = format!("{sentence_end_punct}{curly_brace_close}({soft_separator})[{}{}]", &non_dialog_sentence_start_chars[1..non_dialog_sentence_start_chars.len()-1], &dialog_open_chars[1..dialog_open_chars.len()-1]);
        let dialog_soft_paren_curly_end = format!("{sentence_end_punct}{curly_brace_close}({soft_separator}){not_all_sentence_start_chars}");
        
        let dialog_paren_curly_patterns = vec![
            pure_hard_sep.as_str(),
            dialog_hard_paren_curly_end.as_str(),
            dialog_soft_paren_curly_end.as_str(),
        ];
        let dialog_paren_curly_mappings = vec![
            (MatchType::HardSeparator, DialogState::Unknown),
            (MatchType::DialogEnd, DialogState::Narrative),
            (MatchType::DialogSoftEnd, DialogState::Narrative),
        ];
        state_patterns.insert(DialogState::DialogParenthheticalCurly, Regex::new_many(&dialog_paren_curly_patterns)?);
        state_pattern_mappings.insert(DialogState::DialogParenthheticalCurly, dialog_paren_curly_mappings);
        
        Ok(DialogStateMachine {
            state_patterns,
            state_pattern_mappings,
            abbreviation_checker: AbbreviationChecker::new(),
        })
    }
    
    
    pub fn detect_sentences(&self, text: &str) -> Result<Vec<DialogDetectedSentence>> {
        debug!("Starting dialog state machine detection on {} characters", text.len());
        
        let mut sentences = Vec::new();
        let mut current_state = DialogState::Narrative;
        let mut sentence_start_byte = BytePos::new(0);
        let mut position_byte = BytePos::new(0);
        let mut remaining_text_handled = false;
        
        // PHASE 1: Use incremental position tracker instead of O(N) position conversions
        let mut position_tracker = PositionTracker::new(text);        
        
        while position_byte.0 < text.len() {
            let pattern = match self.state_patterns.get(&current_state) {
                Some(p) => p,
                None => {
                    // Fallback to narrative pattern for unknown states
                    self.state_patterns.get(&DialogState::Narrative).unwrap()
                }
            };
            
            let pattern_mappings = match self.state_pattern_mappings.get(&current_state) {
                Some(m) => m,
                None => {
                    // Fallback to narrative mappings for unknown states
                    self.state_pattern_mappings.get(&DialogState::Narrative).unwrap()
                }
            };
            
            let input = Input::new(&text[position_byte.0..]);
            
            // Use state-specific multi-pattern regex
            if let Some(mat) = pattern.find(input) {
                let match_start_byte = position_byte.advance(mat.start());
                let match_end_byte = position_byte.advance(mat.end());
                let matched_text = &text[match_start_byte.0..match_end_byte.0];
                
                // Get PatternID and map directly to (MatchType, DialogState) using state-specific mappings
                let pattern_id = mat.pattern().as_usize();
                let (match_type, next_state) = if pattern_id < pattern_mappings.len() {
                    pattern_mappings[pattern_id].clone()
                } else {
                    // Fallback for invalid pattern ID
                    (MatchType::NarrativeGestureBoundary, DialogState::Narrative)
                };
                
                
                // Handle special cases for hard separator context
                let (match_type, next_state) = if matches!(match_type, MatchType::HardSeparator) {
                    // Check if this hard separator should be rejected due to preceding internal punctuation
                    if self.should_reject_hard_separator(text.as_bytes(), match_start_byte.0) {
                        (MatchType::DialogSoftEnd, current_state.clone())
                    } else {
                        (match_type, next_state)
                    }
                } else {
                    (match_type, next_state)
                };
                
                match match_type {
                    MatchType::NarrativeToDialog => {
                        // N→D transition: creates sentence boundary AND transitions to dialog state
                        let sentence_end_byte = self.find_sent_sep_start(matched_text)
                            .map(|sep_offset| match_start_byte.advance(sep_offset))
                            .unwrap_or(match_start_byte);
                        
                        if sentence_end_byte.0 > sentence_start_byte.0 {
                            let content = text[sentence_start_byte.0..sentence_end_byte.0].trim().to_string();
                            if !content.is_empty() {
                                // WHY: Check for abbreviation false positives before creating sentence boundary
                                if self.abbreviation_checker.ends_with_title_abbreviation(&content) {
                                    // This is a false positive - don't create sentence boundary
                                    // Continue processing from current position without advancing sentence_start_byte
                                } else {
                                    // Use incremental position tracker instead of O(N) conversions
                                    let (__start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)
                                        .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                    let (__end_char, end_line, end_col) = position_tracker.advance_to_byte(sentence_end_byte)
                                        .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                    
                                    sentences.push(DialogDetectedSentence {
                                        start_byte: sentence_start_byte,
                                        end_byte: sentence_end_byte,
                                        start_line,
                                        start_col,
                                        end_line,
                                        end_col,
                                    });
                                    
                                    // Next sentence starts after the separator
                                    let next_sentence_start_byte = self.find_sent_sep_end(matched_text)
                                        .map(|sep_end_offset| match_start_byte.advance(sep_end_offset))
                                        .unwrap_or(match_end_byte);
                                    
                                    sentence_start_byte = next_sentence_start_byte;
                                }
                            }
                        }
                    }
                    MatchType::NarrativeGestureBoundary => {
                        // This creates a sentence boundary - record the sentence
                        let sentence_end_byte = self.find_sent_sep_start(matched_text)
                            .map(|sep_offset| match_start_byte.advance(sep_offset))
                            .unwrap_or(match_start_byte);
                        
                        if sentence_end_byte.0 > sentence_start_byte.0 {
                            let content = text[sentence_start_byte.0..sentence_end_byte.0].trim().to_string();
                            if !content.is_empty() {
                                // WHY: Check for abbreviation false positives before creating sentence boundary
                                if self.abbreviation_checker.ends_with_title_abbreviation(&content) {
                                    // This is a false positive - don't create sentence boundary
                                    // Continue processing from current position without advancing sentence_start_byte
                                } else {
                                    // Use incremental position tracker instead of O(N) conversions
                                    let (__start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)
                                        .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                    let (__end_char, end_line, end_col) = position_tracker.advance_to_byte(sentence_end_byte)
                                        .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                    
                                    sentences.push(DialogDetectedSentence {
                                        start_byte: sentence_start_byte,
                                        end_byte: sentence_end_byte,
                                        start_line,
                                        start_col,
                                        end_line,
                                        end_col,
                                    });
                                    
                                    // Next sentence starts after the separator
                                    let next_sentence_start_byte = self.find_sent_sep_end(matched_text)
                                        .map(|sep_end_offset| match_start_byte.advance(sep_end_offset))
                                        .unwrap_or(match_end_byte);
                                    
                                    sentence_start_byte = next_sentence_start_byte;
                                }
                            }
                        }
                    }
                    MatchType::DialogOpen => {
                        // State transition only - no sentence boundary created
                        // Continue from match start (include the opening punctuation in dialog)
                        // No sentence recorded, just state change
                    }
                    MatchType::DialogEnd => {
                        // Dialog end creates a sentence boundary
                        // Use separator logic to find where current sentence ends
                        let sentence_end_byte = self.find_sent_sep_start(matched_text)
                            .map(|sep_offset| match_start_byte.advance(sep_offset))
                            .unwrap_or(match_start_byte);
                        
                        if sentence_end_byte.0 > sentence_start_byte.0 {
                            let content = text[sentence_start_byte.0..sentence_end_byte.0].trim().to_string();
                            if !content.is_empty() {
                                // PHASE 1: Use incremental position tracker instead of O(N) conversions
                                let (_start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)
                                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                let (_end_char, end_line, end_col) = position_tracker.advance_to_byte(sentence_end_byte)
                                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                
                                sentences.push(DialogDetectedSentence {
                                    start_byte: sentence_start_byte,
                                    end_byte: sentence_end_byte,
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
                    MatchType::DialogSoftEnd => {
                        // Soft dialog end - state transition only, no sentence boundary
                        // Continue the current sentence through the dialog close
                        // No sentence recorded, just state change
                    }
                    MatchType::HardSeparator => {
                        // Hard separator - record sentence and transition to Unknown
                        if sentence_start_byte.0 < match_start_byte.0 {
                            // For hard separators, preserve punctuation but trim trailing whitespace carefully
                            let raw_content = &text[sentence_start_byte.0..match_start_byte.0];
                            let content = raw_content.trim_start().trim_end_matches(char::is_whitespace).to_string();
                            if !content.is_empty() {
                                // PHASE 1: Use incremental position tracker instead of O(N) conversions
                                let (_start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)
                                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                let (_end_char, end_line, end_col) = position_tracker.advance_to_byte(match_start_byte)
                                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                
                                sentences.push(DialogDetectedSentence {
                                    start_byte: sentence_start_byte,
                                    end_byte: match_start_byte,
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
                        let (_start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)
                            .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                        let (_end_char, end_line, end_col) = position_tracker.advance_to_byte(BytePos::new(text.len()))
                            .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                        
                        sentences.push(DialogDetectedSentence {
                            start_byte: sentence_start_byte,
                            end_byte: BytePos::new(text.len()),
                            start_line,
                            start_col,
                            end_line,
                            end_col,
                        });
                    }
                }
                remaining_text_handled = true;
                break;
            }
        }
        
        // Handle any remaining text after the loop exits naturally (not via else clause)
        if !remaining_text_handled && sentence_start_byte.0 < text.len() {
            let content = text[sentence_start_byte.0..].trim().to_string();
            if !content.is_empty() {
                // WHY: Create new position tracker for final sentence since main tracker is at end of text
                let mut final_position_tracker = PositionTracker::new(text);
                let (_start_char, start_line, start_col) = final_position_tracker.advance_to_byte(sentence_start_byte)
                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                let (_end_char, end_line, end_col) = final_position_tracker.advance_to_byte(BytePos::new(text.len()))
                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                
                sentences.push(DialogDetectedSentence {
                    start_byte: sentence_start_byte,
                    end_byte: BytePos::new(text.len()),
                    start_line,
                    start_col,
                    end_line,
                    end_col,
                });
            }
        }
        
        info!("Dialog state machine detected {} sentences", sentences.len());
        Ok(sentences)
    }
    
    
    /// Find hard separator position (handles both Unix \n\n and Windows \r\n\r\n)
    fn find_hard_separator(&self, text: &str) -> Option<(usize, usize)> {
        if let Some(pos) = text.find("\r\n\r\n") {
            return Some((pos, 4)); // position and length
        }
        if let Some(pos) = text.find("\n\n") {
            return Some((pos, 2)); // position and length
        }
        None
    }
    
    fn find_sent_sep_start(&self, matched_boundary: &str) -> Option<usize> {
        // Find where SENT_SEP starts within a SENT_END + SENT_SEP + SENT_START match
        // Look for the first whitespace character or hard separator
        if let Some((hard_sep_pos, _)) = self.find_hard_separator(matched_boundary) {
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
        if let Some((hard_sep_pos, sep_len)) = self.find_hard_separator(matched_boundary) {
            return Some(hard_sep_pos + sep_len); // After the separator
        }
        
        // Find the end of whitespace sequence - where SENT_START begins
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

/// Main dialog detector with dual API support
pub struct SentenceDetectorDialog {
    machine: DialogStateMachine,
}

impl SentenceDetectorDialog {
    /// Create new dialog detector 
    pub fn new() -> Result<Self> {
        let machine = DialogStateMachine::new()?;
        Ok(Self { machine })
    }

    /// Detect sentences with borrowed API (zero allocations, mmap-friendly)
    pub fn detect_sentences_borrowed<'a>(&self, text: &'a str) -> Result<Vec<DetectedSentenceBorrowed<'a>>> {
        let dialog_sentences = self.machine.detect_sentences(text)?;
        
        let borrowed_sentences = dialog_sentences
            .into_iter()
            .enumerate()
            .map(|(index, dialog_sentence)| {
                // Use precomputed byte positions for O(1) slice creation
                let start_byte = dialog_sentence.start_byte.0;
                let end_byte = dialog_sentence.end_byte.0;
                let raw_content = &text[start_byte..end_byte];
                
                DetectedSentenceBorrowed {
                    index,
                    raw_content,
                    span: Span {
                        start_line: dialog_sentence.start_line.into(),
                        start_col: dialog_sentence.start_col.into(),
                        end_line: dialog_sentence.end_line.into(),
                        end_col: dialog_sentence.end_col.into(),
                    },
                }
            })
            .collect();
            
        Ok(borrowed_sentences)
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;


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
    
    // WHY: Single shared detector instance reduces test overhead from 38+ instantiations
    static SHARED_DETECTOR: OnceLock<SentenceDetectorDialog> = OnceLock::new();
    
    fn get_detector() -> &'static SentenceDetectorDialog {
        SHARED_DETECTOR.get_or_init(|| SentenceDetectorDialog::new().unwrap())
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
}