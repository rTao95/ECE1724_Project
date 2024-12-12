use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId,
};
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::io::Write;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use rand::prelude::IteratorRandom;
use std::collections::HashMap;
use std::fs;
use tokio::{io, io::AsyncBufReadExt, select};
use rand::Rng;
mod chunker;
mod file_operations;
mod storage_manager;
mod utils;

type SharedFileTransferLogs = Arc<Mutex<Vec<FileTransferLog>>>;

struct DownloadState {
    total_chunks: usize,
    received_chunks: usize,
    chunks_data: HashMap<usize, Vec<u8>>, // chunk_index -> data
}

type SharedDownloads = Arc<Mutex<HashMap<String, DownloadState>>>;

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

#[derive(serde::Serialize, serde::Deserialize)]
struct FileRequestMessage {
    sender: String,
    password: String,
    file_name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FileMetadataMessage {
    sender: String,
    password: String,
    file_name: String,
    total_chunks: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FileMessage {
    sender: String,
    password: String,
    file_name: String,
    data: Vec<u8>,
    receivers: Vec<String>,
}

pub async fn run_peer_to_peer_system(
    topic_name: String,
    password: String,
) -> Result<(), Box<dyn Error>> {
    let mut peer_scores: HashMap<PeerId, f64> = HashMap::new();
    let file_transfer_logs: SharedFileTransferLogs = Arc::new(Mutex::new(Vec::new()));
    let downloads: SharedDownloads = Arc::new(Mutex::new(HashMap::new()));

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
            line = stdin.next_line() => {
                match line {
                    Ok(Some(input)) => {
                        handle_user_input(&input, &mut swarm, &topic, &local_peer_id, &password, &mut peer_scores, Arc::clone(&file_transfer_logs), Arc::clone(&downloads)).await?;
                    },
                    Ok(None) => break Ok(()),
                    Err(e) => {
                        println!("Error reading stdin: {}", e);
                    }
                }
            }

            event = swarm.select_next_some() => {
                handle_swarm_event(event, &mut swarm, &password, &mut peer_scores, &local_peer_id, &topic, Arc::clone(&file_transfer_logs), Arc::clone(&downloads)).await?;
            }
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                remove_disconnected_peers_and_decay_scores(&swarm, &mut peer_scores);
            }
        }
    }
}

fn remove_disconnected_peers_and_decay_scores(
    swarm: &libp2p::Swarm<MyBehaviour>,
    peer_scores: &mut HashMap<PeerId, f64>,
) {
    let mut disconnected_peers = Vec::new();

    for peer_id in peer_scores.keys() {
        if !swarm.is_connected(peer_id) {
            disconnected_peers.push(peer_id.clone());
        }
    }

    for peer_id in &disconnected_peers {
        println!("Removing disconnected peer: {}", peer_id);
        peer_scores.remove(peer_id);
    }

    for (peer_id, score) in peer_scores.iter_mut() {
        *score -= 0.1;
        if *score < 0.0 {
            *score = 0.0;
        }
        println!("Updated score for {}: {}", peer_id, score);
    }
}

fn update_peer_score(peer_scores: &mut HashMap<PeerId, f64>, peer_id: &PeerId, delta: f64) {
    let score = peer_scores.entry(peer_id.clone()).or_insert(0.0);
    *score += delta;
    println!("Updated score for {}: {}", peer_id, *score);
}

fn select_peers(peer_scores: &HashMap<PeerId, f64>, n: usize) -> Vec<PeerId> {
    let mut peers_by_score: Vec<_> = peer_scores.iter().collect();
    peers_by_score.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    let mut selected_peers: Vec<PeerId> = peers_by_score
        .iter()
        .take(n.saturating_sub(1))
        .map(|(peer_id, _)| (*peer_id).clone())
        .collect();

    let remaining_peers: Vec<_> = peer_scores
        .keys()
        .filter(|peer_id| !selected_peers.contains(peer_id))
        .cloned()
        .collect();

    if let Some(random_peer) = remaining_peers.into_iter().choose(&mut rand::thread_rng()) {
        selected_peers.push(random_peer);
    }

    selected_peers.truncate(n);
    selected_peers
}

