SELECT u.id, u.email, ug.user_group
FROM users u INNER JOIN user_groups ug
ON u.id = ug.user_id
WHERE u.email = ?1 AND u.password = ?2
