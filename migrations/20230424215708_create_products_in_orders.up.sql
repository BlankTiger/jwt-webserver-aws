-- Add up migration script here
create table if not exists products_in_orders (
	order_id serial references orders(id),
	product_id serial references products(id),
	quantity integer not null,
	primary key(order_id, product_id)
);