async fn handle_user_input(
    input: &str,
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    topic: &gossipsub::IdentTopic,
    local_peer_id: &PeerId,
    password: &str,
    peer_scores: &mut HashMap<PeerId, f64>,
    file_transfer_logs: SharedFileTransferLogs,
    downloads: SharedDownloads,
) -> Result<(), Box<dyn Error>> {
    let trimmed = input.trim();
    if trimmed == "@upload" {
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

            for (i, metadata) in chunks_metadata.iter().enumerate() {
                let mut buffer = vec![0; metadata.size];
                file.read_exact(&mut buffer)?;

                let chunk_filename = format!("{}({}-of-{})", file_name, i + 1, total_chunks);
                storage_manager::save_chunk(&buffer, &metadata, &local_peer_dir, &chunk_filename)?;
                let target_peers = select_peers(peer_scores, 3);

                if target_peers.is_empty() {
                    println!("No valid peers to send the file.");
                    return Ok(());
                }
                let receivers: Vec<String> = target_peers.iter().map(|pid| pid.to_string()).collect();

                let file_message = FileMessage {
                    sender: (*local_peer_id).to_string(),
                    password: password.to_string(),
                    file_name: chunk_filename.clone(),
                    data: buffer,
                    receivers, // Now we include the receivers field
                };

                let serialized = serde_json::to_vec(&file_message)?;

                if let Err(e) = send_file_to_peers(swarm, &target_peers, topic, serialized.clone()).await {
                    println!(
                        "Failed to send file chunk '{}' to peers {:?}: {:?}",
                        chunk_filename, target_peers, e
                    );
                } else {
                    println!(
                        "File chunk '{}' successfully sent to peers: {:?}",
                        chunk_filename, target_peers
                    );
                }

            }
        } else {
            println!("No file selected for upload.");
        }
    } else if trimmed == "@check_scores" {
        println!("Current peer scores:");
        for (peer_id, score) in peer_scores {
            println!("Peer ID: {}, Score: {}", peer_id, score);
        }
    } else if trimmed == "@check_logs" {
        let logs = file_transfer_logs.lock().unwrap();
        let results = search_transfer_log(&logs, None, None, None);
        for log in results {
            println!("Log: {:?}", log);
        }
    } else if trimmed.starts_with("@download ") {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            println!("Usage: @download <filename>");
        } else {
            let requested_file = parts[1].to_string();

            let request_msg = FileRequestMessage {
                sender: (*local_peer_id).to_string(),
                password: password.to_string(),
                file_name: requested_file.clone(),
            };

            let serialized = serde_json::to_vec(&request_msg)?;
            if let Err(e) = swarm
                .behaviour_mut()
                .gossipsub
                .publish(topic.clone(), serialized)
            {
                println!("Failed to publish file request: {:?}", e);
            } else {
                println!("File request for '{}' sent to network.", requested_file);
                let mut dls = downloads.lock().unwrap();
                dls.insert(
                    requested_file,
                    DownloadState {
                        total_chunks: 0,
                        received_chunks: 0,
                        chunks_data: HashMap::new(),
                    },
                );
            }
        }
    } else {
        let message_with_password = format!("{}:{}", password, trimmed);
        if let Err(e) = swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic.clone(), message_with_password.as_bytes())
        {
            println!("Publish error: {:?}", e);
        } else {
            println!("Message sent: {}", trimmed);
        }
    }
    Ok(())
}
async fn send_file_to_peers(
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    target_peers: &[PeerId],
    topic: &gossipsub::IdentTopic,
    serialized_message: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Add all target peers as explicit peers to ensure they are included in the Gossipsub mesh
    for peer_id in target_peers {
        if !swarm.is_connected(peer_id) {
            println!("Peer {} is not connected. Skipping.", peer_id);
            continue;
        }
        swarm.behaviour_mut().gossipsub.add_explicit_peer(peer_id);
        println!("Added peer {} as an explicit peer.", peer_id);
    }

    // Wait briefly to allow the protocol to adjust
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Publish the message to the topic once
    let result = swarm
        .behaviour_mut()
        .gossipsub
        .publish(topic.clone(), serialized_message);

    // Remove all explicit peers to restore normal routing
    for peer_id in target_peers {
        swarm.behaviour_mut().gossipsub.remove_explicit_peer(peer_id);
    }

    match result {
        Ok(_) => {
            println!("File successfully sent to peers: {:?}", target_peers);
            Ok(())
        }
        Err(e) => {
            println!("Failed to send file to peers {:?}: {:?}", target_peers, e);
            Err(e.into())
        }
    }
}
// async fn send_file_to_peer(
//     swarm: &mut libp2p::Swarm<MyBehaviour>,
//     target_peer_id: &PeerId,
//     topic: &gossipsub::IdentTopic,
//     serialized_message: Vec<u8>,
// ) -> Result<(), Box<dyn Error>> {
//     if !swarm.is_connected(target_peer_id) {
//         println!("Peer {} is not connected.", target_peer_id);
//         return Err("Target peer is not connected".into());
//     }

