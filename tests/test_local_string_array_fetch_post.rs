use pvxs::{NTScalarMetadataBuilder, Server};

#[test]
fn test_pv_local_string_array_fetch_post() {
    let initial_value = "Initial string array element";
    let name = "loc:string:array";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_string(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_string(name).expect("fetch initial");
    assert_eq!(fetched.value, initial_value);

    let test_values = vec![
        "Simple test".to_string(),
        "".to_string(),
        "Special chars: !@#$%^&*()".to_string(),
        "Line\\nbreaks\\nand\\ttabs".to_string(),
        format!("Very long string: {}", "A".repeat(100)),
    ];

    for test_val in test_values {
        server.post_string(name, &test_val).expect("post test value");
        let fetched = server.fetch_string(name).expect("fetch test value");
        assert_eq!(fetched.value, test_val);
    }
}

#[test]
fn test_pv_local_string_array_special_characters() {
    let name = "loc:string:special";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_string(name, "", NTScalarMetadataBuilder::new())
        .expect("create pv");

    let long_string = "A".repeat(1000);
    let special_strings = vec![
        "",
        "   spaces   ",
        "!@#$%^&*()_+-=[]{}|;':\",./<>?",
        "0123456789",
        "Hello world 123",
        "Line1\\nLine2\\tTabbed\\rCarriage",
        r#"Single 'quotes' and "double quotes""#,
        r"Path\to\file\name.txt",
        r#"{"key": "value", "number": 123}"#,
        "<tag>content</tag>",
        &long_string,
    ];

    for test_string in special_strings {
        server
            .post_string(name, test_string)
            .expect("post special string");
        let fetched = server.fetch_string(name).expect("fetch special string");
        assert_eq!(fetched.value, test_string);
    }
}

#[test]
fn test_pv_local_string_array_error_handling() {
    let name = "loc:string:errors";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_string(name, "initial", NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_string(name).expect("fetch initial");
    assert_eq!(fetched.value, "initial");

    server.post_string(name, "updated").expect("post updated");
    let fetched = server.fetch_string(name).expect("fetch updated");
    assert_eq!(fetched.value, "updated");

    for edge in ["", "\\0", "high-unicode-literal"] {
        server.post_string(name, edge).expect("post edge case");
        let fetched = server.fetch_string(name).expect("fetch edge case");
        assert_eq!(fetched.value, edge);
    }

    server.post_string(name, "final").expect("post final");
    let fetched = server.fetch_string(name).expect("fetch final");
    assert_eq!(fetched.value, "final");
}