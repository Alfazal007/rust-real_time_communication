create table users (
	id serial primary key,
	username varchar(20) unique not null,
	password varchar(255) not null
);
