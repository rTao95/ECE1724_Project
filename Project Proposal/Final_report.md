# ECE1724 Project Proposal: Distributed File Storage System

## Team Members

Yige Tao 1000741094

Zhonghao Liu 1004796848

Zhouhan Jin 1006146699

## Motivation

In today’s data-driven world, managing and securing large volumes of data efficiently is a significant challenge. Distributed file storage systems offer a way to address these challenges by storing data across multiple nodes, ensuring reliability and accessibility even if individual nodes go offline. However, many existing solutions lack the flexibility and performance needed for seamless integration into varied environments, especially when built from scratch in a language like Rust.

This project seeks to develop a distributed file storage system that combines the resilience of peer-to-peer networking with the stability of a coordinating master node. By using a hybrid approach, the system enables nodes to discover and communicate with each other while efficiently managing data flow. We also incorporate file chunking to handle large files, splitting them into smaller pieces to distribute across nodes, enhancing access speed and storage balance.

Our goal is to create a Rust-based system that goes beyond storage, integrating data redundancy for reliability, user authentication for secure access, and a simple command-line interface for ease of use. These features align with Rust’s strengths in memory safety and concurrency, allowing us to build a tool that’s reliable, secure, and user-friendly.

This project not only fills a current gap in the Rust ecosystem by offering a foundational distributed storage system but also presents a rewarding opportunity for our team to deepen our knowledge of Rust and distributed systems. Our team are excited to bring a fresh, practical tool to the ecosystem, designed to help developers explore and implement distributed storage with ease.

## Objective

The objective of this project is to develop a scalable, distributed file storage system in Rust that combines peer-to-peer networking with distributed data chunking and redundancy. This system will fill a gap in the Rust ecosystem by providing a native, fault-tolerant solution for storing and managing large files across multiple nodes with minimal central control. Utilizing Rust's high performance and memory safety features, the project aims to offer a robust and reliable platform suited for use cases requiring resilient, decentralized storage.

## Key Features

This project provides a peer-to-peer (P2P) file-sharing system using libp2p with the following main features, all of which contribute to achieving the project’s objectives of secure, discoverable, and reliable data sharing:

1. **Node Discovery via mDNS**  
   - Automatically discovers peers on the same local network segment.
   - Eliminates the need for manual peer configuration.
   - Enhances scalability and ease-of-use as new peers join the network.

2. **Public Channel Messaging (Broadcast Communication)**  
   - Allows any peer to broadcast public messages to the entire network.
   - Facilitates announcements, coordination, and open communication among connected peers.

3. **Chunked File Transfers**  
   - Splits large files into fixed-size chunks (e.g., 20 KB).
   - Makes file transfers more efficient and resilient by handling smaller data units.
   - Peers requesting a file can independently retrieve and assemble the chunks.

4. **Data Redundancy (Multiple Recipients)**  
   - Sends each file’s chunks to multiple peers (e.g., three) for redundancy.
   - Increases data availability and reliability, ensuring the file remains accessible even if some peers disconnect or go offline.

5. **Peer Scoring Mechanism**  
   - Assigns an initial random score (e.g., between 3 and 8) to newly discovered peers.
   - Continuously updates scores based on peer behavior (e.g., sharing valid files increases score, invalid interactions decrease it).
   - Helps maintain a trusted and healthy network by decaying scores over time and removing disconnected peers.

6. **File Transfer Logging**  
   - Maintains a log of all file transfers.
   - Allows users to review past transactions, enhancing traceability and auditability.

7. **Password-Protected Messages and Files**  
   - All communications (messages, metadata, and file chunks) include a shared password.
   - Ensures only authorized peers can read or process the transmitted data.

8. **Topic-Based Communication (Gossipsub)**  
   - Uses a gossipsub topic for all interactions, including file requests, metadata, and broadcasts.
   - Leverages the gossip mesh to ensure decentralized, robust message dissemination and improved network fault tolerance.
