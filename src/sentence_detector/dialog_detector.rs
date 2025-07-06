// WHY: Dialog-aware sentence detection with state machine for coalescing and sophisticated boundaries
// Moved from tests to official implementation with dual API support

use anyhow::Result;
use regex_automata::{meta::Regex, Input};
use std::collections::HashMap;
use tracing::{debug, info};

use super::{DetectedSentence, DetectedSentenceBorrowed, DetectedSentenceOwned, Span, AbbreviationChecker};

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
    
    /// Get current position without advancing
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

/// Internal representation for dialog state machine
#[derive(Debug, Clone)]
pub struct DialogDetectedSentence {
    pub start_pos: CharPos,
    pub end_pos: CharPos,
    pub start_byte: BytePos,  // Added for O(1) borrowed API
    pub end_byte: BytePos,    // Added for O(1) borrowed API
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
    DialogSoftEnd,
    HardSeparator,
}

pub struct DialogStateMachine {
    patterns: HashMap<DialogState, Regex>,
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
                    
                    // Closing delimiters - skip and continue looking for meaningful punctuation
                    b'"' | b'\'' | b')' | b']' | b'}' => continue,
                    
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
                            
                            // Smart closing quotes - skip and continue looking
                            '\u{201D}' | '\u{2019}' => continue,
                            
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

