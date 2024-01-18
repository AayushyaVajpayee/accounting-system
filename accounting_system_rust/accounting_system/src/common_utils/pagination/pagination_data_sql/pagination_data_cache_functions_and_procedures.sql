CREATE OR REPLACE FUNCTION get_paginated_data(
    select_page_query text,
    select_count_query text,
    page_size integer,
    query_xx_hash bigint
) returns jsonb
AS
$$
DECLARE
    list       jsonb[]=ARRAY []::jsonb[];
    total_cnt  integer;
    total_pgs  integer;
    row_record record;
BEGIN
    for row_record in execute select_page_query
        loop
            list = array_append(list, to_jsonb(row_record));
        end loop;
    select total_count, total_pages from pagination_data_cache where xx_hash = query_xx_hash into total_cnt,total_pgs;
    if total_cnt is null then
        EXECUTE 'SELECT (' || select_count_query || ')' into total_cnt;
        total_pgs := CEIL(total_cnt::decimal / page_size);
        insert into pagination_data_cache (xx_hash, total_pages, total_count, created_at, expire_at)
        values (query_xx_hash, total_pgs, total_cnt, default, ((extract(epoch from now()) + 900) * 1000000));
    end if;
    -- Construct the data retrieval query
    return jsonb_build_object('rows', list, 'total_pages', total_pgs, 'total_count', total_cnt);
END
$$ LANGUAGE plpgsql;