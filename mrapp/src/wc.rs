use plugins_core::ds::KeyValue;
use regex::Regex;

pub fn mapf(filename: String, contents: String) -> Vec<KeyValue> { 
    let re = Regex::new(r"![a-zA-Z]").expect("Invalid regex");
    let contents = re
        .replace_all(&contents, "")
        .to_string()
        .to_ascii_uppercase();
    let seperator = Regex::new(r"([ \n]+)").expect("Invalid regex");
    let splits: Vec<_> = seperator
        .split(&contents)
        .into_iter()
        .collect();
    let mut kva: Vec<KeyValue> = vec!();
    for word in splits {
        kva.push(KeyValue::new(word.to_string(), "1".to_string()))
    }
    kva
} 

pub fn reducef(key: String, values: Vec<String>) -> String { 
    values.len().to_string()
} 
