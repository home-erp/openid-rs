SELECT c.name, cr.url
FROM clients c INNER JOIN client_redirects cr
ON c.name = cr.client_name
WHERE c.name = ?1
