use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId,
};
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use rand::prelude::IteratorRandom;
use std::collections::HashMap;
use tokio::{io, io::AsyncBufReadExt, select}; // For timestamps
mod chunker;
mod file_operations; // Include the new module
mod storage_manager;
mod utils;

type SharedFileTransferLogs = Arc<Mutex<Vec<FileTransferLog>>>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FileTransferLog {
    peer_id: String,
    file_name: String,
    timestamp: DateTime<Utc>,
    password: String,
}

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

fn search_transfer_log(
    logs: &Vec<FileTransferLog>,
    peer_id: Option<&str>,
    file_name: Option<&str>,
    date: Option<DateTime<Utc>>,
) -> Vec<FileTransferLog> {
    logs.iter()
        .filter(|log| {
            peer_id.map_or(true, |id| log.peer_id == id)
                && file_name.map_or(true, |name| log.file_name.contains(name))
                && date.map_or(true, |d| log.timestamp.date_naive() == d.date_naive())
        })
        .cloned()
        .collect()
}

pub async fn run_peer_to_peer_system(
    topic_name: String,
    password: String,
) -> Result<(), Box<dyn Error>> {
    let mut peer_scores: HashMap<PeerId, f64> = HashMap::new(); // Score matrix
    let file_transfer_logs: SharedFileTransferLogs = Arc::new(Mutex::new(Vec::new())); // Shared logs

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(tcp::Config::default(), libp2p::noise::Config::new, || {
            yamux::Config::default()
        })?
        .with_quic()
        .with_behaviour(|key| {
            let local_peer_id = key.public().to_peer_id();

            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = std::collections::hash_map::DefaultHasher::new();
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
                        handle_user_input(&input, &mut swarm, &topic, &local_peer_id, &password, &mut peer_scores, Arc::clone(&file_transfer_logs)).await?;
                    },
                    Ok(None) => break Ok(()),
                    Err(e) => {
                        println!("Error reading stdin: {}", e);
                    }
                }
            }

            // Handle swarm events
            event = swarm.select_next_some() => {
                handle_swarm_event(event, &mut swarm, &password, &mut peer_scores, &local_peer_id,&topic, Arc::clone(&file_transfer_logs)).await?;
            }
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                // Periodic cleanup and score decay
                remove_disconnected_peers_and_decay_scores(&swarm, &mut peer_scores);
            }
        }
    }
}

/// Removes disconnected peers from the peer_scores and decreases scores for all peers.
fn remove_disconnected_peers_and_decay_scores(
    swarm: &libp2p::Swarm<MyBehaviour>,
    peer_scores: &mut HashMap<PeerId, f64>,
) {
    let mut disconnected_peers = Vec::new();

    // Identify disconnected peers
    for peer_id in peer_scores.keys() {
        if !swarm.is_connected(peer_id) {
            disconnected_peers.push(peer_id.clone());
        }
    }

    // Remove disconnected peers
    for peer_id in &disconnected_peers {
        println!("Removing disconnected peer: {}", peer_id);
        peer_scores.remove(peer_id);
    }

    // Decrease scores for all remaining peers
    for (peer_id, score) in peer_scores.iter_mut() {
        *score -= 0.1; // Decrease by 0.1 (adjust as needed)
        if *score < 0.0 {
            *score = 0.0; // Ensure scores don't drop below zero
        }
        println!("Updated score for {}: {}", peer_id, score);
    }
}

// Helper function to update scores
fn update_peer_score(peer_scores: &mut HashMap<PeerId, f64>, peer_id: &PeerId, delta: f64) {
    let score = peer_scores.entry(peer_id.clone()).or_insert(0.0);
    *score += delta;
    println!("Updated score for {}: {}", peer_id, *score);
}

