use regex::Regex;
use crate::ds::KeyValue;

fn map(filename: String, contents: String) -> Vec<KeyValue> { 
    let seperator = Regex::new(r"([ \n]+)").expect("Invalid regex");
    let splits: Vec<_> = seperator.split(&contents).into_iter().collect();
    let mut kva: Vec<KeyValue> = vec!();
    for word in splits {
        kva.push(KeyValue::new(word.to_string(), "1".to_string()))
    }
    kva
} 

fn reduce(key: String, values: Vec<String>) -> String { 
    values.len().to_string()
} 
