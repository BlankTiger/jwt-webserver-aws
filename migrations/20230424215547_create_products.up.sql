-- Add up migration script here
-- create products table in sqlite3, id, name, price, available
-- create table products (
-- 	id integer not null primary key autoincrement,
-- 	name text not null,
-- 	price real not null,
-- 	available boolean not null
-- );

-- create products table in postgresql
create table products (
	id serial primary key,
	name text not null,
	price int not null,
	available boolean not null
);
