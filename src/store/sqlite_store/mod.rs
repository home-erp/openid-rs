use rusqlite;
use store::error::StoreError;
use store::*;
use std::collections::HashMap;

pub struct SqliteStore {
    db_path: String,
}

static INSERT_USER_SQL: &str = include_str!("insert_user.sql");
static INSERT_CLIENT_SQL: &str = include_str!("insert_client.sql");
static GET_USER_SQL: &str = include_str!("get_user.sql");
static CREATE_TABLES_SQL: &str = include_str!("create_tables_batch.sql");
static GET_CLIENT_SQL: &str = include_str!("get_client.sql");
static REMOVE_USER_GROUP_SQL: &str = include_str!("remove_user_group.sql");
static REMOVE_CLIENT_REDIRECT_SQL: &str = include_str!("remove_redirect.sql");
static DELETE_CLIENT_SQL: &str = include_str!("delete_client.sql");
static DELETE_USER_SQL: &str = include_str!("delete_user.sql");
static LIST_USERS_SQL: &str = include_str!("list_users.sql");
static LIST_CLIENTS_SQL: &str = include_str!("list_clients.sql");

impl SqliteStore {
    fn get_connection(&self) -> Result<rusqlite::Connection, StoreError> {
        rusqlite::Connection::open(self.db_path.clone()).map_err(|e| StoreError::from(e))
    }


    fn insert_user_group_sql(&self, group_name: &str) -> String {
        format!(
            "INSERT INTO user_groups(user_id,user_group) SELECT id, {} from users where email = ?1",
            group_name
        )
    }

    fn insert_client_redirect_sql(&self, redirect_url: &str) -> String {
        format!(
            "INSERT INTO client_redirects(client_id,url) select id, {} from clients where name = ?1",
            redirect_url
        )
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
                id: row.get(0),
                email: row.get(1),
                password: None,
                groups: vec![row.get(2)],
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


    fn get_clients(&self) -> Result<HashMap<String, Client>, StoreError> {
        let con = self.get_connection()?;
        let mut stmt = con.prepare(LIST_CLIENTS_SQL)?;
        let mut rs = stmt.query(&[])?;
        let mut clients = HashMap::new();
        while let Some(result_row) = rs.next() {
            let row = result_row?;
            let possible_redirect_url: rusqlite::Result<String> = row.get_checked(2);
            let id = row.get(0);
            let client = clients.entry(id).or_insert(Client {
                id: row.get(0),
                name: row.get(1),
                redirect_urls: Vec::new(),
            });
            if possible_redirect_url.is_ok() {
                client.redirect_urls.push(row.get(2));
            }
        }
        Ok(clients)
    }


    fn get_users(&self) -> Result<HashMap<String, User>, StoreError> {
        let con = self.get_connection()?;
        let mut stmt = con.prepare(LIST_USERS_SQL)?;
        let mut rs = stmt.query(&[])?;

        let mut users = HashMap::new();
        while let Some(result_row) = rs.next() {
            let row = result_row?;

            let id = row.get(0);

            let user = users.entry(id).or_insert(User {
                id: row.get(0),
                email: row.get(1),
                password: None,
                groups: Vec::new(),
            });

            let possible_group: rusqlite::Result<String> = row.get_checked(2);
            if possible_group.is_ok() {
                user.groups.push(possible_group.unwrap());
            }
        }
        Ok(users)
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
                    id: row.get(0),
                    name: row.get(1),
                    redirect_urls: vec![row.get(2)],
                };
                client = Some(inner);
            } else {
                let mut inner = client.unwrap(); // safe unwrap
                inner.redirect_urls.push(row.get(2));
                client = Some(inner);
            }
        }
        Ok(client)
    }


    fn save_user(&self, user: &User) -> Result<(), StoreError> {
        let mut con = self.get_connection()?;
        let tx = con.transaction()?;
        tx.prepare(INSERT_USER_SQL)?.execute(
            &[
                &user.id,
                &user.email,
                &user.password,
            ],
        )?;
        {
            let sql = "INSERT INTO user_groups(user_id,user_group) values(?1,?2)";
            let mut group_stmt = tx.prepare(sql)?;
            for ref group in &user.groups {
                group_stmt.execute(&[&user.id, &&group[..]])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn save_client(&self, client: &Client) -> Result<(), StoreError> {
        let mut con = self.get_connection()?;
        let tx = con.transaction()?;
        tx.prepare(INSERT_CLIENT_SQL)?.execute(
            &[&client.id, &client.name],
        )?;
        {
            let sql = "INSERT INTO client_redirects(client_id,url) values(?1,?2)";
            let mut redirect_stmt = tx.prepare(sql)?;
            for ref url in &client.redirect_urls {
                redirect_stmt.execute(&[&client.id, &&url[..]])?;
            }
        }
        tx.commit()?;
        Ok(())
    }


    fn add_group(&self, reference: &str, group_name: &str) -> Result<(), StoreError> {
        let sql = self.insert_user_group_sql(group_name);
        self.execute(&sql, &[&reference])
    }
    fn remove_group(&self, reference: &str, group_name: &str) -> Result<(), StoreError> {
        self.execute(REMOVE_USER_GROUP_SQL, &[&reference, &group_name])
    }

    fn add_redirect_url(&self, reference: &str, redirect_url: &str) -> Result<(), StoreError> {
        let sql = self.insert_client_redirect_sql(redirect_url);
        self.execute(&sql, &[&reference])
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
