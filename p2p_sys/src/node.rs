use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::Swarm, swarm::SwarmEvent, tcp, yamux,
    PeerId,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use std::error::Error;

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub struct Node {
    pub swarm: Swarm<MyBehaviour>,
    pub topic: gossipsub::IdentTopic,
}

impl Node {
    /// Initializes the node with Gossipsub and mDNS behavior.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|key| {
                // Use message hash as an ID for content-addressing.
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };

                // Gossipsub configuration.
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .build()
                    .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))?;

                let gossipsub =
                    gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(key.clone()), gossipsub_config)?;

                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
                Ok(MyBehaviour { gossipsub, mdns })
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Create a Gossipsub topic and subscribe to it.
        let topic = gossipsub::IdentTopic::new("test-net");
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        Ok(Node { swarm, topic })
    }
}
