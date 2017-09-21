pub mod error;
pub mod sqlite_store;


use self::error::StoreError;

pub trait Store {
    fn get_user(&self, &str, &str) -> Result<Option<User>, StoreError>;
    fn get_client(&self, &str) -> Result<Option<Client>, StoreError>;
    fn save_user(&self, user: &User) -> Result<(), StoreError>;
    fn save_client(&self, client: &Client) -> Result<(), StoreError>;
    fn delete_user(&self, reference: &str) -> Result<(), StoreError>;
    fn delete_client(&self, reference: &str) -> Result<(), StoreError>;

    fn get_users(&self) -> Result<Vec<User>, StoreError>;
    fn get_clients(&self) -> Result<Vec<Client>, StoreError>;

    fn add_group(&self, reference: &str, group_name: &str) -> Result<(), StoreError>;

    fn remove_group(&self, reference: &str, group_name: &str) -> Result<(), StoreError>;

    fn add_redirect_url(&self, reference: &str, redirect_url: &str) -> Result<(), StoreError>;
    fn remove_redirect_url(&self, reference: &str, redirect_url: &str) -> Result<(), StoreError>;
}

pub struct Client {
    pub name: String,
    pub redirect_urls: Vec<String>,
}


pub struct User {
    pub email: String,
    pub password: Option<String>,
    pub groups: Vec<String>,
}
