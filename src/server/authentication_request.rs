
use {uuid, url};
use server::Config;
use store;

#[derive(FromForm, Serialize, Deserialize)]
pub struct AuthenticationRequest {
    pub response_type: String,
    pub nonce: Option<String>,
    pub redirect_uri: String,
    pub client_id: String,
    pub scope: String,
    pub state: Option<String>,
    pub display: Option<String>,
    pub prompt: Option<String>,
    pub max_age: Option<String>,
    pub ui_locales: Option<String>,
    pub id_token_hint: Option<String>,
    pub login_hint: Option<String>,
    pub acr_values: Option<String>,
}

impl AuthenticationRequest {
    pub fn validate(&mut self, config: &Config) -> Result<(), OidcErr> {
        if self.scope != "openid" {
            return Err(OidcErr::ClientErr("only scopen openid is supported"));
        }
        let flow_check_result = match self.response_type.trim() {
            "id_token" => {
                if self.nonce.is_none() {
                    Err(OidcErr::ClientErr("nonce field required"))
                } else {
                    Ok(())
                }
            }
            "code" => Ok(()),
            _ => Err(OidcErr::ClientErr("invalid response type")),
        };

        let redirect_parse_result = url::Url::parse(&self.redirect_uri.trim());
        println!("{}", self.redirect_uri);
        if redirect_parse_result.is_err() {
            println!("{}", redirect_parse_result.err().unwrap());
            return Err(OidcErr::ClientErr("invalid redirect url"));
        }

        let redirect_uri = redirect_parse_result.unwrap();

        if redirect_uri.scheme() == "http" &&
            (redirect_uri.host().unwrap() != url::Host::Domain("localhost") &&
                 redirect_uri.host().unwrap() != url::Host::Ipv4("127.0.0.1".parse().unwrap()))
        {

            return Err(OidcErr::ClientErr("invalid redirect url"));
        }
        if flow_check_result.is_err() {
            return flow_check_result;
        }

        let client_lookup_result = config.store.get_client(self.client_id.trim());

        if client_lookup_result.is_err() {
            return Err(OidcErr::InternalErr(client_lookup_result.err().unwrap()));
        }
        let possible_client = client_lookup_result.unwrap();
        if possible_client.is_none() {
            return Err(OidcErr::ClientErr("invalid client id"));
        }

        let client = possible_client.unwrap();


        if !client.redirect_urls.iter().any(|item| {
            println!("{}", item);
            println!("{}", self.redirect_uri.trim());
            item == self.redirect_uri.trim()
        })
        {
            return Err(OidcErr::ClientErr("invalid redirect uri"));
        }

        if self.display.is_none() {
            self.display = Some(String::from("page"));
        }

        self.state = Some(uuid::Uuid::new_v4().simple().to_string());

        //TODO implement optional options

        Ok(())

    }
}



pub enum OidcErr {
    InternalErr(store::error::StoreError),
    ClientErr(&'static str),
}
