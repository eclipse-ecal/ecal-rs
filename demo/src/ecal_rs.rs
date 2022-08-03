/// topic: /kpns/test/ping
#[derive(Clone, PartialEq, ::prost::Message, ecal::Message)]
#[type_prefix = "kpns_msgs."]
pub struct Ping {
    #[prost(uint64, tag = "1")]
    pub sync: u64,
}
/// topic: /kpns/test/pong
#[derive(Clone, PartialEq, ::prost::Message, ecal::Message)]
#[type_prefix = "kpns_msgs."]
pub struct Pong {
    #[prost(uint64, tag = "1")]
    pub sync: u64,
}
