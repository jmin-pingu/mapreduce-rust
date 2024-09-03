use mr::ds::Intermediate;

#[test]
fn test_intermediate() {
    let mut intermediate = Intermediate::new();
    assert_eq!(intermediate.get("foo".to_string()), None);
    intermediate.insert("foo".to_string(), "bar0".to_string());
    assert_eq!(intermediate.get("foo".to_string()).unwrap()[0], "bar0".to_string());

    intermediate.insert("foo".to_string(), "bar1".to_string());
    intermediate.insert("foo".to_string(), "bar2".to_string());
    intermediate.insert("foo".to_string(), "bar3".to_string());
    assert_eq!(intermediate.get("foo".to_string()).unwrap()[1], "bar1".to_string());
    assert_eq!(intermediate.get("foo".to_string()).unwrap()[3], "bar3".to_string());
    assert_eq!(intermediate.get("foo".to_string()).unwrap().len(), 4);
}

#[test]
fn test1() { }

#[test]
fn test2() { }

#[test]
fn test3() { }

#[test]
fn test4() { }

#[test]
fn test5() { }

#[test]
fn test6() { }
