create table pagination_data_cache
(
    xx_hash bigint primary key,
    total_pages integer not null,
    total_count integer not null,
    created_at  bigint default extract(epoch from now()) * 1000000,
    expire_at   bigint  not null
);