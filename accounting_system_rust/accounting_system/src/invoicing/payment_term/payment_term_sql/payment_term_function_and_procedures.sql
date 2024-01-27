create or replace function get_or_create_payment_term(_due_days integer, _discount_days integer,
                                                      _discount_percent integer,
                                                      _tenant_id uuid, _created_by uuid) returns uuid as
$$
DECLARE
    term_id uuid;
BEGIN
    select id
    from payment_term
    where tenant_id = _tenant_id
      and due_days = _due_days
      and discount_percent = _discount_percent
      and discount_days = _discount_days
    into term_id;
    if term_id is null then
        select uuid_generate_v7() into term_id;
        insert into payment_term (id, tenant_id, due_days, discount_days, discount_percent,
                                  created_by, updated_by, created_at, updated_at)
        VALUES (term_id,_tenant_id, _due_days, _discount_days, _discount_percent, _created_by, _created_by, default, default);
    end if;
    return term_id;
end
$$ language plpgsql;