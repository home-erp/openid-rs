use rusqlite;
use store::error::StoreError;
use store::*;

pub struct SqliteStore {
    db_path: String,
}

static INSERT_USER_SQL: &str = include_str!("insert_user.sql");
static INSERT_CLIENT_SQL: &str = include_str!("insert_client.sql");
static GET_USER_SQL: &str = include_str!("get_user.sql");
static CREATE_TABLES_SQL: &str = include_str!("create_tables_batch.sql");
static GET_CLIENT_SQL: &str = include_str!("get_client.sql");
static INSERT_USER_GROUP_SQL: &str = include_str!("insert_user_group.sql");
static REMOVE_USER_GROUP_SQL: &str = include_str!("remove_user_group.sql");
static INSERT_CLIENT_REDIRECT_SQL: &str = include_str!("insert_client_redirect.sql");
static REMOVE_CLIENT_REDIRECT_SQL: &str = include_str!("remove_redirect.sql");
static DELETE_CLIENT_SQL: &str = include_str!("delete_client.sql");
static DELETE_USER_SQL: &str = include_str!("delete_user.sql");
static LIST_USERS_SQL: &str = include_str!("list_users.sql");
static LIST_CLIENTS_SQL: &str = include_str!("list_clients.sql");


impl SqliteStore {
    fn get_connection(&self) -> Result<rusqlite::Connection, StoreError> {
        rusqlite::Connection::open(self.db_path.clone()).map_err(|e| StoreError::from(e))
    }

    fn execute(&self, sql: &str, args: &[&rusqlite::types::ToSql]) -> Result<(), StoreError> {
        let con = self.get_connection()?;
        con.execute(sql, args)?;
        Ok(())
    }

    pub fn new(location: &str) -> Result<SqliteStore, StoreError> {
        let result = SqliteStore { db_path: String::from(location) };
        let con = result.get_connection()?;
        let create_result = con.execute_batch(CREATE_TABLES_SQL);
        match create_result {
            Ok(_) => Ok(result),
            Err(e) => Err(StoreError::InternalError(Box::new(e))),
        }
    }
}

fn rows_to_user(mut rows: rusqlite::Rows) -> Result<Option<User>, StoreError> {
    let mut user = None;
    while let Some(result_row) = rows.next() {
        let row = result_row?;
        if user.is_none() {
            let inner = User {
                email: row.get(0),
                password: None,
                groups: vec![row.get(1)],
            };
            user = Some(inner);
        } else {
            let mut inner = user.unwrap(); // safe unwrap
            inner.groups.push(row.get(1));
            user = Some(inner);
        }
    }
    Ok(user)
}



impl Store for SqliteStore {
    fn get_user(&self, email: &str, pwd: &str) -> Result<Option<User>, StoreError> {
        let con = self.get_connection()?;
        let mut stmt = con.prepare(GET_USER_SQL)?;
        let rs = stmt.query(&[&email, &pwd])?;
        rows_to_user(rs)
    }


    fn get_clients(&self) -> Result<Vec<Client>, StoreError> {
        let con = self.get_connection()?;
        let mut stmt = con.prepare(LIST_CLIENTS_SQL)?;
        let mut rs = stmt.query(&[])?;
        let mut result = Vec::new();
        let mut current_client_count = 0;
        while let Some(result_row) = rs.next() {
            let row = result_row?;
            let name: String = row.get(0);
            let possible_redirect_url: rusqlite::Result<String> = row.get_checked(1);

            if result.is_empty() {
                result.push(Client {
                    name: row.get(0),
                    redirect_urls: Vec::new(),
                });
            }
            let same_client = {

                let client = &mut result[current_client_count];
                if client.name == name && possible_redirect_url.is_ok() {
                    client.redirect_urls.push(row.get(1));
                }
                client.name == name
            };

            if !same_client {
                let redirect_urls = if possible_redirect_url.is_ok() {
                    vec![row.get(1)]
                } else {
                    Vec::new()
                };
                result.push(Client {
                    name: name,
                    redirect_urls: redirect_urls,
                });
                current_client_count = current_client_count + 1;
            }
        }
        Ok(result)
    }


