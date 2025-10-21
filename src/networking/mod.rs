mod ecs;
mod messages;
mod sockets;

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::networking::messages::NetPacketBody;

    use super::*;

    #[test]
    fn aaa() {
        let mut server = sockets::NetServer::new(Some(5555)).unwrap();
        let mut client = sockets::NetClient::new("127.0.0.1:5555".parse().unwrap()).unwrap();

        let client_message = messages::NetMessage {
            header: messages::NetHeader {},
            body: vec![NetPacketBody::Ping(Cow::Borrowed(b"hello"))],
        };

        client
            .node
            .send(&client_message, client.server_addr)
            .unwrap();

        let msg = server.node.recv().unwrap();
        println!("server sees message: {:?}", msg);
    }
}
