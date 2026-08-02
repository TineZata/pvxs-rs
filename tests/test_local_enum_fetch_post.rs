use pvxs::{NTEnumMetadataBuilder, Server};

#[test]
fn test_pv_local_enum_fetch_post() {
    let name = "loc:enum";
    let choices = vec!["OFF", "ON", "STANDBY"];
    let initial_index = 1;
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_enum(name, choices.clone(), initial_index, NTEnumMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_enum(name).expect("fetch initial");
    assert_eq!(fetched.value, initial_index);
    assert_eq!(fetched.value_choices.len(), choices.len());
    for (i, choice) in choices.iter().enumerate() {
        assert_eq!(&fetched.value_choices[i], choice);
    }

    let new_index = 2;
    server.post_enum(name, new_index).expect("post enum");
    let fetched = server.fetch_enum(name).expect("fetch posted");
    assert_eq!(fetched.value, new_index);

    assert!(server.post_enum(name, 99).is_err());
}

#[test]
fn test_pv_local_enum_fetch_post_with_error_propagation() {
    let name = "loc:enum:err";
    let baudrate = vec!["9600", "19200", "38400", "57600", "115200"];
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_enum(name, baudrate, 0, NTEnumMetadataBuilder::new())
        .expect("create pv");

    let fetched = server.fetch_enum(name).expect("fetch initial");
    assert_eq!(fetched.value, 0);

    server.post_enum(name, 1).expect("post enum");
    let fetched = server.fetch_enum(name).expect("fetch posted");
    assert_eq!(fetched.value, 1);
}

#[test]
fn test_pv_local_enum_all_states() {
    let name = "loc:enum:states";
    let choices = vec!["STATE_0", "STATE_1", "STATE_2", "STATE_3", "STATE_4"];
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_enum(name, choices.clone(), 0, NTEnumMetadataBuilder::new())
        .expect("create pv");

    for (expected_index, expected_choice) in choices.iter().enumerate() {
        server
            .post_enum(name, expected_index as i16)
            .expect("post enum state");
        let fetched = server.fetch_enum(name).expect("fetch enum state");
        let index = fetched.value as usize;
        assert_eq!(index, expected_index);
        assert_eq!(&fetched.value_choices[index], expected_choice);
    }
}

#[test]
fn test_pv_local_enum_boundary_conditions() {
    let choices = vec!["FIRST", "MIDDLE", "LAST"];
    let name = "loc:enum:bounds";
    let server = Server::start_isolated().expect("start isolated server");

    server
        .create_pv_enum(name, choices, 0, NTEnumMetadataBuilder::new())
        .expect("create pv");

    server.post_enum(name, 0).expect("set first");
    assert_eq!(server.fetch_enum(name).expect("fetch first").value, 0);

    server.post_enum(name, 2).expect("set last");
    assert_eq!(server.fetch_enum(name).expect("fetch last").value, 2);

    assert!(server.post_enum(name, -1).is_err());
}