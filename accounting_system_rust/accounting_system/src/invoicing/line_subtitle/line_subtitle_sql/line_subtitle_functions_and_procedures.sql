create or replace function get_or_create_line_subtitle(subtitle text, _tenant_id uuid, subtitle_hash bigint)
    returns uuid as
$$
DECLARE
    subtitle_id uuid;
BEGIN
    if subtitle is not null then
        select id
        from line_subtitle
        where line_subtitle.tenant_id = _tenant_id
          and xx_hash = subtitle_hash
        into subtitle_id;
        if subtitle_id is null then
            select uuid_generate_v7() into subtitle_id;
            insert into line_subtitle (id, tenant_id, description, xx_hash, created_at)
            values (subtitle_id, _tenant_id, subtitle, subtitle_hash, default);
        end if;
    else
        subtitle_id := null;
    end if;
    return subtitle_id;
end;
$$ language plpgsql;