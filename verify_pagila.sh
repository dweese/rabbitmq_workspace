# Quick stats check
psql -d pagila -c "
SELECT 
    'Films' as table_name, COUNT(*) as count FROM film
UNION ALL
SELECT 
    'Customers', COUNT(*) FROM customer
UNION ALL
SELECT 
    'Rentals', COUNT(*) FROM rental
UNION ALL
SELECT 
    'Staff', COUNT(*) FROM staff
UNION ALL
SELECT 
    'Stores', COUNT(*) FROM store
ORDER BY table_name;
"