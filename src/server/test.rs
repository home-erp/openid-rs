use {Uuid, Config, User, fs, add_mounts, authentication_request, rocket, RwLock};
use HashMap;
use Client;
use store::{Store, SqliteStore};

#[test]
fn test_sqlite_user_api() {
    let tmp_dir = format!("/tmp/{}", Uuid::new_v4().simple().to_string());
    let store = SqliteStore::new(&tmp_dir[..]);
    let user = User {
        id: String::from("123"),
        email: String::from("user@example.com"),
        password: Some(String::from("secret")),
        groups: vec![String::from("user"), String::from("admin")],
    };
    store.save_user(&user).expect("could not save user");
    let possible_user = store.get_user("user@example.com", "secret").expect(
        "could not load user",
    );

    assert!(possible_user.is_some());

    let loaded_user = possible_user.unwrap();
    assert_eq!(loaded_user.id, "123");


    let mut invalid_user = store.get_user("user@othersite.com", "secret").expect(
        "could not load user",
    );

    assert!(invalid_user.is_none());

    invalid_user = store.get_user("user@example.com", "wrong_secret").unwrap();

    assert!(invalid_user.is_none());

    fs::remove_dir_all(&tmp_dir).unwrap();
}


#[test]
fn test_authorization_endpoint() {
    let mut authentication_request = authentication_request::AuthenticationRequest {
        response_type: String::from("id_token"),
        nonce: Some(String::from("123")),
        redirect_uri: String::from("https://example.org/cb"),
        client_id: String::from("111"),
        scope: String::from("openid"),
        state: None,
        display: None,
        prompt: None,
        max_age: None,
        ui_locales: None,
        id_token_hint: None,
        login_hint: None,
        acr_values: None,
    };

    let tmp_dir = format!("/tmp/{}", Uuid::new_v4().simple().to_string());
    let store = SqliteStore::new(&tmp_dir[..]);

    let auth_client = Client {
        id: String::from("111"),
        name: String::from("foobar"),
        redirect_uris: vec![
            String::from("https://example.com/cb"),
            String::from("http://localhost/cb"),
            String::from("http://example.com/cb"),
        ],
    };

    store.save_client(&auth_client).expect("save client");

    assert!(store.get_client("111").expect("load client").is_some());

    let config = Config {
        issuer: String::from("localhost"),
        config_dir_path: String::from("~/.config/openid-rs"),
        store: store,
        sessions: RwLock::new(HashMap::new()),
        token_duration: 7 * 24 * 60 * 60,
        codes: RwLock::new(HashMap::new()),
        salt: String::from("wurstbrot"),
    };



    //.mount("/", routes![authorize]).

    let mut rocket_instance = rocket::ignite().manage(config);
    rocket_instance = add_mounts(rocket_instance);
    let client = rocket::local::Client::new(rocket_instance).expect("valid rocket instance");

    //    &redirect_uri=https%3A%2F%2Fexample.com%2Fcb
    //    &redirect_uri=https://example.com/cb
    let mut req = client.get(
        r#"/authorize?response_type=id_token
    &nonce=123
    &redirect_uri=https%3A%2F%2Fexample.com%2Fcb
    &client_id=111&scope=openid"#,
    );
    let mut response = req.dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    //code flow is allowed to leave nonce empty
    response = client
        .get(
            r#"/authorize?response_type=code
    &redirect_uri=https%3A%2F%2Fexample.com%2Fcb
    &client_id=111&scope=openid"#,
        )
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    //http is allowed as a callback protocol if the host is localhost
    response = client
        .get(
            r#"/authorize?response_type=code
    &redirect_uri=http%3A%2F%2Flocalhost%2Fcb
    &client_id=111&scope=openid"#,
        )
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    //http is NOT allowed as a callback protocol if the host is  NOT localhost
    response = client
        .get(
            r#"/authorize?response_type=code
    &redirect_uri=http%3A%2F%2Fexample.com%2Fcb
    &client_id=111&scope=openid"#,
        )
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::BadRequest);

    //only registered callbacks are allowed
    response = client
        .get(
            r#"/authorize?response_type=id_token
    &nonce=123
    &redirect_uri=https%3A%2F%2Fexample.com%2Fwrong_cb
    &client_id=111&scope=openid"#,
        )
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::BadRequest);

    // implicit flow must have nonce
    response = client
        .get(
            r#"/authorize?response_type=id_token
    &redirect_uri=https%3A%2F%2Fexample.com%2Fcb
    &client_id=111&scope=openid"#,
        )
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::BadRequest);

    //scope must be openid
    response = client
        .get(
            "/authorize?response_type=code
            &redirect_uri=https%3A%2F%2Fexample.com%2Fcb&client_id=111&scope=bla",
        )
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::BadRequest);

    //only registered clients are allowed
    response = client
        .get(
            "/authorize?response_type=code
            &redirect_uri=https%3A%2F%2Fexample.com%2Fcb&client_id=222&scope=openid",
        )
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::BadRequest);

    // the response type must be either code or token
    response = client
        .get(
            "/authorize?response_type=asdf
            &redirect_uri=https%3A%2F%2Fexample.com%2Fcb&client_id=111&scope=openid",
        )
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::BadRequest);

}
