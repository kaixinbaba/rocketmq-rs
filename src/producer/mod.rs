use std::fmt;
use std::time::Duration;

use crate::client::{Client, ClientOptions, InnerProducer};
use crate::message::MessageExt;
use crate::namesrv::NameServer;
use crate::resolver::{HttpResolver, NsResolver, PassthroughResolver};
use crate::Error;
use selector::{QueueSelector, RoundRobinQueueSelector};

pub mod selector;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum PullStatus {
    Found = 0,
    NoNewMsg = 1,
    NoMsgMatched = 2,
    OffsetIllegal = 3,
    BrokerTimeout = 4,
}

#[derive(Debug, Clone)]
pub struct PullResult {
    pub next_begin_offset: i64,
    pub min_offset: i64,
    pub max_offset: i64,
    pub status: PullStatus,
    pub suggest_which_broker_id: i64,
    pub message_exts: Vec<MessageExt>,
    pub body: Vec<u8>,
}

pub struct ProducerOptions {
    selector: Box<dyn QueueSelector>,
    send_msg_timeout: Duration,
    default_topic_queue_nums: usize,
    create_topic_key: String,
    resolver: Box<dyn NsResolver>,
}

impl fmt::Debug for ProducerOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProducerOptions")
            .field("send_msg_timeout", &self.send_msg_timeout)
            .field("default_queue_nums", &self.default_topic_queue_nums)
            .field("create_topic_key", &self.create_topic_key)
            .field("resolver", &self.resolver.description())
            .finish()
    }
}

impl Default for ProducerOptions {
    fn default() -> ProducerOptions {
        Self {
            selector: Box::new(RoundRobinQueueSelector::new()),
            send_msg_timeout: Duration::from_secs(3),
            default_topic_queue_nums: 4,
            create_topic_key: "TBW102".to_string(),
            resolver: Box::new(HttpResolver::new("DEFAULT".to_string())),
        }
    }
}

impl ProducerOptions {
    pub fn new() -> Self {
        ProducerOptions::default()
    }

    pub fn set_send_msg_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.send_msg_timeout = timeout;
        self
    }

    pub fn set_default_topic_queue_nums(&mut self, queue_nums: usize) -> &mut Self {
        self.default_topic_queue_nums = queue_nums;
        self
    }

    pub fn set_create_topic_key(&mut self, key: &str) -> &mut Self {
        self.create_topic_key = key.to_string();
        self
    }

    pub fn set_resolver<R: NsResolver + 'static>(&mut self, resolver: R) -> &mut Self {
        self.resolver = Box::new(resolver);
        self
    }

    pub fn set_name_server(&mut self, addrs: Vec<String>) -> &mut Self {
        self.resolver = Box::new(PassthroughResolver::new(
            addrs,
            HttpResolver::new("DEFAULT".to_string()),
        ));
        self
    }

    pub fn set_name_server_domain(&mut self, url: &str) -> &mut Self {
        self.resolver = Box::new(HttpResolver::with_domain(
            "DEFAULT".to_string(),
            url.to_string(),
        ));
        self
    }
}

/// RocketMQ producer
pub struct Producer {
    group: String,
    options: ProducerOptions,
    client: Client<Self, crate::consumer::Consumer, Box<dyn NsResolver>>,
}

impl Producer {
    pub fn new(group: &str) -> Result<Self, Error> {
        Self::with_options(group, ProducerOptions::new())
    }

    pub fn with_options(group: &str, options: ProducerOptions) -> Result<Self, Error> {
        let client_options = ClientOptions::new(group);
        let name_server = NameServer::new(dyn_clone::clone_box(&*options.resolver))?;
        Ok(Self {
            group: group.to_string(),
            options,
            client: Client::new(client_options, name_server),
        })
    }
}

impl fmt::Debug for Producer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Producer")
            .field("group", &self.group)
            .field("options", &self.options)
            .finish()
    }
}

impl InnerProducer for Producer {
    fn publish_topic_list(&self) -> Vec<String> {
        unimplemented!()
    }

    fn update_topic_publish_info(&self) {
        unimplemented!()
    }

    fn is_publish_topic_need_update(&self, topic: &str) -> bool {
        unimplemented!()
    }
}
