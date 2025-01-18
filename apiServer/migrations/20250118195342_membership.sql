create table membership (
	user_id int references users(id),
	channel_id int references channel(id),
    primary key (user_id, channel_id)
);
