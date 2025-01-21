use std::collections::HashSet;

pub struct RedisPubSub {
    pub subscribed_channels: HashSet<i32>,
}

impl RedisPubSub {
    pub async fn subscribe(
        &mut self,
        client: &redis::Client,
        channels_to_subscribe: Vec<i32>,
    ) -> bool {
        let connection_res = client.get_connection();
        if connection_res.is_err() {
            return false;
        }
        let mut connection = connection_res.unwrap();
        let mut pubsub = connection.as_pubsub();
        for channel in channels_to_subscribe.iter() {
            if !self.subscribed_channels.contains(channel) {
                let subscription_res = pubsub.subscribe(channel);
                if subscription_res.is_err() {
                    println!("Issue subscribing to {:?}", channel);
                } else {
                    self.subscribed_channels.insert(*channel);
                }
            }
        }
        true
    }
}
