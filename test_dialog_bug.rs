use anyhow::Result;
use crate::sentence_detector::dialog_detector::SentenceDetectorDialog;

fn main() -> Result<()> {
    // Extract the exact problematic text from the gutenberg file
    let test_text = r#"in the cabinets of the curious. For more than a century after Belon's
work, the birds themselves had not been seen in England; for, in the
_Museum Tradescantianum_, the standard collection of the time, and
which, from the list of contributors, appears to have been the great
receptacle for all curiosities, we read of an "Azacari (or Toucan) of
Brazil; has his beak four inches long, almost two thick, like a Turk's
sword" (A.D. 1656). From this description Tradescant knew the nature of
the bird, if he had not seen it.

Mr. Swainson states, that the enormous bills give to these birds a most"#;

    let detector = SentenceDetectorDialog::new()?;
    let sentences = detector.detect_sentences_borrowed(test_text)?;
    
    println!("Detected {} sentences:", sentences.len());
    for (i, sentence) in sentences.iter().enumerate() {
        println!("{}: {:?}", i, sentence.raw_content.trim());
    }
    
    // Expected: Should detect sentence boundary at 'sword" (A.D. 1656).' 
    // and separate "From this description..." as a new sentence
    
    Ok(())
}