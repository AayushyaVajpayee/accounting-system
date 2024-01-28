create or replace procedure persist_additional_charge(req create_invoice_request, invoice_tab_id uuid) as
$$
DECLARE
    additional_charges create_additional_charge_request[] := req.additional_charges;
    line               create_additional_charge_request;
    title_id           uuid;
BEGIN
    foreach line in array additional_charges
        loop
            select get_or_create_line_title(line.line_title, line.title_xx_hash,
                                            null, req.tenant_id)
            into title_id;
            insert into additional_charge (id, tenant_id, invoice_table_id, line_no, line_title_id, rate,
                                           created_by, updated_by, created_at, updated_at)
            values (line.line_id, req.tenant_id, invoice_tab_id, line.line_no, title_id, line.rate,
                    req.created_by, req.created_by, default, default);
        end loop;
end;
$$
    language plpgsql;