fn select_peers(peer_scores: &HashMap<PeerId, f64>, n: usize) -> Vec<PeerId> {
    // Sort peers by score in descending order
    let mut peers_by_score: Vec<_> = peer_scores.iter().collect();
    peers_by_score.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    // Select up to n-1 top peers
    let mut selected_peers: Vec<PeerId> = peers_by_score
        .iter()
        .take(n.saturating_sub(1))
        .map(|(peer_id, _)| (*peer_id).clone())
        .collect();

    // Add a random peer if available
    let remaining_peers: Vec<_> = peer_scores
        .keys()
        .filter(|peer_id| !selected_peers.contains(peer_id))
        .cloned()
        .collect();

    if let Some(random_peer) = remaining_peers.into_iter().choose(&mut rand::thread_rng()) {
        selected_peers.push(random_peer);
    }

    // If there are fewer peers than requested, return all available
    selected_peers.truncate(n);
    selected_peers
}

// Handle user input with score updates
async fn handle_user_input(
    input: &str,
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    topic: &gossipsub::IdentTopic,
    local_peer_id: &PeerId,
    password: &str,
    peer_scores: &mut HashMap<PeerId, f64>,
    file_transfer_logs: SharedFileTransferLogs,
) -> Result<(), Box<dyn Error>> {
    if input.trim() == "@upload" {
        // Select peers for file distribution
        let target_peers = select_peers(peer_scores, 3); // Default n = 3

        if target_peers.is_empty() {
            println!("No valid peers to send the file.");
            return Ok(());
        }

        if let Some(file_path) = file_operations::select_file() {
            let file_name = file_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();

            let chunks_metadata = chunker::chunk_file(file_path.to_str().unwrap())?;
            let total_chunks = chunks_metadata.len();
            if total_chunks == 0 {
                println!("No chunks created, file might be empty!");
                return Ok(());
            }

            let local_peer_dir = format!("./{}", local_peer_id);
            std::fs::create_dir_all(&local_peer_dir)?;

            let mut file = std::fs::File::open(&file_path)?;

            // Read, save and send each chunk
            for (i, metadata) in chunks_metadata.iter().enumerate() {
                let mut buffer = vec![0; metadata.size];
                file.read_exact(&mut buffer)?;

                let chunk_filename = format!("{}({}-of-{})", file_name, i + 1, total_chunks);

                // Save chunk locally
                storage_manager::save_chunk(&buffer, &metadata, &local_peer_dir, &chunk_filename)?;

                // Prepare a FileMessage for this chunk
                let file_message = FileMessage {
                    sender: (*local_peer_id).to_string(),
                    password: password.to_string(),
                    file_name: chunk_filename.clone(),
                    data: buffer,
                };

                let serialized = serde_json::to_vec(&file_message)?;
                for peer_id in &target_peers {
                    if let Err(e) =
                        send_file_to_peer(swarm, peer_id, topic, serialized.clone()).await
                    {
                        println!("Failed to send file chunk to peer {}: {:?}", peer_id, e);
                    } else {
                        println!("File chunk '{}' sent to peer: {}", chunk_filename, peer_id);
                    }
                }
            }
        } else {
            println!("No file selected for upload.");
        }
    } else if input.trim() == "@check_scores" {
        println!("Current peer scores:");
        for (peer_id, score) in peer_scores {
            println!("Peer ID: {}, Score: {}", peer_id, score);
        }
    } else if input.trim() == "@check_logs" {
        let logs = file_transfer_logs.lock().unwrap();
        let results = search_transfer_log(&logs, None, None, None);
        for log in results {
            println!("Log: {:?}", log);
        }
    } else {
        // Handle regular messages
        let message_with_password = format!("{}:{}", password, input);
        if let Err(e) = swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), message_with_password.as_bytes())
        {
            println!("Publish error: {:?}", e);
        } else {
            println!("Message sent: {}", input);
        }
    }
    Ok(())
}

