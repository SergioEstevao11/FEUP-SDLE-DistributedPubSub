use std::collections::{HashMap, VecDeque};
use rpubsub::Topic;


#[derive(Debug)]
struct Update {
    content: String,
    pending_updates: usize
}

type UpdatesQueue = VecDeque<Update>;

#[derive(Debug)]
struct SubscriptionInfo {
    last_recv_sequence_num:    Option<u128>,
    topic_update_idx: Option<usize>
}

#[derive(Debug)]
pub struct TopicInfo {
    subscriptions: HashMap<String, SubscriptionInfo>,
    update_queue: UpdatesQueue
}

impl TopicInfo {
    pub fn new() -> Self {
        let subs = HashMap::new();
        let queue = UpdatesQueue::new();
        Self { subscriptions: subs, update_queue: queue}
    }

    pub fn remove_subscription_info(&mut self, ip: &String) {
        self.subscriptions.remove(ip);
    }
}

pub enum Error {
    NOTOPIC,
    NOSUB,
    ALREASUB
}

//#[derive(Debug)]
/*pub struct TopicsState {
    topics: HashMap<Topic, TopicInfo>
}*/
pub type TopicsState = HashMap<Topic, TopicInfo>;


pub fn add_topic(state: &mut TopicsState, topic: &Topic) {
    state.insert(topic.clone(), TopicInfo::new());
}

fn topic_subscriber_num(topic_info: &TopicInfo) -> usize {
    topic_info.subscriptions.len()
}

fn remove_nonpending_updates(queue: &mut UpdatesQueue) {
    //while !queue.is_empty() && queue.back_mut().unwrap().pending_updates == 0 {
    //    queue.pop_back();
    while !queue.is_empty() && queue.front().as_mut().unwrap().pending_updates == 0 {
        queue.pop_front();
    }
}


fn latest_update_idx_in_topic(topic_info: &mut TopicInfo) -> Option<usize> {
    let size = topic_info.update_queue.len();
    if size == 0 { None } else { Some(size-1) }
}

/*fn latest_update_idx_in_topic(topic_info: &mut TopicInfo) -> Option<usize> {
    if topic_info.update_queue.len() == 0 { None } else { Some(0) }
}*/


fn associate_subscribers_to_last_update(topic_info: &mut TopicInfo) {
    let idx = latest_update_idx_in_topic(topic_info);

    let subscriptions = topic_info.subscriptions.values_mut();
    for subscription_info in subscriptions {
        if subscription_info.topic_update_idx.is_none() {
            subscription_info.topic_update_idx = idx;
        }
    }
}
/*fn associate_subscribers_to_last_update(topic_info: &mut TopicInfo) {
    let idx = latest_update_idx_in_topic(topic_info);

    let subscriptions = topic_info.subscriptions.values_mut();
    for subscription_info in subscriptions {
        if subscription_info.topic_update_idx.is_none() {
            subscription_info.topic_update_idx = idx;
        }
    }
}*/


pub fn add_subscription(state: &mut TopicsState, topic: &Topic, ip: &String) -> Result<(), Error> {
    if !state.contains_key(topic) {
        add_topic(state, topic);
    }

    let topic_info = state.get_mut(topic).unwrap();
    let subscription = topic_info.subscriptions.get_mut(ip);
    match subscription {
        Some(_) => Err(Error::ALREASUB),

        None => {
            // Receives only updates inserted after his subscription
            topic_info.subscriptions.insert(ip.clone(), SubscriptionInfo { last_recv_sequence_num: None, topic_update_idx: None });
            Ok(())
        }
    }
}

pub fn remove_subscription(state: &mut TopicsState, topic: &Topic, ip: &String) -> Result<(), Error> {
    if !state.contains_key(topic) {
        return Err(Error::NOTOPIC);
    }

    let topic_info = state.get_mut(topic).unwrap();
    let subscription = topic_info.subscriptions.get_mut(ip);

    match subscription {
        Some(subscription_info) => {
            let idx = subscription_info.topic_update_idx;

            topic_info.remove_subscription_info(ip);

            let update_queue = &mut topic_info.update_queue;

            if idx.is_none() {
                return Ok(());
            }

            for i in idx.unwrap()..update_queue.len() {
                //if update_queue.get(i).unwrap().pending_updates != 0 {
                    update_queue.get_mut(i).unwrap().pending_updates -= 1;
                //}
            }

            /*for i in idx.unwrap()..0 {
                if update_queue.get(i).unwrap().pending_updates != 0 {
                    update_queue.get_mut(i).unwrap().pending_updates -= 1;
                }
            }*/
            remove_nonpending_updates(update_queue);
           
            Ok(())
        },

        None => Err(Error::NOSUB)
    }
}



