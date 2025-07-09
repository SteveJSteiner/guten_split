// WHY: standalone normalization logic enabling zero-allocation batch processing
// Separates sentence detection from normalization for fair performance comparisons

/// Normalize sentence by removing interior hard line breaks and collapsing whitespace
/// Implements F-6: normalize sentences by removing hard line breaks, treat \r\n as single break
pub fn normalize_sentence(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    normalize_sentence_into(text, &mut result);
    result
}

/// Normalize sentence into supplied buffer to avoid allocation
/// WHY: enables buffer reuse in batch processing scenarios
pub fn normalize_sentence_into(text: &str, buffer: &mut String) {
    buffer.clear();
    buffer.reserve(text.len()); // Ensure capacity for worst case
    
    let mut chars = text.chars().peekable();
    let mut prev_was_space = false;
    
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                // Handle \r\n as single break (peek ahead for \n)
                if chars.peek() == Some(&'\n') {
                    chars.next(); // consume the \n
                }
                // Replace with single space, but don't add if previous was space
                if !prev_was_space {
                    buffer.push(' ');
                    prev_was_space = true;
                }
            }
            '\n' => {
                // Replace with single space, but don't add if previous was space
                if !prev_was_space {
                    buffer.push(' ');
                    prev_was_space = true;
                }
            }
            _ if ch.is_whitespace() => {
                // Collapse any whitespace into single space
                if !prev_was_space {
                    buffer.push(' ');
                    prev_was_space = true;
                }
            }
            _ => {
                // WHY: preserve all other bytes as specified in F-6
                buffer.push(ch);
                prev_was_space = false;
            }
        }
    }
    
    // WHY: trim only leading/trailing whitespace, preserve interior structure
    let trimmed = buffer.trim();
    if trimmed.len() != buffer.len() {
        // Need to create new string with trimmed content
        let trimmed_content = trimmed.to_string();
        buffer.clear();
        buffer.push_str(&trimmed_content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_sentence_basic() {
        let input = "This is a\nsentence with\r\nline breaks.";
        let expected = "This is a sentence with line breaks.";
        assert_eq!(normalize_sentence(input), expected);
    }

    #[test]
    fn test_normalize_sentence_into_buffer_reuse() {
        let mut buffer = String::new();
        
        // First normalization
        normalize_sentence_into("Line one.\nLine two.", &mut buffer);
        assert_eq!(buffer, "Line one. Line two.");
        
        // Buffer reuse - should clear and reuse
        normalize_sentence_into("Different\r\ncontent.", &mut buffer);
        assert_eq!(buffer, "Different content.");
    }

    #[test]
    fn test_normalize_sentence_whitespace_collapse() {
        let input = "Multiple\n\n\nspaces\r\n\r\n   here.";
        let result = normalize_sentence(input);
        assert_eq!(result, "Multiple spaces here.");
    }

    #[test]
    fn test_normalize_sentence_preserve_interior() {
        let input = "  Leading and trailing  ";
        let result = normalize_sentence(input);
        assert_eq!(result, "Leading and trailing");
    }

    #[test]
    fn test_normalize_sentence_empty() {
        assert_eq!(normalize_sentence(""), "");
        assert_eq!(normalize_sentence("   "), "");
    }

    #[test]
    fn test_normalize_sentence_unicode() {
        let input = "Unicode\n世界\r\nwith émojis 🦀.";
        let expected = "Unicode 世界 with émojis 🦀.";
        assert_eq!(normalize_sentence(input), expected);
    }

    #[test]
    fn test_normalize_sentence_tabs() {
        let input = "Text\twith\ttabs\there.";
        let expected = "Text with tabs here.";
        assert_eq!(normalize_sentence(input), expected);
    }

    #[test]
    fn test_normalize_sentence_mixed_whitespace() {
        let input = "Mixed\t\n\twhitespace\r\n\there.";
        let expected = "Mixed whitespace here.";
        assert_eq!(normalize_sentence(input), expected);
    }

    #[test]
    fn test_normalize_sentence_consecutive_tabs() {
        let input = "Multiple\t\t\tconsecutive\ttabs.";
        let expected = "Multiple consecutive tabs.";
        assert_eq!(normalize_sentence(input), expected);
    }
}