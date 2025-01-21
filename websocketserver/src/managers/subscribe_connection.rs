use std::collections::HashSet;

pub struct RedisPubSub {
    pub subscribed_channels: HashSet<i32>,
}

impl RedisPubSub {
    pub async fn subscribe(
        &mut self,
        pubsub: &mut redis::aio::PubSub,
        channels_to_subscribe: Vec<i32>,
    ) -> bool {
        println!("need to subscribe");
        {
            println!("2");
            for channel in channels_to_subscribe.iter() {
                if !self.subscribed_channels.contains(channel) {
                    let subscription_res = pubsub.subscribe(channel).await;
                    if subscription_res.is_err() {
                        println!("Issue subscribing to {:?}", channel);
                    } else {
                        println!("connected");
                        self.subscribed_channels.insert(*channel);
                    }
                }
            }
            println!("2");
        }
        true
    }
}
