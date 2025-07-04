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
    ParenthheticalRound,
    ParenthheticalSquare,
    ParenthheticalCurly,
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
pub enum ExitReason {
    HardSeparator,
    QuoteEnd,
    ParenthheticalEnd,
    NarrativeEnd,
}

pub struct DialogStateMachine {
    patterns: HashMap<DialogState, Regex>,
    quote_starts: Regex,
    paren_starts: Regex,
}

impl DialogStateMachine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut patterns = HashMap::new();
        
        // Pattern components
        let basic_sent_sep = r"\s+";
        let hard_sent_sep = r"\n\n";
        
        // Sentence endings by context
        let narrative_sent_end = r"[.!?]";
        let double_quote_sent_end = r"[.!?]\x22";
        let single_quote_sent_end = r"[.!?]\x27";
        let smart_double_sent_end = r"[.!?]\u{201D}";
        let smart_single_sent_end = r"[.!?]\u{2019}";
        let paren_round_sent_end = r"[.!?]\)";
        let paren_square_sent_end = r"[.!?]\]";
        let paren_curly_sent_end = r"[.!?]\}";
        
        // Sentence starts
        let basic_start = r"(?:[A-Z]|[\x22\x27\u{201C}\u{2018}]|[\(\[\{])";
        
        // Build state-specific patterns
        let narrative_pattern = format!(
            "(?:{}{}{})|(?:{})",
            narrative_sent_end, basic_sent_sep, basic_start, hard_sent_sep
        );
        
        let dialog_double_pattern = format!(
            "(?:{}{}{})|(?:{})",
            double_quote_sent_end, basic_sent_sep, basic_start, hard_sent_sep
        );
        
        let dialog_single_pattern = format!(
            "(?:{}{}{})|(?:{})",
            single_quote_sent_end, basic_sent_sep, basic_start, hard_sent_sep
        );
        
        let dialog_smart_double_pattern = format!(
            "(?:{}{}{})|(?:{})",
            smart_double_sent_end, basic_sent_sep, basic_start, hard_sent_sep
        );
        
        let dialog_smart_single_pattern = format!(
            "(?:{}{}{})|(?:{})",
            smart_single_sent_end, basic_sent_sep, basic_start, hard_sent_sep
        );
        
        let paren_round_pattern = format!(
            "(?:{}{}{})|(?:{})",
            paren_round_sent_end, basic_sent_sep, basic_start, hard_sent_sep
        );
        
        let paren_square_pattern = format!(
            "(?:{}{}{})|(?:{})",
            paren_square_sent_end, basic_sent_sep, basic_start, hard_sent_sep
        );
        
        let paren_curly_pattern = format!(
            "(?:{}{}{})|(?:{})",
            paren_curly_sent_end, basic_sent_sep, basic_start, hard_sent_sep
        );
        
        // Compile patterns
        patterns.insert(DialogState::Narrative, Regex::new(&narrative_pattern)?);
        patterns.insert(DialogState::DialogDoubleQuote, Regex::new(&dialog_double_pattern)?);
        patterns.insert(DialogState::DialogSingleQuote, Regex::new(&dialog_single_pattern)?);
        patterns.insert(DialogState::DialogSmartDoubleOpen, Regex::new(&dialog_smart_double_pattern)?);
        patterns.insert(DialogState::DialogSmartSingleOpen, Regex::new(&dialog_smart_single_pattern)?);
        patterns.insert(DialogState::ParenthheticalRound, Regex::new(&paren_round_pattern)?);
        patterns.insert(DialogState::ParenthheticalSquare, Regex::new(&paren_square_pattern)?);
        patterns.insert(DialogState::ParenthheticalCurly, Regex::new(&paren_curly_pattern)?);
        
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
                
                // Determine exit reason and next state
                let (_exit_reason, next_state) = self.determine_exit_reason_and_next_state(
                    &text[match_start_byte.0..match_end_byte.0],
                    &text[match_end_byte.0..],
                    &current_state,
                );
                
                // Sentence end is BEFORE the current SENT_SEP
                // The match includes SENT_END + SENT_SEP + SENT_START
                // We need to find where SENT_SEP starts within the match
                let sentence_end_byte = self.find_sent_sep_start(&text[match_start_byte.0..match_end_byte.0])
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
                
                // Next sentence start is AFTER the current SENT_SEP
                // Find where SENT_SEP ends within the match
                let next_sentence_start_byte = self.find_sent_sep_end(&text[match_start_byte.0..match_end_byte.0])
                    .map(|sep_end_offset| match_start_byte.advance(sep_end_offset))
                    .unwrap_or(match_end_byte);
                
                // Update position and state
                sentence_start_byte = next_sentence_start_byte;
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
    
    fn determine_exit_reason_and_next_state(
        &self,
        matched_text: &str,
        following_text: &str,
        current_state: &DialogState,
    ) -> (ExitReason, DialogState) {
        // Check for hard separator
        if matched_text.contains("\n\n") {
            return (ExitReason::HardSeparator, DialogState::Unknown);
        }
        
        // Check for quote endings
        if matched_text.contains('"') && matched_text.contains("[.!?]") {
            return (ExitReason::QuoteEnd, DialogState::Narrative);
        }
        if matched_text.contains("'") && matched_text.contains("[.!?]") {
            return (ExitReason::QuoteEnd, DialogState::Narrative);
        }
        if matched_text.contains('\u{201D}') {
            return (ExitReason::QuoteEnd, DialogState::Narrative);
        }
        if matched_text.contains('\u{2019}') {
            return (ExitReason::QuoteEnd, DialogState::Narrative);
        }
        
        // Check for parenthetical endings
        if matched_text.contains(')') && matched_text.contains("[.!?]") {
            return (ExitReason::ParenthheticalEnd, DialogState::Narrative);
        }
        if matched_text.contains(']') && matched_text.contains("[.!?]") {
            return (ExitReason::ParenthheticalEnd, DialogState::Narrative);
        }
        if matched_text.contains('}') && matched_text.contains("[.!?]") {
            return (ExitReason::ParenthheticalEnd, DialogState::Narrative);
        }
        
        // Handle narrative transitions by checking following text
        match current_state {
            DialogState::Narrative => {
                // Check for quote starts
                if let Some(quote_match) = self.quote_starts.find(Input::new(following_text)) {
                    let quote_char = &following_text[quote_match.start()..quote_match.end()];
                    match quote_char {
                        "\"" => return (ExitReason::NarrativeEnd, DialogState::DialogDoubleQuote),
                        "'" => return (ExitReason::NarrativeEnd, DialogState::DialogSingleQuote),
                        "\u{201C}" => return (ExitReason::NarrativeEnd, DialogState::DialogSmartDoubleOpen),
                        "\u{2018}" => return (ExitReason::NarrativeEnd, DialogState::DialogSmartSingleOpen),
                        _ => {}
                    }
                }
                
                // Check for parenthetical starts
                if let Some(paren_match) = self.paren_starts.find(Input::new(following_text)) {
                    let paren_char = &following_text[paren_match.start()..paren_match.end()];
                    match paren_char {
                        "(" => return (ExitReason::NarrativeEnd, DialogState::ParenthheticalRound),
                        "[" => return (ExitReason::NarrativeEnd, DialogState::ParenthheticalSquare),
                        "{" => return (ExitReason::NarrativeEnd, DialogState::ParenthheticalCurly),
                        _ => {}
                    }
                }
                
                (ExitReason::NarrativeEnd, DialogState::Narrative)
            }
            DialogState::Unknown => {
                // Determine next state based on context
                self.determine_state_from_context(following_text)
            }
            _ => {
                // Continue in current state for dialog/parenthetical contexts
                (ExitReason::NarrativeEnd, current_state.clone())
            }
        }
    }
    
    fn determine_state_from_context(&self, text: &str) -> (ExitReason, DialogState) {
        // Check for quote starts
        if let Some(quote_match) = self.quote_starts.find(Input::new(text)) {
            let quote_char = &text[quote_match.start()..quote_match.end()];
            match quote_char {
                "\"" => return (ExitReason::NarrativeEnd, DialogState::DialogDoubleQuote),
                "'" => return (ExitReason::NarrativeEnd, DialogState::DialogSingleQuote),
                "\u{201C}" => return (ExitReason::NarrativeEnd, DialogState::DialogSmartDoubleOpen),
                "\u{2018}" => return (ExitReason::NarrativeEnd, DialogState::DialogSmartSingleOpen),
                _ => {}
            }
        }
        
        // Check for parenthetical starts
        if let Some(paren_match) = self.paren_starts.find(Input::new(text)) {
            let paren_char = &text[paren_match.start()..paren_match.end()];
            match paren_char {
                "(" => return (ExitReason::NarrativeEnd, DialogState::ParenthheticalRound),
                "[" => return (ExitReason::NarrativeEnd, DialogState::ParenthheticalSquare),
                "{" => return (ExitReason::NarrativeEnd, DialogState::ParenthheticalCurly),
                _ => {}
            }
        }
        
        // Default to narrative
        (ExitReason::NarrativeEnd, DialogState::Narrative)
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