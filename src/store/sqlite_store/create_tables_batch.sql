CREATE TABLE IF NOT EXISTS users (email text primary key not null, password text);
CREATE TABLE IF NOT EXISTS clients (name text primary key not null);
CREATE TABLE IF NOT EXISTS client_redirects(client_name text, url text, PRIMARY KEY (client_name, url) FOREIGN KEY(client_name) references clients(name) ON DELETE CASCADE);
CREATE TABLE IF NOT EXISTS user_groups (user_email text, user_group text, PRIMARY KEY (user_email, user_group) FOREIGN KEY(user_email) references users(email) ON DELETE CASCADE);
