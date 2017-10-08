CREATE TABLE IF NOT EXISTS users (id text primary key, email unique not null, password text);
CREATE TABLE IF NOT EXISTS clients (id text primary key,name unique not null);
CREATE TABLE IF NOT EXISTS client_redirects(client_id text, url text, PRIMARY KEY (client_id, url) FOREIGN KEY(client_id) references clients(id) ON DELETE CASCADE);
CREATE TABLE IF NOT EXISTS user_groups (user_id text, user_group text, PRIMARY KEY (user_id, user_group) FOREIGN KEY(user_id) references users(id) ON DELETE CASCADE);
