create table messages (
	id serial primary key,
	sender_id int references users(id) not null,
	channel_id int references channel(id) not null,
	message text not null
);