pub fn add_update(state: &mut TopicsState, topic: &Topic, content: &String) -> Result<(), Error> {
    if !state.contains_key(topic) {
        return Err(Error::NOTOPIC);
    }

    let topic_info = state.get_mut(topic).unwrap();
    let sub_num = topic_subscriber_num(&topic_info);
    let update = Update { content: content.clone(), pending_updates: sub_num };
    let queue = &mut topic_info.update_queue;
    queue.push_back(update);
    //queue.push_front(update);
    // when a topic doesnt have an update and gets one, update all None topic_update_idxs
    associate_subscribers_to_last_update(topic_info);

    Ok(())
}

pub fn update_subscriber_update_ack(state: &mut TopicsState, topic: &Topic, ip: &String, sequence_num: u128) -> Result<Option<usize>, Error> {
    if !state.contains_key(topic) {
        return Err(Error::NOTOPIC);
    }

    let topic_info = state.get_mut(topic).unwrap();
    let subscription_info = topic_info.subscriptions.get_mut(ip);

    let queue = &mut topic_info.update_queue;
    let mut ret_idx: Option<usize> = subscription_info.as_ref().unwrap().topic_update_idx.clone();

    return match subscription_info {
        Some(subscription_info) => {
            if subscription_info.last_recv_sequence_num.is_none() {
                subscription_info.last_recv_sequence_num = Some(sequence_num);
            } else if sequence_num == subscription_info.last_recv_sequence_num.unwrap()+1 {
                subscription_info.last_recv_sequence_num = Some(sequence_num);

                
                if !subscription_info.topic_update_idx.is_none() {
                    let idx = subscription_info.topic_update_idx.unwrap();
                    let pending_updates = queue.get(idx).unwrap().pending_updates.clone()-1;
                    queue.get_mut(idx).unwrap().pending_updates -= 1;

                    if idx+1 == queue.len() {
                        subscription_info.topic_update_idx = None;
                        ret_idx = None;
                    } else {
                        subscription_info.topic_update_idx = Some(idx+1);
                        ret_idx = Some(idx+1);
                    }

                    if idx == 0 && pending_updates == 0 {
                        queue.pop_front();
                        for info in &mut topic_info.subscriptions {
                            if info.1.topic_update_idx.is_none() {
                                continue
                            }
                            println!("{} : {}", info.1.topic_update_idx.unwrap(), info.0 );
                            info.1.topic_update_idx = Some(info.1.topic_update_idx.unwrap()-1);

                            if info.0.eq(ip) {
                                ret_idx = info.1.topic_update_idx;
                            }
                        }

                    }

                }
/*                 if !subscription_info.topic_update_idx.is_none() {
                    let idx = subscription_info.topic_update_idx.unwrap();
                    let pending_updates = queue.get(idx).unwrap().pending_updates.clone()-1;
                    queue.get_mut(idx).unwrap().pending_updates -= 1;

                    if idx == queue.len()-1 && pending_updates == 0 {
                        queue.pop_back();
                    }

                    if idx == 0 {
                        subscription_info.topic_update_idx = None;
                    } else {
                        subscription_info.topic_update_idx = Some(idx-1);
                    }
                }*/
            }
            Ok(ret_idx)
        },

        None => Err(Error::NOSUB)
    }
}

pub fn get_next_subscriber_update(state: &mut TopicsState, topic: &Topic, ip: &String, sequence_num: u128) -> Result<Option<String>, Error> {
    match update_subscriber_update_ack(state, topic, ip, sequence_num) {
        Ok(topic_update_idx) => {
            Ok(match topic_update_idx {
                Some(idx) => {println!("{}", idx);
                    Some(state.get_mut(topic).unwrap().update_queue.get(idx).unwrap().content.clone())
                }None => None,
            })
        },
        Err(e) => Err(e),
    }

}