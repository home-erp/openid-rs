SELECT u.email, ug.user_group
FROM users u INNER JOIN user_groups ug
ON u.email = ug.user_email
WHERE u.email = ?1 AND u.password = ?2
