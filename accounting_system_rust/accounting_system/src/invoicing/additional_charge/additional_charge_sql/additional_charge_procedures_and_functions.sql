
create or replace procedure persist_additional_charge(req create_additional_charge_request[], invoice_tab_id uuid,_tenant_id uuid,_created_by uuid) as
$$
DECLARE
    additional_charges create_additional_charge_request[] := req;
    line               create_additional_charge_request;
    title_id           uuid;
BEGIN
    if additional_charges is null then
        return;
    end if;
    foreach line in array additional_charges
        loop
            select get_or_create_line_title(line.line_title, line.title_xx_hash,
                                            null, _tenant_id)
            into title_id;
            insert into additional_charge (id, tenant_id, invoice_table_id, line_no, line_title_id, rate,
                                           created_by, updated_by, created_at, updated_at)
            values (line.line_id, _tenant_id, invoice_tab_id, line.line_no, title_id, line.rate,
                    _created_by, _created_by, default, default);
        end loop;
end;
$$
    language plpgsql;
