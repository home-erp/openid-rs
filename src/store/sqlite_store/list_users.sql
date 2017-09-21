SELECT u.email, ug.user_group FROM users u left outer join user_groups ug on u.email = ug.user_email
