use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NetMessage<'data> {
    pub(crate) header: NetHeader,
    pub(crate) body: Vec<NetPacketBody<'data>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetHeader {}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetPacketBody<'data> {
    Ping(Cow<'data, [u8]>),
    Pong(Cow<'data, [u8]>),
}