async fn send_file_to_peer(
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    target_peer_id: &PeerId,
    topic: &gossipsub::IdentTopic,
    serialized_message: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    // Ensure the peer is connected
    if !swarm.is_connected(target_peer_id) {
        println!("Peer {} is not connected.", target_peer_id);
        return Err("Target peer is not connected".into());
    }

    // Add the target peer as an explicit peer
    swarm
        .behaviour_mut()
        .gossipsub
        .add_explicit_peer(target_peer_id);
    println!("Added peer {} as an explicit peer.", target_peer_id);

    // Wait briefly
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Attempt to send the file message
    let result = swarm
        .behaviour_mut()
        .gossipsub
        .publish(topic.clone(), serialized_message);

    // Remove the explicit peer
    swarm
        .behaviour_mut()
        .gossipsub
        .remove_explicit_peer(target_peer_id);

    match result {
        Ok(_) => {
            println!("File successfully sent to peer {}.", target_peer_id);
            Ok(())
        }
        Err(e) => {
            println!("Failed to send file to peer {}: {:?}", target_peer_id, e);
            Err(e.into())
        }
    }
}

async fn handle_swarm_event(
    event: SwarmEvent<MyBehaviourEvent>,
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    password: &str,
    peer_scores: &mut HashMap<PeerId, f64>,
    local_peer_id: &PeerId,
    topic: &gossipsub::IdentTopic,
    file_transfer_logs: SharedFileTransferLogs,
) -> Result<(), Box<dyn Error>> {
    match event {
        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, _) in list {
                println!("mDNS discovered a new peer: {}", peer_id);
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                peer_scores.entry(peer_id).or_insert(0.5);
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
            handle_received_message(
                &peer_id,
                &id,
                &message,
                password,
                peer_scores,
                local_peer_id,
                swarm,
                topic,
                file_transfer_logs,
            )
            .await?;
        }
        SwarmEvent::NewListenAddr { address, .. } => {
            println!("Local node is listening on {}", address);
        }
        _ => {}
    }
    Ok(())
}

async fn handle_received_message(
    peer_id: &PeerId,
    message_id: &gossipsub::MessageId,
    message: &gossipsub::Message,
    password: &str,
    peer_scores: &mut HashMap<PeerId, f64>,
    local_peer_id: &PeerId,
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    topic: &gossipsub::IdentTopic,
    file_transfer_logs: SharedFileTransferLogs,
) -> Result<(), Box<dyn Error>> {
    if let Ok(file_message) = serde_json::from_slice::<FileMessage>(&message.data) {
        if file_message.password == password {
            let local_peer_id_str = format!("./{}/", local_peer_id);
            std::fs::create_dir_all(&local_peer_id_str)?;
            let file_path = format!("{}/{}", local_peer_id_str, file_message.file_name);
            std::fs::write(&file_path, &file_message.data)?;

            println!(
                "Received file chunk '{}' from peer {} and saved to '{}'",
                file_message.file_name, peer_id, file_path
            );

            // Increment the peer's score
            update_peer_score(peer_scores, peer_id, 1.0);

            let log_message = FileTransferLog {
                peer_id: local_peer_id.to_string(),
                file_name: file_message.file_name.clone(),
                timestamp: chrono::Utc::now(),
                password: password.to_string(),
            };

            if let Ok(serialized_log) = serde_json::to_vec(&log_message) {
                if let Err(e) = swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topic.clone(), serialized_log)
                {
                    println!("Failed to publish file log info: {:?}", e);
                } else {
                    println!("File log info published to topic.");
                }
            }
        } else {
            println!("Received file with invalid password from peer: {}", peer_id);
            update_peer_score(peer_scores, peer_id, -0.5);
        }
    } else if let Ok(log_message) = serde_json::from_slice::<FileTransferLog>(&message.data) {
        // Update the file transfer logs
        if log_message.password == password {
            let mut logs = file_transfer_logs.lock().unwrap();
            logs.push(FileTransferLog {
                peer_id: log_message.peer_id.clone(),
                file_name: log_message.file_name.clone(),
                timestamp: log_message.timestamp,
                password: log_message.password,
            });
            println!(
                "Updated logs with new entry: peer_id={}, file_name={}",
                log_message.peer_id, log_message.file_name
            );
        } else {
            println!(
                "Received file transfer log with invalid password from peer: {}",
                peer_id
            );
        }
    } else {
        // Treat as a regular text message
        let message_content = String::from_utf8_lossy(&message.data);
        if message_content.starts_with(password) {
            let actual_message = &message_content[password.len() + 1..]; // Skip password and colon
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
