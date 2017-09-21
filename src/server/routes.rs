use {rocket, openssl, serde_json};
use server::authentication_request::{self, OidcErr};
use std::fs::File;
use std::io::Read;


use uuid::Uuid;
use rustwt::Algorithm;
use rustwt::id_token::IDToken;
use rocket::{State, Response};
use rocket::request::Form;
use rocket::http::{Cookie, Cookies, Status};
use std::io::Cursor;
use std::ops::Deref;
use rocket::request::{self, Request, FromRequest};
use rocket::Outcome;
use server::Config;
use base64;



static FORM_TEMPLATE: &'static str = include_str!("form.html");

#[derive(FromForm)]
struct Login {
    email: String,
    password: String,
    state: String,
}


struct RequestedHost(String);


impl<'a, 'r> FromRequest<'a, 'r> for RequestedHost {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<RequestedHost, ()> {
        Outcome::Success(RequestedHost(
            request.headers().get("Host").next().unwrap().to_string(),
        ))
    }
}


#[get("/authorize?<authentication_request>")]
pub fn authorize<'r>(
    mut authentication_request: authentication_request::AuthenticationRequest,
    state: State<Config>,
    mut cookies: Cookies,
) -> Response<'r> {
    let config = state.deref();
    let validation_result = authentication_request.validate(&config);

    if validation_result.is_err() {
        let message = match validation_result.err().unwrap() {
            OidcErr::ClientErr(m) => String::from(m),
            OidcErr::InternalErr(m) => m.to_string(),
        };
        println!("{}", message);
        return Response::build().status(Status::BadRequest).finalize();
    }

    let session_id = Uuid::new_v4().simple().to_string();

    let mut sessions = config.sessions.write().unwrap();
    sessions.insert(
        session_id.clone(),
        authentication_request.state.clone().unwrap(),
    );
    let request_string = serde_json::to_string(&authentication_request).unwrap();
    cookies.add_private(Cookie::new("auth-request", request_string));
    cookies.add_private(Cookie::new("session", session_id));

    rocket::Response::build()
        .sized_body(Cursor::new(FORM_TEMPLATE.replace(
            "{{CORS-TOKEN}}",
            &authentication_request.state.unwrap(),
        )))
        .finalize()
}



#[get("/public-key")]
pub fn public_key<'r>(state: State<Config>) -> String {
    state.verification_key.clone()
}




#[post("/login", data = "<login_form>")]
fn login<'r>(
    login_form: Form<Login>,
    state: State<Config>,
    host: RequestedHost,
    mut cookies: Cookies,
) -> rocket::Response<'r> {

    let possible_cookie = cookies.get_private("auth-request");

    let login = login_form.into_inner();

    let mut pwd = login.password;

    pwd.push_str(&state.salt[..]);
    let hashed_pwd_bytes = openssl::sha::sha256(pwd.as_bytes());

    let hashed_pwd = base64::encode(&hashed_pwd_bytes);

    if possible_cookie.is_none() {
        return rocket::Response::build()
            .raw_status(400, "auth-request cookie not present")
            .finalize();
    }

    let cookie = possible_cookie.unwrap();

    let auth_request: authentication_request::AuthenticationRequest =
        serde_json::from_str(cookie.value()).unwrap();
    let auth_state = auth_request.state.unwrap();
    if login.state != auth_state {
        println!("{} vs {}", login.state, auth_state);
        return Response::build().raw_status(400, "wrong state").finalize();
    }
    let issuer = state.inner().issuer.clone();
    let key_path = format!("{}/private/sign-key.pem", state.config_dir_path.clone());
    let get_user_result = state.store.get_user(&login.email, &hashed_pwd);

    if get_user_result.is_err() {
        return rocket::Response::build()
            .raw_status(500, "error while connecting to database")
            .finalize();
    }
    let possible_user = get_user_result.unwrap();
    if possible_user.is_none() {
        println!("user not found!");
        return rocket::Response::build()
            .raw_status(404, "user not found")
            .finalize();
    }
    let user = possible_user.unwrap();
    println!("user logged in!");

    let iss = match issuer {
        Some(i) => i,
        None => host.0,
    };

    let mut pem = String::new();

    let mut f = File::open(key_path).expect("could not open private key file");

    f.read_to_string(&mut pem).expect(
        "unable to read private key file",
    );

    let id_token_builder = IDToken::build(&iss, &user.email, &[&auth_request.client_id], 60 * 20)
        .amr(&["password"])
        .nonce(auth_request.nonce.unwrap());
    // .sign_with_pem(pem, Algorithm::ES512)
    // .expect("could not sign token");


    if auth_request.response_type == "code" {
        let code = Uuid::new_v4().simple().to_string();
        let mut codes = state.codes.write().expect(
            "could not aquire lock on code map",
        );
        let token = id_token_builder.to_token_structure(Algorithm::ES512);
        codes.insert(code.clone(), token);
        let location =
            format!(
                "{}?code={}&state={}",
                auth_request.redirect_uri,
                code,
                auth_state,
            );
        Response::build()
            .raw_header("Location", location)
            .raw_status(302, "Found")
            .finalize()
    } else {
        //implicit flow, return token directly to callback
        let jwt = id_token_builder
            .sign_with_pem(pem, Algorithm::ES512)
            .expect("could not sign token");

        let location =
            format!(
                    "{}?token_type=bearer&id_token={}&expires_in={}&state={}",
                    auth_request.redirect_uri,
                    jwt,
                    state.token_duration,
                    auth_state,
                );
        println!("{}", location);
        rocket::Response::build()
            .raw_header("Location", location)
            .raw_status(302, "Found")
            .finalize()
    }
}
