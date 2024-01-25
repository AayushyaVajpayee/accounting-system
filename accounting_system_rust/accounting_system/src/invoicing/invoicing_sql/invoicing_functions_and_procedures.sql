

create type create_payment_terms_request as
(
    due_days         integer,
    discount_days    integer,
    discount_percent integer
);
create type create_invoice_line_request as
(
    line_id            uuid,
    line_no            text,
    hsn_sac_code       text,
    line_title         text,
    title_hsn_sac_hash bigint,
    line_subtitle      text,
    subtitle_hash      bigint,
    quantity           double precision,
    uqc                text,
    unit_price         integer,
    tax_rate_bps       integer,
    discount_bps       integer,
    cess_bps           integer,
    mrp                integer,
    batch_no           text,
    expiry_date_ms     bigint,
    line_net_total     double precision,
    igst_applicable    bool
);

create type create_additional_charge_request as
(
    line_id       uuid,
    line_no       smallint,
    line_title    text,
    title_xx_hash bigint,
    rate          double precision
);
create type create_invoice_request as
(
    idempotence_key                 uuid,
    tenant_id                       uuid,
    invoice_template_id             uuid,
    invoicing_series_mst_id         uuid,
    invoice_date_ms                 bigint,
    currency_id                     uuid,
    service_invoice                 bool,
    b2b_invoice                     bool,
    e_invoicing_applicable          bool,
    supplier_id                     uuid,
    billed_to_customer_id           uuid,
    shipped_to_customer_id          uuid,
    order_number                    text,
    order_date                      bigint,
    payment_terms                   create_payment_terms_request,
    invoice_lines                   create_invoice_line_request[],
    additional_charges              create_additional_charge_request[],
    financial_year                  smallint,
    total_taxable_amount            double precision,
    total_tax_amount                double precision,
    total_additional_charges_amount double precision,
    round_off                       double precision,
    total_payable_amount            double precision,
    created_by                      uuid
);
create or replace function create_invoice_number(invoicing_series_mst_id uuid, _financial_year smallint,
                                                 _tenant_id uuid) returns text as
$$
DECLARE
    inv_number            text;
    zero_padding          bool;
    invoice_number_prefix text;
    invoice_counter       integer;
BEGIN
    select zero_padded_counter, prefix
    from invoicing_series_mst
    where id = invoicing_series_mst_id
      and tenant_id = _tenant_id
    into zero_padding,invoice_number_prefix;
    update invoicing_series_counter
    set counter=counter + 1
    where invoicing_series_id = invoicing_series_mst_id
      and financial_year = _financial_year
      and tenant_id = _tenant_id
    returning counter into invoice_counter;
    select get_invoice_number(invoice_number_prefix,
                              invoice_counter,
                              zero_padding)
    into inv_number;
    return inv_number;
END
$$ language plpgsql;

create or replace function create_invoice_table_entry(req create_invoice_request, _payment_term_id uuid) returns uuid as
$$
DECLARE
    inv_number text;
    inv_id     uuid;
BEGIN
    select uuid_generate_v7() into inv_id;
    select create_invoice_number(req.invoicing_series_mst_id,
                                 req.financial_year, req.tenant_id)
    into inv_number;
    insert into invoice (id, entity_version_id, tenant_id, active, approval_status, remarks, invoicing_mst_id,
                         financial_year, invoice_number, currency_id, service_invoice, invoice_date_ms,
                         e_invoicing_applicable, supplier_business_entity, b2b_invoice, billed_to_business_entity,
                         shipped_to_business_entity, purchase_order_number, einvoice_json_s3_id, total_taxable_amount,
                         total_tax_amount, total_additional_charges_amount, round_off, total_payable_amount,
                         invoice_pdf_s3_id, invoice_template_id, payment_term_id, created_by, updated_by, created_at,
                         updated_at)
    values (inv_id, 0, req.tenant_id, true, 1, null, req.invoicing_series_mst_id, req.financial_year, inv_number,
            req.currency_id, req.service_invoice, req.invoice_date_ms, req.e_invoicing_applicable, req.supplier_id,
            req.b2b_invoice, req.billed_to_customer_id, req.shipped_to_customer_id, req.order_number, null,
            req.total_taxable_amount, req.total_tax_amount, req.total_additional_charges_amount, req.round_off,
            req.total_payable_amount, null, req.invoice_template_id, _payment_term_id, req.created_by, req.created_by,
            default, default);


END

$$ language plpgsql;


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
end;
$$ language plpgsql;
create or replace procedure persist_invoice_lines(req create_invoice_request, invoice_tab_id uuid) as
$$
DECLARE
    invoice_lines create_invoice_line_request[] := req.invoice_lines;
    line          create_invoice_line_request;
    title_id      uuid;
    subtitle_id   uuid;
BEGIN
    FOREACH line in array invoice_lines
        loop
            select get_or_create_line_title(line.line_title, line.title_hsn_sac_hash,
                                            line.hsn_sac_code, req.tenant_id)
            into title_id;
            select get_or_create_line_subtitle(line.line_subtitle, req.tenant_id,
                                               line.subtitle_hash)
            into subtitle_id;
            insert into invoice_line (id, entity_version_id, tenant_id, active, approval_status, remarks,
                                      invoice_table_id, line_title_hsn_sac_id, line_subtitle_id, quantity,
                                      unit_price, tax_rate_bps, discount_bps, cess_bps, line_number, line_net_total,
                                      mrp, batch, expiry_date_ms, uqc, igst_applicable, created_by, updated_by,
                                      created_at, updated_at)
            values (line.line_id, 0, req.tenant_id, true, 1, null, invoice_tab_id, title_id, subtitle_id,
                    line.quantity, line.unit_price, line.tax_rate_bps, line.discount_bps, line.cess_bps,
                    line.line_no, line.line_net_total, line.mrp, line.batch_no, line.expiry_date_ms, line.uqc,
                    line.igst_applicable, req.created_by, req.created_by, default, default);
        end loop;
end
$$ language plpgsql;


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
        VALUES (term_id, 0, _due_days, _discount_days, _discount_percent, _created_by, _created_by, default, default);
    end if;
    return term_id;
end
$$ language plpgsql;
create or replace function create_invoice(req create_invoice_request) returns uuid as
$$
DECLARE
    resp            jsonb;
    invoice_id      uuid;
    impacted_rows   int;
    payment_term_id uuid;
    payment_terms   create_payment_terms_request := req.payment_terms;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_invoice', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        if payment_terms is null then
            select get_or_create_payment_term(payment_terms.due_days, payment_terms.discount_days,
                                              payment_terms.discount_percent, req.tenant_id,
                                              req.created_by)
            into payment_term_id;
        end if;
        select create_invoice_table_entry(req, payment_term_id) into invoice_id;
        select persist_invoice_lines(req, invoice_id);
        select persist_additional_charge(req, invoice_id);
    else
        select response
        from idempotence_store
        where idempotence_store.idempotence_key = req.idempotence_key
          and workflow_type = 'create_invoice'
        into resp;
        return (resp ->> 'id')::uuid;
    end if;
end;

$$ language plpgsql;

create or replace function get_invoice_number(invoice_number_prefix text, invoice_counter text, zero_padding bool) returns text as
$$
DECLARE
    invoice_number text;
BEGIN
    if zero_padding then
        invoice_number := invoice_number_prefix ||
                          LPAD(invoice_counter, 16 - (length(invoice_number_prefix) + length(invoice_counter)), '0');
    else
        invoice_number := invoice_number_prefix || invoice_counter;
    end if;
    return invoice_number;
end
$$
    language plpgsql


