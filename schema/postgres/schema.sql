create table if not exists tenant(
id serial primary key,
display_name varchar(100) not null,
created_by varchar(50) not null,
updated_by varchar(50),
created_at bigint default extract(epoch from now())*1000000,
updated_at bigint default extract(epoch from now())*1000000
);

create table if not exists currency_master(
    id smallserial primary key,
    tenant_id integer not null references tenant(id),
    scale smallint not null,
    display_name varchar(16) not null,
    description varchar(50),
    created_by varchar(50) not null,
    updated_by varchar(50),
    created_at bigint default extract(epoch from now())*1000000,
    updated_at bigint default extract(epoch from now())*1000000
);

create table if not exists account_type_master(
    id smallserial primary key,
    tenant_id integer not null references tenant(id),
    scale smallint not null,
    display_name varchar(30) not null,
    account_code smallint not null,
    created_by varchar(50) not null,
    updated_by varchar(50),
    created_at bigint default extract(epoch from now())*1000000,
    updated_at bigint default extract(epoch from now())*1000000
);

create table if not exists app_user(
    id serial primary key,
    tenant_id integer not null references tenant(id),
    first_name varchar(50) not null,
    last_name varchar(50),
    email_id varchar(200),
    mobile_number varchar(14),
    created_by varchar(50) not null,
    updated_by varchar(50),
    created_at bigint default extract(epoch from now())*1000000,
    updated_at bigint default extract(epoch from now())*1000000
);

create table user_account(
 id serial primary key,
 tenant_id integer not null references tenant(id),
 account_type_id smallint not null,
 user_id integer not null references app_user(id),
 created_by varchar(50) not null,
 updated_by varchar(50),
 created_at bigint default extract(epoch from now())*1000000,
 updated_at bigint default extract(epoch from now())*1000000
);

create table ledger_master(
  id serial primary key,
  tenant_id integer not null references tenant(id),
  display_name varchar(50),
  debit_account_id integer not null references user_account(id),
  credit_account_id integer not null references user_account(id),
  currency_master_id smallint not null references currency_master(id),
  created_by varchar(50) not null,
  updated_by varchar(50),
  created_at bigint default extract(epoch from now())*1000000,
  updated_at bigint default extract(epoch from now())*1000000
);
