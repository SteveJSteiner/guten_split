use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Rule {
    id: String,
    description: String,
    end_chars: Vec<String>,
    separators: Vec<String>,
    start_chars: Vec<String>,
    context_template: String,
    expected_match_type: String,
    expected_next_state: String,
    creates_sentence_boundary: bool,
    validated: bool,
    #[serde(default)]
    current_state: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CategoryRules {
    category: String,
    description: String,
    rules: Vec<Rule>,
    #[serde(default)]
    negative_rules: Option<Vec<Rule>>,
}

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

fn expand_rule(rule: &Rule, category: &str) -> Vec<GeneratedTestCase> {
    let mut test_cases = Vec::new();
    
    // Handle empty arrays for dialog opens/ends that don't use all pattern parts
    let end_chars: Vec<&str> = if rule.end_chars.is_empty() { 
        vec![""] 
    } else { 
        rule.end_chars.iter().map(|s| s.as_str()).collect() 
    };
    
    let separators: Vec<&str> = if rule.separators.is_empty() { 
        vec![""] 
    } else { 
        rule.separators.iter().map(|s| s.as_str()).collect() 
    };
    
    let start_chars: Vec<&str> = rule.start_chars.iter().map(|s| s.as_str()).collect();
    
    // Generate all combinations
    for &end_char in &end_chars {
        for &separator in &separators {
            for &start_char in &start_chars {
                let pattern = format!("{end_char}{separator}{start_char}");
                
                // Skip empty patterns
                if pattern.trim().is_empty() {
                    continue;
                }
                
                // Expand template
                let full_text = rule.context_template
                    .replace("{end}", end_char)
                    .replace("{sep}", separator)
                    .replace("{start}", start_char);
                
                // Find pattern position in full text for context extraction
                let pattern_pos = full_text.find(&pattern).unwrap_or(0);
                let context_before = full_text[..pattern_pos].to_string();
                let context_after = if pattern_pos + pattern.len() < full_text.len() {
                    full_text[pattern_pos + pattern.len()..].to_string()
                } else {
                    String::new()
                };
                
                // Generate readable test case ID
                let end_id = if end_char.is_empty() { "none" } else { 
                    match end_char {
                        "." => "period",
                        "!" => "excl", 
                        "?" => "quest",
                        "\"" => "dquote",
                        "'" => "squote",
                        _ => "other",
                    }
                };
                
                let sep_id = if separator.is_empty() { "none" } else {
                    match separator {
                        " " => "space",
                        "\t" => "tab", 
                        "\n" => "newline",
                        "\n\n" => "dnewline",
                        _ => "complex",
                    }
                };
                
                let start_id = match start_char {
                    "A" | "B" | "H" | "S" => "upper",
                    "\"" => "dquote",
                    "'" => "squote",
                    "(" => "lparen",
                    ")" => "rparen",
                    "[" => "lbracket",
                    "]" => "rbracket",
                    "{" => "lbrace",
                    "}" => "rbrace",
                    "a" | "b" | "c" => "lower",
                    "1" | "2" | "9" => "digit",
                    _ if start_char.chars().next().unwrap() as u32 > 127 => "unicode",
                    _ => "other",
                };
                
                let test_id = format!("{}_{}_{}_{}_{}",
                    category,
                    rule.id,
                    end_id,
                    sep_id,
                    start_id
                );
                
                test_cases.push(GeneratedTestCase {
                    id: test_id,
                    pattern: pattern.clone(),
                    current_state: rule.current_state.as_ref().unwrap_or(&"Narrative".to_string()).clone(),
                    full_text,
                    context_before,
                    context_after,
                    expected_match_type: rule.expected_match_type.clone(),
                    expected_next_state: rule.expected_next_state.clone(),
                    creates_sentence_boundary: rule.creates_sentence_boundary,
                    validated: rule.validated,
                    source_rule: rule.id.clone(),
                    source_category: category.to_string(),
                    notes: format!("{} | Pattern: '{}'", rule.description, pattern),
                });
            }
        }
    }
    
    test_cases
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rule_dir = "tests/boundary_rules";
    let mut all_test_cases = Vec::new();
    let mut source_files = Vec::new();
    
    // Process all rule files in the directory
    for entry in fs::read_dir(rule_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") && 
           path.file_name().and_then(|s| s.to_str()) != Some("known_issues.json") {
            
            let filename = path.file_name().unwrap().to_str().unwrap();
            source_files.push(filename.to_string());
            
            println!("Processing rule file: {filename}");
            
            let json_content = fs::read_to_string(&path)?;
            let category_rules: CategoryRules = serde_json::from_str(&json_content)?;
            
            // Expand positive rules
            for rule in &category_rules.rules {
                let mut test_cases = expand_rule(rule, &category_rules.category);
                all_test_cases.append(&mut test_cases);
            }
            
            // Expand negative rules if they exist
            if let Some(negative_rules) = &category_rules.negative_rules {
                for rule in negative_rules {
                    let mut test_cases = expand_rule(rule, &format!("{}_negative", category_rules.category));
                    all_test_cases.append(&mut test_cases);
                }
            }
        }
    }
    
    // Create output data structure
    let generated_data = GeneratedTestData {
        schema_version: "2.0".to_string(),
        description: "Test cases generated from production rules in boundary_rules/ directory".to_string(),
        generated_from: source_files,
        test_cases: all_test_cases,
    };
    
    // Write to output file (allow override via environment variable)
    let output_path = std::env::var("BOUNDARY_TEST_OUTPUT")
        .unwrap_or_else(|_| "tests/generated_boundary_tests.json".to_string());
    let json_string = serde_json::to_string_pretty(&generated_data)?;
    fs::write(&output_path, json_string)?;
    
    println!("Generated {} test cases from {} rule files", 
        generated_data.test_cases.len(), 
        generated_data.generated_from.len());
    println!("Output written to: {output_path}");
    
    // Print breakdown by category
    let mut category_counts = std::collections::HashMap::new();
    for test_case in &generated_data.test_cases {
        *category_counts.entry(&test_case.source_category).or_insert(0) += 1;
    }
    
    println!("\nBreakdown by category:");
    for (category, count) in category_counts {
        println!("  {category}: {count} test cases");
    }
    
    Ok(())
}