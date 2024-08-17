mod common;

#[cfg(test)]
mod tests {
    use crate::common::{
        broker_addr, build_create_pros, build_v3_conn_pros, build_v5_conn_pros, build_v5_pros,
        distinct_conn,
    };
    use common_base::tools::unique_id;
    use mqtt_broker::handler::connection::REQUEST_RESPONSE_PREFIX_NAME;
    use paho_mqtt::{Client, PropertyCode, ReasonCode};
    use std::process;

    #[tokio::test]
    async fn client34_connect_test() {
        let mqtt_version = 3;
        let client_id = unique_id();
        let addr = broker_addr();
        v3_wrong_password_test(mqtt_version, &client_id, &addr);
        v3_session_present_test(mqtt_version, &client_id, &addr);

        let mqtt_version = 4;
        let client_id = unique_id();
        let addr = broker_addr();
        v3_wrong_password_test(mqtt_version, &client_id, &addr);
        v3_session_present_test(mqtt_version, &client_id, &addr);
    }

    #[tokio::test]
    async fn client5_connect_test() {
        let client_id = unique_id();
        let addr = broker_addr();
        v5_wrong_password_test(&client_id, &addr);
        v5_session_present_test(&client_id, &addr);
        v5_response_test(&client_id, &addr);
        v5_assigned_client_id_test(&addr);
        v5_request_response_test(&client_id, &addr);
    }

    fn v3_wrong_password_test(mqtt_version: u32, client_id: &String, addr: &String) {
        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v3_conn_pros(mqtt_version, true);

        match cli.connect(conn_opts) {
            Ok(_) => {
                assert!(false)
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                assert!(true)
            }
        }
    }

    fn v3_session_present_test(mqtt_version: u32, client_id: &String, addr: &String) {
        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v3_conn_pros(mqtt_version, false);

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);

        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v3_conn_pros(mqtt_version, false);

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(!resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }

        distinct_conn(cli);
    }

    fn v5_wrong_password_test(client_id: &String, addr: &String) {
        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let props = build_v5_pros();
        let conn_opts = build_v5_conn_pros(props, true);
        match cli.connect(conn_opts) {
            Ok(_) => {
                assert!(false)
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                assert!(true)
            }
        }
    }

    fn v5_session_present_test(client_id: &String, addr: &String) {
        let mqtt_version = 5;
        let props = build_v5_pros();

        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros(props.clone(), false);
        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);

        let create_opts = build_create_pros(client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros(props.clone(), false);

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(!resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);
    }

    fn v5_assigned_client_id_test(addr: &String) {
        let mqtt_version = 5;
        let client_id = "".to_string();
        let props = build_v5_pros();

        let create_opts = build_create_pros(&client_id, addr);
        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros(props.clone(), false);
        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);

                let resp_pros = response.properties();
                let assign_client_id = resp_pros
                    .get(PropertyCode::AssignedClientIdentifer)
                    .unwrap()
                    .get_string()
                    .unwrap();
                assert!(!assign_client_id.is_empty());
                assert_eq!(assign_client_id.len(), unique_id().len());
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);
    }

    fn v5_request_response_test(client_id: &String, addr: &String) {
        let mqtt_version = 5;

        let mut pros = build_v5_pros();
        pros.push_val(PropertyCode::RequestResponseInformation, 0)
            .unwrap();

        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros(pros.clone(), false);

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                // response
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(!resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);

                // properties
                let resp_pros = response.properties();
                assert!(resp_pros
                    .get(PropertyCode::RequestResponseInformation)
                    .is_none());
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);
    }

    fn v5_response_test(client_id: &String, addr: &String) {
        let mqtt_version = 5;

        let pros = build_v5_pros();
        let create_opts = build_create_pros(client_id, addr);

        let cli = Client::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        let conn_opts = build_v5_conn_pros(pros.clone(), false);

        match cli.connect(conn_opts) {
            Ok(response) => {
                let resp = response.connect_response().unwrap();
                // response
                assert_eq!(format!("tcp://{}", resp.server_uri), broker_addr());
                assert_eq!(mqtt_version, resp.mqtt_version);
                assert!(!resp.session_present);
                assert_eq!(response.reason_code(), ReasonCode::Success);

                // properties
                let resp_pros = response.properties();
                assert_eq!(
                    resp_pros
                        .get(PropertyCode::SessionExpiryInterval)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    3
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::ReceiveMaximum)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    65535
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::MaximumQos)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    2
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::RetainAvailable)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    1
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::MaximumPacketSize)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    10485760
                );

                assert!(resp_pros
                    .get(PropertyCode::AssignedClientIdentifer)
                    .is_none());

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::TopicAliasMaximum)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    65535
                );

                assert!(resp_pros.get(PropertyCode::ReasonString).is_none());

                assert!(resp_pros.get(PropertyCode::UserProperty).is_none());

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::WildcardSubscriptionAvailable)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    1
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::SubscriptionIdentifiersAvailable)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    1
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::SharedSubscriptionAvailable)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    1
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::ServerKeepAlive)
                        .unwrap()
                        .get_int()
                        .unwrap(),
                    40
                );

                assert_eq!(
                    resp_pros
                        .get(PropertyCode::ResponseInformation)
                        .unwrap()
                        .get_string()
                        .unwrap(),
                    REQUEST_RESPONSE_PREFIX_NAME.to_string()
                );

                assert!(resp_pros.get(PropertyCode::ServerReference).is_none());

                assert!(resp_pros.get(PropertyCode::AuthenticationMethod).is_none());
                assert!(resp_pros.get(PropertyCode::AuthenticationData).is_none());
            }
            Err(e) => {
                println!("Unable to connect:\n\t{:?}", e);
                process::exit(1);
            }
        }
        distinct_conn(cli);
    }
}