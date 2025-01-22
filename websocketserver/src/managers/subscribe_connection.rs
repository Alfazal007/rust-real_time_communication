use std::collections::HashSet;

use redis::aio::PubSubSink;

pub struct RedisPubSub {
    pub subscribed_channels: HashSet<i32>,
    pub pubsub_sink: PubSubSink,
}

impl RedisPubSub {
    pub async fn subscribe(&mut self, channels_to_subscribe: Vec<i32>) -> bool {
        println!("need to subscribe");
        {
            for channel in channels_to_subscribe.iter() {
                println!("Trying to subscribe to {:?}", channel);
                if !self.subscribed_channels.contains(channel) {
                    println!("In here 1");
                    let subscription_res = self.pubsub_sink.subscribe(channel).await;
                    println!("In here 2");
                    if subscription_res.is_err() {
                        println!("Issue subscribing to {:?}", channel);
                    } else {
                        println!("connected");
                        self.subscribed_channels.insert(*channel);
                    }
                }
            }
        }
        true
    }
}
