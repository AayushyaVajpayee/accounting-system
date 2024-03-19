create type create_payment_terms_request as
(
    due_days         integer,
    discount_days    integer,
    discount_percent real
);
create type create_invoice_line_request as
(
    line_id                   uuid,
    line_no                   smallint,
    hsn_sac_code              text,
    line_title                text,
    title_hsn_sac_hash        bigint,
    line_subtitle             text,
    subtitle_hash             bigint,
    quantity                  double precision,
    free_quantity             double precision,
    uqc                       text,
    unit_price                double precision,
    tax_percentage            real,
    discount_percentage       real,
    cess_percentage           real,
    mrp                       real,
    batch_no                  text,
    expiry_date_ms            bigint,
    line_net_total            double precision,
    reverse_charge_applicable bool
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
    created_by                      uuid,
    igst_applicable                 bool,
    invoice_remarks                 text
);

create or replace function get_invoice_number(invoice_number_prefix text, invoice_counter integer,
                                              zero_padding bool) returns text as
$$
DECLARE
    invoice_number text;
BEGIN
    if zero_padding then
        invoice_number := invoice_number_prefix ||
                          LPAD(invoice_counter::text,
                               16 - (length(invoice_number_prefix) + length(invoice_counter::text)) + 1, '0');
    else
        invoice_number := invoice_number_prefix || invoice_counter;
    end if;
    return invoice_number;
end
$$
    language plpgsql;



create or replace function create_invoice_number(invoicing_series_mst_id uuid, _financial_year smallint,
                                                 _tenant_id uuid, _created_by uuid) returns text as
$$
DECLARE
    inv_number            text;
    zero_padding          bool;
    invoice_number_prefix text;
    invoice_counter       integer;
    counter_updated       integer;
BEGIN
    select zero_padded_counter, prefix
    from invoicing_series_mst
    where id = invoicing_series_mst_id
      and tenant_id = _tenant_id
    into zero_padding,invoice_number_prefix;
    insert into invoicing_series_counter (id, entity_version_id, tenant_id, invoicing_series_id, financial_year,
                                          counter, start_value, created_by, updated_by, created_at, updated_at)
    VALUES (uuid_generate_v7(), 0, _tenant_id, invoicing_series_mst_id, _financial_year, 1, 0,
            _created_by, _created_by, default, default)
    on conflict (tenant_id, invoicing_series_id, financial_year)
        do update set counter = invoicing_series_counter.counter + 1
    returning invoicing_series_counter.counter into invoice_counter;
    select get_invoice_number(invoice_number_prefix,
                              invoice_counter,
                              zero_padding)

    into inv_number;
    return inv_number;
END
$$ language plpgsql;

create or replace function create_invoice_table_entry(req create_invoice_request, _payment_term_id uuid) returns jsonb as
$$
DECLARE
    inv_number text;
    inv_id     uuid;
BEGIN
    select uuid_generate_v7() into inv_id;
    select create_invoice_number(req.invoicing_series_mst_id,
                                 req.financial_year, req.tenant_id, req.created_by)
    into inv_number;
    insert into invoice (id, entity_version_id, tenant_id, active, approval_status, remarks, invoicing_mst_id,
                         financial_year, invoice_number, currency_id, service_invoice, invoice_date_ms,
                         e_invoicing_applicable, supplier_business_entity, b2b_invoice, billed_to_business_entity,
                         shipped_to_business_entity, purchase_order_number, einvoice_json_s3_id, total_taxable_amount,
                         total_tax_amount, total_additional_charges_amount, round_off, total_payable_amount,
                         invoice_pdf_s3_id, invoice_template_id, payment_term_id,invoice_remarks, created_by, updated_by, created_at,
                         updated_at)
    values (inv_id, 0, req.tenant_id, true, 1, null, req.invoicing_series_mst_id, req.financial_year, inv_number,
            req.currency_id, req.service_invoice, req.invoice_date_ms, req.e_invoicing_applicable, req.supplier_id,
            req.b2b_invoice, req.billed_to_customer_id, req.shipped_to_customer_id, req.order_number, null,
            req.total_taxable_amount, req.total_tax_amount, req.total_additional_charges_amount, req.round_off,
            req.total_payable_amount, null, req.invoice_template_id, _payment_term_id,req.invoice_remarks ,req.created_by, req.created_by,
            default, default);
    return jsonb_build_object('invoice_number', inv_number, 'invoice_id', inv_id);
END

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
                                      invoice_table_id, line_title_hsn_sac_id, line_subtitle_id, quantity,free_quantity,
                                      unit_price, tax_percentage, discount_percentage, cess_percentage, line_number,
                                      line_net_total,
                                      mrp, batch, expiry_date_ms, uqc, reverse_charge_applicable, created_by,
                                      updated_by,
                                      created_at, updated_at)
            values (line.line_id, 0, req.tenant_id, true, 1, null, invoice_tab_id, title_id, subtitle_id,
                    line.quantity,line.free_quantity, line.unit_price, line.tax_percentage, line.discount_percentage, line.cess_percentage,
                    line.line_no, line.line_net_total, line.mrp, line.batch_no, line.expiry_date_ms, line.uqc,
                    line.reverse_charge_applicable,
                    req.created_by, req.created_by, default, default);
        end loop;
end
$$ language plpgsql;



create or replace function create_invoice(req create_invoice_request) returns jsonb as
$$
DECLARE
    resp            jsonb;
    invoice_id      uuid;
    invoice_id_num  jsonb;
    impacted_rows   int;
    payment_term_id uuid;
    payment_terms   create_payment_terms_request := req.payment_terms;
BEGIN
    insert into idempotence_store (idempotence_key, workflow_type, response, created_at, updated_at)
    values (req.idempotence_key, 'create_invoice', null, default, default)
    on conflict do nothing;
    get diagnostics impacted_rows= row_count;
    if impacted_rows != 0 then
        if payment_terms is not null then
            select get_or_create_payment_term(payment_terms.due_days, payment_terms.discount_days,
                                              payment_terms.discount_percent, req.tenant_id,
                                              req.created_by)
            into payment_term_id;
        end if;
        select create_invoice_table_entry(req, payment_term_id) into invoice_id_num;
        select invoice_id_num ->> 'invoice_id' into invoice_id;
        call persist_invoice_lines(req, invoice_id);
        call persist_additional_charge(req.additional_charges, invoice_id, req.tenant_id, req.created_by);
        update idempotence_store
        set response=invoice_id_num
        where idempotence_key = req.idempotence_key
          and workflow_type = 'create_invoice';
        return invoice_id_num;
    else
        select response
        from idempotence_store
        where idempotence_store.idempotence_key = req.idempotence_key
          and workflow_type = 'create_invoice'
        into resp;
        return resp;
    end if;
end;

$$ language plpgsql;
