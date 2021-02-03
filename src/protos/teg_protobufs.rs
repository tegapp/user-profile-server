/// Combinators send CombinatorMessages
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InviteCode {
    #[prost(bytes, tag="1")]
    pub secret: std::vec::Vec<u8>,
    #[prost(bytes, tag="2")]
    pub host_public_key: std::vec::Vec<u8>,
}
