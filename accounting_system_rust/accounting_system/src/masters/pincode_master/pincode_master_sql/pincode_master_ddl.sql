create table pincode_master
(
    id      uuid primary key,
    pincode    varchar(20),--if india then integer otherwise varchar
    city_id uuid references city_master (id),
    created_by uuid not null references app_user (id),
    updated_by uuid references app_user (id),
    created_at bigint default extract(epoch from now()) * 1000000,
    updated_at bigint default extract(epoch from now()) * 1000000,
    country_id uuid references country_master (id)
);
