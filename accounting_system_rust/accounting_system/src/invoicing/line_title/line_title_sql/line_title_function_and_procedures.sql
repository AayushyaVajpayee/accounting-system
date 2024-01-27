create or replace function get_or_create_line_title(title text, title_xx_hash bigint,
                                                    hsn_sac_code text, _tenant_id uuid) returns uuid as
$$
DECLARE
    title_id uuid;
BEGIN
    select id
    from line_title
    where tenant_id = _tenant_id
      and xx_hash = title_xx_hash
    into title_id;
    if title_id is null then
        select uuid_generate_v7() into title_id;
        insert into line_title (id, tenant_id, description, hsn_code, xx_hash, created_at)
        values (title_id, _tenant_id, title, hsn_sac_code, title_xx_hash, default);
    end if;
    return title_id;
end;
$$ language plpgsql;