    fn get_users(&self) -> Result<Vec<User>, StoreError> {
        let con = self.get_connection()?;
        let mut stmt = con.prepare(LIST_USERS_SQL)?;
        let mut rs = stmt.query(&[])?;
        let mut result = Vec::new();
        let mut current_user_count = 0;
        while let Some(result_row) = rs.next() {
            let row = result_row?;
            let email: String = row.get(0);
            let possible_group: rusqlite::Result<String> = row.get_checked(1);
            if result.is_empty() {
                result.push(User {
                    email: email.clone(),
                    password: None,
                    groups: Vec::new(),
                });
            }


            // this section is a little weird to please the borrow checker.
            let same_user = {
                let user = &mut result[current_user_count];
                if user.email == email && possible_group.is_ok() {
                    user.groups.push(row.get(1));
                }
                user.email == email
            };

            if !same_user {

                let groups = if possible_group.is_ok() {
                    vec![row.get(1)]
                } else {
                    Vec::new()
                };
                result.push(User {
                    email: email,
                    password: None,
                    groups: groups,
                });
                current_user_count = current_user_count + 1;
            }

        }

        Ok(result)
    }

    fn get_client(&self, reference: &str) -> Result<Option<Client>, StoreError> {
        let con = self.get_connection()?;
        let mut stmt = con.prepare(GET_CLIENT_SQL)?;
        let mut rs = stmt.query(&[&reference])?;
        let mut client = None;
        while let Some(result_row) = rs.next() {
            let row = result_row?;
            if client.is_none() {
                let inner = Client {
                    name: row.get(0),
                    redirect_urls: vec![row.get(1)],
                };
                client = Some(inner);
            } else {
                let mut inner = client.unwrap(); // safe unwrap
                inner.redirect_urls.push(row.get(1));
                client = Some(inner);
            }
        }
        Ok(client)
    }


    fn save_user(&self, user: &User) -> Result<(), StoreError> {
        let mut con = self.get_connection()?;
        let tx = con.transaction()?;
        tx.prepare(INSERT_USER_SQL)?.execute(
            &[&user.email, &user.password],
        )?;
        {
            let mut group_stmt = tx.prepare(INSERT_USER_GROUP_SQL)?;
            for ref group in &user.groups {
                group_stmt.execute(&[&user.email, &&group[..]])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn save_client(&self, client: &Client) -> Result<(), StoreError> {
        let mut con = self.get_connection()?;
        let tx = con.transaction()?;
        tx.prepare(INSERT_CLIENT_SQL)?.execute(&[&client.name])?;
        {
            let mut redirect_stmt = tx.prepare(INSERT_CLIENT_REDIRECT_SQL)?;
            for ref url in &client.redirect_urls {
                redirect_stmt.execute(&[&client.name, &&url[..]])?;
            }
        }
        tx.commit()?;
        Ok(())
    }


    fn add_group(&self, reference: &str, group_name: &str) -> Result<(), StoreError> {
        self.execute(INSERT_USER_GROUP_SQL, &[&reference, &group_name])
    }
    fn remove_group(&self, reference: &str, group_name: &str) -> Result<(), StoreError> {
        self.execute(REMOVE_USER_GROUP_SQL, &[&reference, &group_name])
    }

    fn add_redirect_url(&self, reference: &str, redirect_url: &str) -> Result<(), StoreError> {
        self.execute(INSERT_CLIENT_REDIRECT_SQL, &[&reference, &redirect_url])
    }
    fn remove_redirect_url(&self, reference: &str, redirect_url: &str) -> Result<(), StoreError> {
        self.execute(REMOVE_CLIENT_REDIRECT_SQL, &[&reference, &redirect_url])
    }

    fn delete_client(&self, reference: &str) -> Result<(), StoreError> {
        let con = self.get_connection()?;
        con.execute("PRAGMA foreign_keys = ON", &[])?;
        con.execute(DELETE_CLIENT_SQL, &[&reference])?;
        Ok(())
    }

    fn delete_user(&self, reference: &str) -> Result<(), StoreError> {
        let con = self.get_connection()?;
        con.execute("PRAGMA foreign_keys = ON", &[])?;
        con.execute(DELETE_USER_SQL, &[&reference])?;
        Ok(())
    }
}
