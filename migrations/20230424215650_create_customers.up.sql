-- Add up migration script here
create table if not exists customers (
	id serial primary key,
	name text not null,
	address text not null
);
