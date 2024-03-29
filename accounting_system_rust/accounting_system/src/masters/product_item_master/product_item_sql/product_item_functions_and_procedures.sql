create type create_tax_rate_request as
(
    idempotence_key uuid,
    tenant_id       uuid,
    created_by      uuid,
    tax_percentage  real,
    start_date      timestamp
);

create type create_cess_request as
(
    idempotence_key      uuid,
    tenant_id            uuid,
    created_by           uuid,
    cess_strategy        text,
    cess_rate_percentage real,
    cess_amount_per_unit double precision,
    retail_sale_price    double precision,
    start_date           timestamp
);

create type create_product_item_request as
(
    idempotence_key         uuid,
    tenant_id               uuid,
    created_by              uuid,
    title                   text,
    subtitle                text,
    product_hash            text,--use xxhash?lets go with md5 where spaces are trimmed is case insensitive
    uom                     text,
    hsn_sac_code            text,
    create_tax_rate_request create_tax_rate_request,
    create_cess_request     create_cess_request

);--title and subtitle should be hashed to maintain uniqueness

create or replace function create_product_item(req create_product_item_request) returns uuid as
$$
DECLARE
    _product_id   uuid;
    resp          jsonb;
    impacted_rows int;
    tax_request   create_tax_rate_request := req.create_tax_rate_request;
    cess_request  create_cess_request     := req.create_cess_request;
    _tax_rate_id  uuid;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_product_item', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        select uuid_generate_v7() into _product_id;
        insert into product_item (id, tenant_id, entity_version_id, active, approval_status, remarks, title,
                                  subtitle, hash, hsn_sac_code, created_by, updated_by, created_at, updated_at)
        values (_product_id, req.tenant_id, 0, true, 1, null, req.title, req.subtitle, req.product_hash,
                req.hsn_sac_code, req.created_by, req.created_by, default, default);
        insert into product_tax_rate (id, tenant_id, entity_version_id, active, approval_status, remarks,
                                      product_id, tax_rate_percentage, start_date, end_date, created_by,
                                      updated_by, created_at, updated_at)
        values (uuid_generate_v7(), req.tenant_id, 0, true, 1, null, _product_id, tax_request.tax_percentage,
                tax_request.start_date, null, tax_request.created_by, tax_request.created_by, default, default)
        returning id into _tax_rate_id;
        insert into cess_tax_rate (id, tenant_id, entity_version_id, active, approval_status, remarks, product_id,
                                   cess_strategy, cess_rate_percentage, cess_amount_per_unit, retail_sale_price,
                                   start_date, end_date, created_by, updated_by, created_at, updated_at)
        values (uuid_generate_v7(), cess_request.tenant_id, 0, true, 1, null, _product_id, cess_request.cess_strategy,
                cess_request.cess_rate_percentage, cess_request.cess_amount_per_unit,
                cess_request.retail_sale_price, cess_request.start_date, null, cess_request.created_by,
                cess_request.created_by, default, default);
        update idempotence_store
        set response=jsonb_build_object('id', _product_id)
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_product_item';
        return _product_id;
    else
        select response
        from idempotence_store
        where idempotence_store.idempotence_key = req.idempotence_key
          and workflow_type = 'create_product_item'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;

end;
$$ language plpgsql;

create or replace function get_product_item(_product_id uuid, _tenant_id uuid) returns jsonb as
$$
DECLARE
    product_item_row   record;
    tax_rates_records  jsonb;
    cess_rates_records jsonb;
    resp jsonb;
BEGIN
    select id,
           tenant_id,
           entity_version_id,
           active,
           approval_status,
           remarks,
           title,
           subtitle,
           hsn_sac_code,
           created_by,
           updated_by,
           created_at,
           updated_at
    from product_item
    where id = _product_id
      and tenant_id = _tenant_id
    into product_item_row;

    select jsonb_agg(jsonb_build_object(
            'tax_rate_percentage', tax_rate_percentage,
            'start_date', start_date,
            'end_date', end_date
                     ))
    from product_tax_rate
    where product_id = _product_id
      and tenant_id = _tenant_id
    into tax_rates_records;

    select jsonb_agg(jsonb_build_object(
            'cess_strategy', cess_strategy,
            'cess_rate_percentage', cess_rate_percentage,
            'cess_amount_per_unit', cess_amount_per_unit,
            'retail_sale_price', retail_sale_price,
            'start_date', start_date,
            'end_date', end_date
                     ))
    from cess_tax_rate
    where product_id = _product_id
      and tenant_id = _tenant_id
    into cess_rates_records;

    select jsonb_build_object(
            'product_item', to_jsonb(product_item_row),
            'temporal_tax_rates', tax_rates_records,
            'temporal_cess_rates', cess_rates_records
           ) into resp;
    return resp;
end;
$$ language plpgsql;