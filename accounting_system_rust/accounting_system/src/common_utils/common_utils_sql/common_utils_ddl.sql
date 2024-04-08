create type mime_type as enum ('csv','docx','jpeg','json','png','pdf','txt','xlsx');
create type workflow_type as enum ('dummy_test','create_tenant','create_account_type_mst','create_account',
    'create_currency','create_app_user','create_company_mst','create_address','create_company_unit_mst',
    'create_invoice_no_series','create_business_entity','create_invoice','create_product_item','create_invoice_template');
create table idempotence_store
(
    idempotence_key uuid          not null,
    workflow_type   workflow_type not null,
--     request jsonb not null,
    response        jsonb,
    created_at      bigint default extract(epoch from now()) * 1000000,
    updated_at      bigint default extract(epoch from now()) * 1000000,
    primary key (idempotence_key, workflow_type)
);

