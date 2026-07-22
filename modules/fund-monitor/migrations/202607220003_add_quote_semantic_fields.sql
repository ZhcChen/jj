ALTER TABLE fund_quotes ADD COLUMN nav_date TEXT;
ALTER TABLE fund_quotes ADD COLUMN confirmed_change_rate REAL;
ALTER TABLE fund_quotes ADD COLUMN estimated_change_rate REAL;
ALTER TABLE fund_quotes ADD COLUMN estimated_at TEXT;

UPDATE fund_quotes
SET confirmed_change_rate = change_rate
WHERE confirmed_change_rate IS NULL;
