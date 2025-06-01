use std::collections::HashMap;
use crate::api::api::{AccessErrorStructure, NumOrString};

#[test]
fn error_parse() {
    let test_1 = "{\"error\": { \"code\": 6, \"error\": \"Incorrect ID\" } }";

    let returned = serde_json::from_str::<AccessErrorStructure>(test_1).unwrap();
    assert_eq!(returned, AccessErrorStructure{
        error: HashMap::from([
            ("error".to_string(), NumOrString::String("Incorrect ID".to_string())),
            ("code".to_string(), NumOrString::Num(6)),
        ])
    })
}