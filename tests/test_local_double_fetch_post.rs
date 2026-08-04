use pvxs::{NTScalarMetadataBuilder, Server};

#[test]
fn test_pv_local_double_fetch_post() {
    let initial_value = std::f64::consts::PI;
    let name = "loc:double";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_double(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_double(name).expect("fetch initial");
    assert_eq!(fetched.value, initial_value);

    assert!(server.post_int32(name, 42).is_err());
    assert!(server.post_string(name, "not_a_number").is_err());

    let fetched = server.fetch_double(name).expect("fetch after invalid posts");
    assert_eq!(fetched.value, initial_value);

    let new_value = std::f64::consts::E;
    server.post_double(name, new_value).expect("post double");
    let fetched = server.fetch_double(name).expect("fetch posted");
    assert_eq!(fetched.value, new_value);
}

#[test]
fn test_pv_local_double_fetch_post_with_error_propagation() {
    let initial_value = 123.456;
    let name = "loc:double:err";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_double(name, initial_value, NTScalarMetadataBuilder::new())
        .expect("create pv");

    assert!(server.post_string(name, "invalid_double").is_err());

    let fetched = server.fetch_double(name).expect("fetch after failed post");
    assert_eq!(fetched.value, initial_value);

    let new_value = 987.654;
    server.post_double(name, new_value).expect("post valid double");
    let fetched = server.fetch_double(name).expect("fetch valid double");
    assert_eq!(fetched.value, new_value);
}

#[test]
fn test_pv_local_double_special_values() {
    let name = "loc:double:special";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_double(name, 0.0, NTScalarMetadataBuilder::new())
        .expect("create pv");

    for value in [f64::INFINITY, f64::NEG_INFINITY, f64::MAX, f64::MIN] {
        server.post_double(name, value).expect("post special double");
        let fetched = server.fetch_double(name).expect("fetch special double");
        assert_eq!(fetched.value, value);
    }

    server.post_double(name, f64::NAN).expect("post NaN");
    let fetched = server.fetch_double(name).expect("fetch NaN");
    assert!(fetched.value.is_nan());
}