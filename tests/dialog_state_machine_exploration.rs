use regex_automata::{meta::Regex, Input};
use std::collections::HashMap;

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
    pub start_pos: usize,
    pub end_pos: usize,
    pub content: String,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
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
    
    pub fn detect_sentences(&self, text: &str) -> Vec<DetectedSentence> {
        let mut sentences = Vec::new();
        let mut current_state = DialogState::Narrative;
        let mut sentence_start = 0;
        let mut position = 0;
        
        while position < text.len() {
            let pattern = match self.patterns.get(&current_state) {
                Some(p) => p,
                None => {
                    // Fallback to narrative pattern for unknown states
                    self.patterns.get(&DialogState::Narrative).unwrap()
                }
            };
            
            let input = Input::new(&text[position..]);
            
            if let Some(mat) = pattern.find(input) {
                let match_start = position + mat.start();
                let match_end = position + mat.end();
                
                // Determine exit reason and next state
                let (exit_reason, next_state) = self.determine_exit_reason_and_next_state(
                    &text[match_start..match_end],
                    &text[match_end..],
                    &current_state,
                );
                
                // Record sentence (from sentence_start to start of separator)
                let sentence_end = self.find_sentence_end(&text[sentence_start..match_start]) + sentence_start;
                
                if sentence_end > sentence_start {
                    let content = text[sentence_start..sentence_end].trim().to_string();
                    if !content.is_empty() {
                        let (start_line, start_col) = self.get_line_col(text, sentence_start);
                        let (end_line, end_col) = self.get_line_col(text, sentence_end);
                        
                        sentences.push(DetectedSentence {
                            start_pos: sentence_start,
                            end_pos: sentence_end,
                            content,
                            start_line,
                            start_col,
                            end_line,
                            end_col,
                        });
                    }
                }
                
                // Update position and state
                sentence_start = match_end;
                position = match_end;
                current_state = next_state;
            } else {
                // No more boundaries found, handle remaining text
                if sentence_start < text.len() {
                    let content = text[sentence_start..].trim().to_string();
                    if !content.is_empty() {
                        let (start_line, start_col) = self.get_line_col(text, sentence_start);
                        let (end_line, end_col) = self.get_line_col(text, text.len());
                        
                        sentences.push(DetectedSentence {
                            start_pos: sentence_start,
                            end_pos: text.len(),
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
        
        sentences
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
    
    fn find_sentence_end(&self, text: &str) -> usize {
        // Find the last non-whitespace character
        text.trim_end().len()
    }
    
    fn get_line_col(&self, text: &str, pos: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        
        for (i, ch) in text.char_indices() {
            if i >= pos {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        
        (line, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_narrative_sentences() {
        let machine = DialogStateMachine::new().unwrap();
        let text = "This is a sentence. This is another sentence.";
        let sentences = machine.detect_sentences(text);
        
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
        let sentences = machine.detect_sentences(text);
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].content, "He said, \"Stop her, sir! Ting-a-ling-ling!\"");
        assert_eq!(sentences[1].content, "The headway ran almost out.");
    }
    
    #[test]
    fn test_hard_separator() {
        let machine = DialogStateMachine::new().unwrap();
        let text = "First sentence.\n\nSecond sentence.";
        let sentences = machine.detect_sentences(text);
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].content, "First sentence.");
        assert_eq!(sentences[1].content, "Second sentence.");
    }
    
    #[test]
    fn test_parenthetical_boundaries() {
        let machine = DialogStateMachine::new().unwrap();
        let text = "He left (quietly.) She followed.";
        let sentences = machine.detect_sentences(text);
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].content, "He left (quietly.)");
        assert_eq!(sentences[1].content, "She followed.");
    }
    
    #[test]
    fn test_false_positive_examples() {
        let machine = DialogStateMachine::new().unwrap();
        
        // Example #1: Dialog coalescing
        let text = "The switch hovered in the air—the peril was desperate—\n\n\"My! Look behind you, aunt!\" The old lady whirled round.";
        let sentences = machine.detect_sentences(text);
        
        // Should have 3 sentences: narrative, dialog, narrative
        assert_eq!(sentences.len(), 3);
        assert!(sentences[1].content.contains("My! Look behind you, aunt!"));
        
        // Example #5: Dialog with multiple exclamations
        let text = "He was boat and captain: \"Stop her, sir! Ting-a-ling-ling!\" The headway ran almost out.";
        let sentences = machine.detect_sentences(text);
        
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].content, "He was boat and captain: \"Stop her, sir! Ting-a-ling-ling!\"");
        assert_eq!(sentences[1].content, "The headway ran almost out.");
    }
}