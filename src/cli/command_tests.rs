use crate::cli::command::{Command, ReadSpec};

#[test]
fn parse_read_default_is_raw_entire_file() {
    let cmd = Command::parse(&vec!["read".to_string()]).unwrap();
    match cmd {
        Command::READ { spec, pretty } => {
            assert!(spec.is_none());
            assert!(!pretty);
        }
        _ => panic!("expected READ"),
    }
}

#[test]
fn parse_read_pretty_entire_file() {
    let cmd = Command::parse(&vec!["read".to_string(), "--pretty".to_string()]).unwrap();
    match cmd {
        Command::READ { spec, pretty } => {
            assert!(spec.is_none());
            assert!(pretty);
        }
        _ => panic!("expected READ"),
    }
}

#[test]
fn parse_read_pretty_count_from_end() {
    let cmd = Command::parse(&vec![
        "read".to_string(),
        "--pretty".to_string(),
        "2".to_string(),
    ])
    .unwrap();
    match cmd {
        Command::READ { spec, pretty } => {
            assert!(pretty);
            assert!(matches!(spec, Some(ReadSpec::CountFromEnd(2))));
        }
        _ => panic!("expected READ"),
    }
}

#[test]
fn parse_read_pretty_index_from_end() {
    let cmd = Command::parse(&vec![
        "read".to_string(),
        "--pretty".to_string(),
        "-2".to_string(),
    ])
    .unwrap();

    match cmd {
        Command::READ { spec, pretty } => {
            assert!(pretty);
            assert!(matches!(spec, Some(ReadSpec::IndexFromEnd(2))));
        }
        _ => panic!("expected READ"),
    }
}
