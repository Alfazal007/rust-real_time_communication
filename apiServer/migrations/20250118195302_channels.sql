create table channel (
	id serial primary key,
	name varchar(20) unique not null,
	admin_id int references users(id) not null
);
