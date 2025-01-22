use std::collections::HashSet;

use redis::aio::PubSubSink;

pub struct RedisPubSub {
    pub subscribed_channels: HashSet<i32>,
    pub pubsub_sink: PubSubSink,
}

impl RedisPubSub {
    pub async fn subscribe(&mut self, channels_to_subscribe: Vec<i32>) -> bool {
        {
            for channel in channels_to_subscribe.iter() {
                if !self.subscribed_channels.contains(channel) {
                    let subscription_res = self.pubsub_sink.subscribe(channel).await;
                    if subscription_res.is_err() {
                        println!("Issue subscribing to {:?}", channel);
                    } else {
                        self.subscribed_channels.insert(*channel);
                    }
                }
            }
        }
        true
    }
}
