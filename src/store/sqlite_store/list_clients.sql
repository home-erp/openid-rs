select c.name, cr.url from clients c left outer join client_redirects cr on c.name = cr.client_name