//     swarm
//         .behaviour_mut()
//         .gossipsub
//         .add_explicit_peer(target_peer_id);
//     println!("Added peer {} as an explicit peer.", target_peer_id);

//     tokio::time::sleep(Duration::from_secs(2)).await;

//     let result = swarm
//         .behaviour_mut()
//         .gossipsub
//         .publish(topic.clone(), serialized_message);

//     swarm
//         .behaviour_mut()
//         .gossipsub
//         .remove_explicit_peer(target_peer_id);

//     match result {
//         Ok(_) => {
//             println!("File successfully sent to peer {}.", target_peer_id);
//             Ok(())
//         }
//         Err(e) => {
//             println!("Failed to send file to peer {}: {:?}", target_peer_id, e);
//             Err(e.into())
//         }
//     }
// }

async fn handle_swarm_event(
    event: SwarmEvent<MyBehaviourEvent>,
    swarm: &mut libp2p::Swarm<MyBehaviour>,
    password: &str,
    peer_scores: &mut HashMap<PeerId, f64>,
    local_peer_id: &PeerId,
    topic: &gossipsub::IdentTopic,
    file_transfer_logs: SharedFileTransferLogs,
    downloads: SharedDownloads,
) -> Result<(), Box<dyn Error>> {
    match event {
        SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, _) in list {
                println!("mDNS discovered a new peer: {}", peer_id);
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                let initial_score: f64 = rand::thread_rng().gen_range(3.0..=8.0);
                peer_scores.entry(peer_id).or_insert(initial_score);
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
                downloads.clone(),
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
    downloads: SharedDownloads,
) -> Result<(), Box<dyn Error>> {
    let data = &message.data;

    if let Ok(file_message) = serde_json::from_slice::<FileMessage>(data) {
        let local_peer_id_str = local_peer_id.to_string();
        if !file_message.receivers.contains(&local_peer_id_str) {
            println!(
                "Received file chunk '{}' from peer {} but local peer is not a listed receiver. Ignoring.",
                file_message.file_name, peer_id
            );
            // Here you can choose to return or just ignore the message.
            return Ok(());
        }
        if file_message.password == password {
            let local_peer_id_str = format!("./{}/", local_peer_id);
            std::fs::create_dir_all(&local_peer_id_str)?;
            let file_path = format!("{}/{}", local_peer_id_str, file_message.file_name);
            std::fs::write(&file_path, &file_message.data)?;

            println!(
                "Received file chunk '{}' from peer {} and saved to '{}'",
                file_message.file_name, peer_id, file_path
            );

            if let Some((original_name, i, n)) = parse_chunk_filename(&file_message.file_name) {
                let mut dls = downloads.lock().unwrap();
                if let Some(download_state) = dls.get_mut(&original_name) {
                    // MODIFIED: Only increment received_chunks if this chunk wasn't already known
                    let is_new_chunk = !download_state.chunks_data.contains_key(&i);
                    download_state
                        .chunks_data
                        .entry(i)
                        .or_insert(file_message.data.clone());

                    if download_state.total_chunks == 0 {
                        download_state.total_chunks = n;
                    }

                    if is_new_chunk {
                        download_state.received_chunks += 1;
                        println!(
                            "Download progress for '{}': {}/{}",
                            original_name,
                            download_state.received_chunks,
                            download_state.total_chunks
                        );
                    }

                    // MODIFIED: Attempt reassembly only if all unique chunks are received
                    if download_state.total_chunks > 0
                        && download_state.received_chunks == download_state.total_chunks
                    {
                        // Verify all chunks are present
                        let all_present = (1..=download_state.total_chunks)
                            .all(|x| download_state.chunks_data.contains_key(&x));

                        if all_present {
                            let output_path = format!("{}/{}", local_peer_id_str, original_name);
                            let mut output_file = fs::File::create(&output_path)?;
                            for chunk_idx in 1..=download_state.total_chunks {
                                let chunk_data =
                                    download_state.chunks_data.get(&chunk_idx).unwrap();
                                output_file.write_all(chunk_data)?;
                            }
                            println!("Download complete: {}", output_path);

                            dls.remove(&original_name);
                        } else {
                            // MODIFIED: Do not remove download_state. Just note it's not complete.
                            println!(
                                "Not all chunks received yet for '{}', waiting for missing chunks.",
                                original_name
                            );
                            // We do nothing else, waiting for missing chunks to arrive
                        }
                    }
                }
            }

            update_peer_score(peer_scores, peer_id, 0.5);

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
            update_peer_score(peer_scores, peer_id, -0.01);
        }
    } else if let Ok(log_message) = serde_json::from_slice::<FileTransferLog>(data) {
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
    } else if let Ok(request_msg) = serde_json::from_slice::<FileRequestMessage>(data) {
        if request_msg.password == password {
            let local_peer_id_str = format!("./{}/", local_peer_id);
            if let Some((total_chunks, chunk_files)) =
                find_chunks_for_file(&local_peer_id_str, &request_msg.file_name)
            {
                let meta_msg = FileMetadataMessage {
                    sender: local_peer_id.to_string(),
                    password: password.to_string(),
                    file_name: request_msg.file_name.clone(),
                    total_chunks,
                };

                if let Ok(serialized_meta) = serde_json::to_vec(&meta_msg) {
                    if let Err(e) = swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(topic.clone(), serialized_meta)
                    {
                        println!("Failed to publish file metadata: {:?}", e);
                    } else {
                        println!("Sent file metadata for '{}'", request_msg.file_name);
                    }
                }

                for cf in chunk_files {
                    let chunk_data = fs::read(format!("{}{}", local_peer_id_str, cf))?;
                    let receivers = vec![peer_id.to_string()];
                    let file_message = FileMessage {
                        sender: local_peer_id.to_string(),
                        password: password.to_string(),
                        file_name: cf.clone(),
                        data: chunk_data,
                        receivers:receivers,
                    };
                    if let Ok(serialized_chunk) = serde_json::to_vec(&file_message) {
                        if let Err(e) = swarm
                            .behaviour_mut()
                            .gossipsub
                            .publish(topic.clone(), serialized_chunk)
                        {
                            println!("Failed to publish file chunk '{}': {:?}", cf, e);
                        } else {
                            println!("Sent file chunk '{}'", cf);
                        }
                    }
                }
            } else {
                println!(
                    "Requested file '{}' not found locally.",
                    request_msg.file_name
                );
            }
        } else {
            println!(
                "Received file request with invalid password from peer: {}",
                peer_id
            );
        }
    } else if let Ok(meta_msg) = serde_json::from_slice::<FileMetadataMessage>(data) {
        if meta_msg.password == password {
            let mut dls = downloads.lock().unwrap();
            if let Some(download_state) = dls.get_mut(&meta_msg.file_name) {
                download_state.total_chunks = meta_msg.total_chunks;
                println!(
                    "Receiving '{}' with {} chunks.",
                    meta_msg.file_name, meta_msg.total_chunks
                );
            }
        } else {
            println!(
                "Received file metadata with invalid password from peer: {}",
                peer_id
            );
        }
    } else {
        let message_content = String::from_utf8_lossy(&message.data);
        if message_content.starts_with(password) {
            let actual_message = &message_content[password.len() + 1..];
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

fn parse_chunk_filename(name: &str) -> Option<(String, usize, usize)> {
    let idx = name.rfind('(')?;
    let meta = &name[idx + 1..name.len() - 1];
    let parts: Vec<_> = meta.split("-of-").collect();
    if parts.len() != 2 {
        return None;
    }
    let i = parts[0].parse::<usize>().ok()?;
    let n = parts[1].parse::<usize>().ok()?;

    let original_name = (&name[..idx]).to_string();
    Some((original_name, i, n))
}

fn find_chunks_for_file(dir: &str, file_name: &str) -> Option<(usize, Vec<String>)> {
    let entries = fs::read_dir(dir).ok()?;
    let mut chunk_files = Vec::new();
    let mut total_chunks = 0;

    for entry in entries {
        if let Ok(e) = entry {
            let fname = e.file_name().to_string_lossy().to_string();
            if fname.starts_with(file_name) {
                if let Some((_, _, n)) = parse_chunk_filename(&fname) {
                    if n > total_chunks {
                        total_chunks = n;
                    }
                    chunk_files.push(fname);
                }
            }
        }
    }

    if chunk_files.is_empty() {
        None
    } else {
        chunk_files.sort_by_key(|f| {
            if let Some((_, i, _)) = parse_chunk_filename(f) {
                i
            } else {
                usize::MAX
            }
        });
        Some((total_chunks, chunk_files))
    }
}
