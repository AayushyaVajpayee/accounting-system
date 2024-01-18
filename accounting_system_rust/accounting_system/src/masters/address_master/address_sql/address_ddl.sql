-- assumption is address will be of india only. how do we make this internationsl?
-- what will be required to make the system be able to serve international boundaries without much change
create table address
(
    id                uuid primary key,
    entity_version_id integer default 0,
    tenant_id         uuid references tenant (id),
    active            bool,
    approval_status   smallint                            not null,
    remarks           varchar(70),

    line_1            varchar(60)                         not null,
    line_2     varchar(60),
    landmark   varchar(60), -- mostly landmark,
    city_id    uuid references city_master (id),
    state_id   uuid references state_master (id),
    pincode_id uuid references pincode_master (id),
    country    uuid references country_master (id) not null,
    created_by        uuid references app_user (id)       not null,
    updated_by        uuid references app_user (id),
    created_at        bigint  default extract(epoch from now()) * 1000000,
    updated_at        bigint  default extract(epoch from now()) * 1000000
);