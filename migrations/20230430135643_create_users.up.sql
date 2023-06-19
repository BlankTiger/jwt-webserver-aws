-- Add up migration script here
create type user_roles as enum ('admin', 'customer', 'guest');

create table users (
	id serial primary key,
	name varchar(255) not null,
	passwd_hash varchar(255) not null,
	role user_roles not null default 'guest'
);
