UPDATE fund_quotes
SET fetched_at = created_at
WHERE source = 'eastmoney/pingzhongdata'
  AND strftime('%H:%M:%S', fetched_at) = '16:00:00'
  AND (julianday(created_at) - julianday(fetched_at)) > 0.25;
