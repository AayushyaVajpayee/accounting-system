CREATE UNIQUE INDEX IF NOT EXISTS idx_invoicing_series_counter_unique
    ON invoicing_series_counter (tenant_id, invoicing_series_id, financial_year);
