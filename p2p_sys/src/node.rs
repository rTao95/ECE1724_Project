use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::{io, io::AsyncBufReadExt, select};

mod file_operations; // Include the new module

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub async fn run_peer_to_peer_system(
    topic_name: String,
    password: String,
) -> Result<(), Box<dyn Error>> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(tcp::Config::default(), libp2p::noise::Config::new, || {
            yamux::Config::default()
        })?
        .with_quic()
        .with_behaviour(|key| {
            let local_peer_id = key.public().to_peer_id();

            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(message_id_fn)
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id.clone())?;
            Ok(MyBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let topic = gossipsub::IdentTopic::new(&topic_name);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    let local_peer_id = *swarm.local_peer_id();
    println!("Local Peer ID: {}", local_peer_id);

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!(
        "Listening for peers and publishing to topic: {}",
        topic_name
    );

    loop {
        select! {
            // Handle user input
            line = stdin.next_line() => {
                match line {
                    Ok(Some(input)) => {
                        handle_user_input(&input, &mut swarm, &topic, &local_peer_id, &password).await?;
                    },
                    Ok(None) => break Ok(()),
                    Err(e) => {
                        println!("Error reading stdin: {}", e);
                    }
                }
            }

            // Handle swarm events
            event = swarm.select_next_some() => {
                handle_swarm_event(event, &mut swarm, &password).await?;
            }
        }
    }
}

// Helper function to handle user input
async fn handle_user_input(
    input: &str,
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    topic: &gossipsub::IdentTopic,
    local_peer_id: &PeerId,
    password: &str,
) -> Result<(), Box<dyn Error>> {
    if input.trim() == "@upload" {
        // Call the function to select and broadcast a file
        if let Some(file_path) = file_operations::select_file() {
            match file_operations::read_file_to_bytes(&file_path) {
                Ok(file_data) => {
                    // Get the file name
                    let file_name = file_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Create a FileMessage with the password
                    let file_message = FileMessage {
                        sender: (*local_peer_id).to_string(),
                        password: password.to_string(),
                        file_name,
                        data: file_data,
                    };

                    // Serialize the FileMessage
                    let serialized = serde_json::to_vec(&file_message)?;

                    // Publish the serialized file message
                    if let Err(e) = swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(topic.clone(), serialized)
                    {
                        println!("Publish error: {:?}", e);
                    } else {
                        println!("File broadcasted to peers.");
                    }
                }
                Err(e) => println!("Failed to read file: {}", e),
            }
        } else {
            println!("No file selected for upload.");
        }
    } else {
        // Regular text message with password prepended
        let message_with_password = format!("{}:{}", password, input);
        if let Err(e) = swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), message_with_password.as_bytes())
        {
            println!("Publish error: {:?}", e);
        }
    }
    Ok(())
}

// Helper function to handle swarm events
async fn handle_swarm_event(
    event: SwarmEvent<MyBehaviourEvent>,
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    password: &str,
) -> Result<(), Box<dyn Error>> {
    match event {
        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, _) in list {
                println!("mDNS discovered a new peer: {}", peer_id);
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            }
        }
        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
            for (peer_id, _) in list {
                println!("mDNS discovered peer has expired: {}", peer_id);
                swarm
                    .behaviour_mut()
                    .gossipsub
                    .remove_explicit_peer(&peer_id);
            }
        }
        SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
            propagation_source: peer_id,
            message_id: id,
            message,
        })) => {
            handle_received_message(&peer_id, &id, &message, password).await?;
        }
        SwarmEvent::NewListenAddr { address, .. } => {
            println!("Local node is listening on {}", address);
        }
        _ => {}
    }
    Ok(())
}

// Helper function to handle received messages
async fn handle_received_message(
    peer_id: &PeerId,
    message_id: &gossipsub::MessageId,
    message: &gossipsub::Message,
    password: &str,
) -> Result<(), Box<dyn Error>> {
    // Attempt to deserialize the message as FileMessage
    if let Ok(file_message) = serde_json::from_slice::<FileMessage>(&message.data) {
        if file_message.password == password {
            // Save the received file
            let peer_dir = format!("./{}/", peer_id);
            std::fs::create_dir_all(&peer_dir)?;
            let file_path = format!("{}/{}", peer_dir, file_message.file_name);
            std::fs::write(&file_path, &file_message.data)?;
            println!(
                "Received file '{}' from peer {} and saved to '{}'",
                file_message.file_name, peer_id, file_path
            );
        } else {
            println!("Received file with invalid password from peer: {}", peer_id);
        }
    } else {
        // Treat as a regular text message
        let message_content = String::from_utf8_lossy(&message.data);
        if message_content.starts_with(password) {
            let actual_message = &message_content[password.len() + 1..]; // Skip the password and the colon
            println!(
                "Got message: '{}' with id: {} from peer: {}",
                actual_message, message_id, peer_id
            );
        } else {
            println!(
                "Received message with invalid password from peer: {}",
                peer_id
            );
        }
    }
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FileMessage {
    sender: String,
    password: String,
    file_name: String,
    data: Vec<u8>,
}