    pub fn new() -> Result<Self> {
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
        
        Ok(DialogStateMachine {
            patterns,
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
                
                let (match_type, next_state) = self.classify_match(matched_text, &current_state, text.as_bytes(), match_start_byte.0);
                
                match match_type {
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
                                    // PHASE 1: Use incremental position tracker instead of O(N) conversions
                                    let (start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)
                                        .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                    let (end_char, end_line, end_col) = position_tracker.advance_to_byte(sentence_end_byte)
                                        .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                    
                                    sentences.push(DialogDetectedSentence {
                                        start_pos: start_char,
                                        end_pos: end_char,
                                        start_byte: sentence_start_byte,
                                        end_byte: sentence_end_byte,
                                        content,
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
                        // Use special logic for dialog endings to include closing quotes
                        let sentence_end_byte = self.find_dialog_sent_end(matched_text)
                            .map(|sep_offset| match_start_byte.advance(sep_offset))
                            .unwrap_or(match_start_byte);
                        
                        if sentence_end_byte.0 > sentence_start_byte.0 {
                            let content = text[sentence_start_byte.0..sentence_end_byte.0].trim().to_string();
                            if !content.is_empty() {
                                // PHASE 1: Use incremental position tracker instead of O(N) conversions
                                let (start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)
                                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                let (end_char, end_line, end_col) = position_tracker.advance_to_byte(sentence_end_byte)
                                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                
                                sentences.push(DialogDetectedSentence {
                                    start_pos: start_char,
                                    end_pos: end_char,
                                    start_byte: sentence_start_byte,
                                    end_byte: sentence_end_byte,
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
                                let (start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)
                                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                let (end_char, end_line, end_col) = position_tracker.advance_to_byte(match_start_byte)
                                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                                
                                sentences.push(DialogDetectedSentence {
                                    start_pos: start_char,
                                    end_pos: end_char,
                                    start_byte: sentence_start_byte,
                                    end_byte: match_start_byte,
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
                        let (start_char, start_line, start_col) = position_tracker.advance_to_byte(sentence_start_byte)
                            .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                        let (end_char, end_line, end_col) = position_tracker.advance_to_byte(BytePos::new(text.len()))
                            .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                        
                        sentences.push(DialogDetectedSentence {
                            start_pos: start_char,
                            end_pos: end_char,
                            start_byte: sentence_start_byte,
                            end_byte: BytePos::new(text.len()),
                            content,
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
                let (start_char, start_line, start_col) = final_position_tracker.advance_to_byte(sentence_start_byte)
                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                let (end_char, end_line, end_col) = final_position_tracker.advance_to_byte(BytePos::new(text.len()))
                    .map_err(|e| anyhow::anyhow!("Position tracking error: {}", e))?;
                
                sentences.push(DialogDetectedSentence {
                    start_pos: start_char,
                    end_pos: end_char,
                    start_byte: sentence_start_byte,
                    end_byte: BytePos::new(text.len()),
                    content,
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
    
    fn classify_match(&self, matched_text: &str, current_state: &DialogState, text_bytes: &[u8], match_start_byte: usize) -> (MatchType, DialogState) {
        // Check for pure hard separator (exactly \n\n)
        if matched_text == "\n\n" {
            // WHY: Check if this hard separator should be rejected due to preceding internal punctuation
            // This implements the core dialog coalescing logic for internal punctuation
            if self.should_reject_hard_separator(text_bytes, match_start_byte) {
                // Reject this hard separator - treat as non-boundary, continue current state
                // Keep the current state to maintain continuity across the rejected separator
                return (MatchType::DialogSoftEnd, current_state.clone());
            }
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
            // Return DialogSoftEnd and transition to Narrative but don't create sentence boundary
            (MatchType::DialogSoftEnd, DialogState::Narrative)
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
    
    fn find_dialog_sent_end(&self, matched_boundary: &str) -> Option<usize> {
        // For dialog endings, include the closing quote in the sentence content
        // Unlike find_sent_sep_start which finds separator start, this finds sentence content end
        
        if let Some(hard_sep_pos) = matched_boundary.find("\n\n") {
            return Some(hard_sep_pos);
        }
        
        // Find the closing quote and include it in the sentence
        let closing_quotes = ["\"", "'", "\u{201D}", "\u{2019}", ")", "]", "}"];
        
        for quote in &closing_quotes {
            if let Some(quote_pos) = matched_boundary.find(quote) {
                // Return position after the quote to include it in sentence content
                return Some(quote_pos + quote.len());
            }
        }
        
        // Fallback to original logic if no quotes found
        self.find_sent_sep_start(matched_boundary)
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

    /// Detect sentences with owned API (async I/O-friendly)
    pub fn detect_sentences_owned(&self, text: &str) -> Result<Vec<DetectedSentenceOwned>> {
        let borrowed_sentences = self.detect_sentences_borrowed(text)?;
        
        let owned_sentences = borrowed_sentences
            .into_iter()
            .map(|borrowed| DetectedSentenceOwned {
                index: borrowed.index,
                raw_content: borrowed.raw_content.to_string(),
                span: borrowed.span,
            })
            .collect();
            
        Ok(owned_sentences)
    }

    /// Legacy API for backward compatibility
    pub fn detect_sentences(&self, text: &str) -> Result<Vec<DetectedSentence>> {
        let borrowed_sentences = self.detect_sentences_borrowed(text)?;
        
        let legacy_sentences = borrowed_sentences
            .into_iter()
            .map(|borrowed| DetectedSentence {
                index: borrowed.index,
                normalized_content: borrowed.normalize(),
                span: borrowed.span,
            })
            .collect();
            
        Ok(legacy_sentences)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_narrative_sentences() {
        let detector = SentenceDetectorDialog::new().unwrap();
        let text = "This is a sentence. This is another sentence.";
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].raw().contains("This is a sentence"));
        assert!(sentences[1].raw().contains("This is another sentence"));
    }

    #[test]
    fn test_dialog_coalescing() {
        let detector = SentenceDetectorDialog::new().unwrap();
        let text = "He said, \"Stop her, sir! Ting-a-ling-ling!\" The headway ran almost out.";
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].raw().contains("Stop her, sir! Ting-a-ling-ling!"));
        assert!(sentences[1].raw().contains("The headway ran almost out"));
    }

    #[test]
    fn test_dual_api_equivalence() {
        let detector = SentenceDetectorDialog::new().unwrap();
        let text = "Hello world. This is a test. How are you?";
        
        let borrowed_result = detector.detect_sentences_borrowed(text).unwrap();
        let owned_result = detector.detect_sentences_owned(text).unwrap();
        
        assert_eq!(borrowed_result.len(), owned_result.len());
        
        for (borrowed, owned) in borrowed_result.iter().zip(owned_result.iter()) {
            assert_eq!(borrowed.index, owned.index);
            assert_eq!(borrowed.raw(), owned.raw());
            assert_eq!(borrowed.span, owned.span);
        }
    }

    #[test]
    fn test_abbreviation_handling() {
        let detector = SentenceDetectorDialog::new().unwrap();
        
        // Test case from task: "Dr. Smith" should not be split
        let text = "Dr. Smith examined the patient. The results were clear.";
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].raw().contains("Dr. Smith examined the patient"));
        assert!(sentences[1].raw().contains("The results were clear"));
        assert!(!sentences[0].raw().trim().ends_with("Dr."));
    }

    #[test]
    fn test_multiple_title_abbreviations() {
        let detector = SentenceDetectorDialog::new().unwrap();
        
        // Test multiple title abbreviations
        let text = "Mr. and Mrs. Johnson arrived. They were late.";
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].raw().contains("Mr. and Mrs. Johnson arrived"));
        assert!(sentences[1].raw().contains("They were late"));
    }

    #[test]
    fn test_geographic_abbreviations() {
        let detector = SentenceDetectorDialog::new().unwrap();
        
        // Test geographic abbreviations
        let text = "The U.S.A. declared independence. It was 1776.";
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].raw().contains("The U.S.A. declared independence"));
        assert!(sentences[1].raw().contains("It was 1776"));
    }

    #[test]
    fn test_measurement_abbreviations() {
        let detector = SentenceDetectorDialog::new().unwrap();
        
        // Test measurement abbreviations
        let text = "Distance is 2.5 mi. from here. We can walk it.";
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].raw().contains("Distance is 2.5 mi. from here"));
        assert!(sentences[1].raw().contains("We can walk it"));
    }

    #[test]
    fn test_dialog_with_abbreviations() {
        let detector = SentenceDetectorDialog::new().unwrap();
        
        // Test dialog with abbreviations - this is a more complex case
        let text = "He said, 'Dr. Smith will see you.' She nodded.";
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        
        // Should coalesce the dialog and treat "Dr. Smith" as one unit
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].raw().contains("Dr. Smith will see you"));
        assert!(sentences[1].raw().contains("She nodded"));
    }

    #[test]
    fn test_soft_dialog_transitions() {
        let detector = SentenceDetectorDialog::new().unwrap();
        
        // Test case 1: comma + quote should soft transition, continue sentence
        let text = "\"Hello,\" she said quietly.";
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        // Should be one sentence - soft transition should continue
        assert_eq!(sentences.len(), 1, "Soft transition with comma should continue sentence");
        assert!(sentences[0].raw().contains("Hello") && sentences[0].raw().contains("she said"));
        
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
        let detector = SentenceDetectorDialog::new().unwrap();
        
        // Test case: exclamation + space + capital should hard transition, create boundary
        let text = "\"Wait!\" he shouted loudly. Then he left.";
        let sentences = detector.detect_sentences_borrowed(text).unwrap();
        // Should be two sentences - hard transition should create boundary
        assert_eq!(sentences.len(), 2, "Hard transition should create sentence boundary");
        assert!(sentences[0].raw().contains("Wait!") && sentences[0].raw().contains("he shouted"));
        assert!(sentences[1].raw().contains("Then he left"));
    }
}