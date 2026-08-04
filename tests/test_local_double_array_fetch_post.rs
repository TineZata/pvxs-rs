// Copyright 2026 Tine Zata
// SPDX-License-Identifier: MPL-2.0
use pvxs::{NTScalarMetadataBuilder, Server};

#[test]
fn test_pv_local_double_array_fetch_post() {
    let initial_array = vec![std::f64::consts::PI, std::f64::consts::E, 1.61803];
    let name = "loc:double:array";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_double_array(name, initial_array.clone(), NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_double_array(name).expect("fetch initial");
    assert_eq!(fetched.value, initial_array);

    let test_values = vec![0.0, -1.5, std::f64::consts::E, 1e-10, 1e10];
    server
        .post_double_array(name, test_values.clone())
        .expect("post double array");
    let fetched = server.fetch_double_array(name).expect("fetch posted");
    assert_eq!(fetched.value, test_values);
}

#[test]
fn test_pv_local_double_array_special_values() {
    let name = "loc:double:special";
    let server = Server::start_isolated().expect("start isolated server");

    let mut special_values = vec![
        0.0,
        -0.0,
        std::f64::consts::PI,
        std::f64::consts::E,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        1e-308,
        1e308,
    ];

    server
        .create_pv_double_array(name, special_values.clone(), NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_double_array(name).expect("fetch initial");
    assert_eq!(fetched.value, special_values);

    special_values.push(f64::INFINITY);
    special_values.push(f64::NEG_INFINITY);
    special_values.push(f64::NAN);

    server
        .post_double_array(name, special_values.clone())
        .expect("post special array");
    let fetched = server.fetch_double_array(name).expect("fetch special array");

    assert_eq!(fetched.value.len(), special_values.len());
    for (expected, actual) in special_values.iter().zip(fetched.value.iter()) {
        if expected.is_nan() {
            assert!(actual.is_nan());
        } else {
            assert_eq!(expected, actual);
        }
    }
}