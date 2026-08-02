use pvxs::{NTScalarMetadataBuilder, Server};

#[test]
fn test_pv_local_int32_array_fetch_post() {
    let initial_array = vec![42, 43, 44];
    let name = "loc:int32:array";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_int32_array(name, initial_array.clone(), NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_int32_array(name).expect("fetch initial");
    assert_eq!(fetched.value, initial_array);
}

#[test]
fn test_pv_local_int32_array_boundary_values() {
    let boundary_array = vec![i32::MIN, -1, 0, 1, i32::MAX];
    let name = "loc:int32:boundary";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_int32_array(name, boundary_array.clone(), NTScalarMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_int32_array(name).expect("fetch boundary");
    assert_eq!(fetched.value, boundary_array);
}

#[test]
fn test_pv_local_int32_array_empty_array_failes() {
    let server = Server::start_isolated().expect("start isolated server");
    assert!(
        server
            .create_pv_int32_array("loc:int32:convert", vec![], NTScalarMetadataBuilder::new())
            .is_err()
    );
}

#[test]
fn test_pv_local_int32_array_large_array() {
    let server = Server::start_isolated().expect("start isolated server");
    let large_array: Vec<i32> = (0..1000).collect();
    server
        .create_pv_int32_array("loc:int32:large", large_array, NTScalarMetadataBuilder::new())
        .expect("create large array pv");
}

#[test]
fn test_pv_local_int32_posting_to_array() {
    let name = "loc:int32:post";
    let server = Server::start_isolated().expect("start isolated server");

    let mut initial_array = vec![10, 20, 30];
    server
        .create_pv_int32_array(name, initial_array.clone(), NTScalarMetadataBuilder::new())
        .expect("create pv");

    initial_array[0] = 99;
    server
        .post_int32_array(name, initial_array.clone())
        .expect("post array");

    let fetched = server.fetch_int32_array(name).expect("fetch posted");
    assert_eq!(fetched.value, initial_array);
}