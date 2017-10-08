SELECT c.id,c.name, cr.url
FROM clients c INNER JOIN client_redirects cr
ON c.id = cr.client_id
WHERE c.name = ?1
