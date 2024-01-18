CREATE UNIQUE INDEX company_unit_master_gstin_unique_idx
    ON company_unit_master (gstin)
    WHERE approval_status <> 4;