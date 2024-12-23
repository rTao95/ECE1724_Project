# ECE1724 Project Proposal: Distributed File Storage System

## Team Members

Yige Tao 1000741094 yige.tao@mail.utoronto.ca

Zhonghao Liu 1004796848 zhh.liu@mail.utoronto.ca

Zhouhan Jin 1006146699 zhouhan.jin@mail.utoronto.ca

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

## Demo Video

https://drive.google.com/file/d/1-rJA1W_pPp8SuCvOCGJEp53cy9ptvYI1/view?usp=drive_link
  
## User (and Developer) Guide

### Running a Peer

1. **Starting a Peer**  
   - Ensure that you have built the project following the steps in the Reproducibility Guide.
   - Run the peer with:
     ```bash
     cargo run --release -- --topic <topic_name> --password <password>
     ```
     Example:
     ```bash
     cargo run --release -- --topic mytopic --password secretpassword
     ```
   - This command launches a peer that subscribes to the specified topic (`mytopic`) and uses `secretpassword` for authentication.

2. **Basic Interactions (Commands)**  
   - `@upload`:  
     Prompts you to select a file for upload. Once selected, the file is split into chunks and distributed to multiple peers for redundancy.
   
   - `@download <filename>`:  
     Requests the specified file from the network. If found, you receive metadata and file chunks, which the peer automatically reassembles.
   
   - `@check_scores`:  
     Prints the current known peer scores, helping you understand the trustworthiness of your peers.
   
   - `@check_logs`:  
     Prints the file transfer logs, allowing you to review past transfers.
   
   - Any other text input (e.g., `Hello network!`):  
     Sends a public message to **all peers** in the topic, facilitating open communication and announcements.

3. **Behavior and Storage**  
   - Files and their chunks are stored in a directory named after the local peer’s ID.
   - Scores are dynamically updated; well-behaved peers become more trusted.
   - All file transfers are logged, providing an audit trail of network activities.

4. **For Developers**  
   - To modify chunk sizes, peer scoring logic, or other behaviors, review and update the relevant modules (e.g., `chunker`, `storage_manager`, or event-handling code).
   - The code is structured to allow extension of functionalities, integration with additional encryption, or alternative discovery mechanisms.

---

## Reproducibility Guide

### Prerequisites
- **Rust & Cargo Installed**:  
  On Linux or macOS:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
  ```
---

## Steps to Build and Run

### 1. Clone the Repository
Clone the project repository using the following commands:
```bash
git clone <URL_OF_YOUR_PROJECT_REPOSITORY>
cd <PROJECT_DIRECTORY>
```

### 2. Build the Project
To build the project, run:
```bash
cargo build --release
```
This command generates an optimized binary in the `target/release/` directory.

### 3. Run a Peer
Start a peer by running:
```bash
cargo run --release -- --topic <topic_name> --password <password>
```
Example:
```bash
cargo run --release -- --topic mytopic --password secretpassword
```

- Replace `<topic_name>` with the desired gossipsub topic name.
- Replace `<password>` with the shared password for secure communication.

When the command runs:
- The peer prints its local Peer ID.
- It starts listening for connections over TCP and QUIC.

### 4. Connect Multiple Peers
To create a network:
- Repeat the above command in multiple terminals or machines.
- Ensure all peers use the same topic name and password.
- Peers on the same local network automatically discover each other via mDNS.

### 5. Testing the Features
After setting up the peers, test the following features:

#### File Upload
1. Use the `@upload` command in a peer's terminal.
2. Select a file to upload.
3. The file is chunked and sent to multiple peers for redundancy.

#### File Download
1. Use the `@download <filename>` command.
2. If the file exists in the network, it will be downloaded chunk-by-chunk and reassembled.

#### Public Messaging
1. Send a public message by typing any text (e.g., `Hello network!`).
2. The message will be broadcast to all peers subscribed to the topic.

#### Check Peer Scores
1. Use the `@check_scores` command to display the current peer scores.

#### View File Logs
1. Use the `@check_logs` command to view the history of file transfers.

---

No additional setup or configurations are required. Following these steps ensures that any user or instructor can build, run, and test the project seamlessly.

## Contributions
Our team worked diligently and collaboratively, with each member contributing significantly to the success of the project. Below is a breakdown of the specific contributions:
1. Node Discovery and P2P Networking: Yige Tao, Zhonghao Liu
2. File Chunking and Distribution: Zhonghao Liu, Zhouhan Jin
3. Data Redundancy and Reliability: Yige Tao, Zhonghao Liu
4. User Authentication and Access Control: Yige Tao, Zhouhan Jin
5. Master Node for Initial Coordination: Yige Tao, Zhouhan Jin
6. Front-End Interface (CLI): Zhonghao Liu, Zhouhan Jin
7. Test and write final report: Yige Tao, Zhonghao Liu, Zhouhan Jin

## Lessons Learned and Concluding Remarks
Throughout the development of this project, our team gained a deeper understanding of building a peer-to-peer (P2P) file distribution system. Initially, we experimented with direct broadcasting of files using a pub/sub mechanism and discovered the challenges of ensuring reliable, timely, and complete file transfers in a decentralized network. We learned that effectively handling chunked data distribution requires careful coordination: proper file chunk naming, robust metadata exchange for total chunks and file structure, and a strict approach to verifying the completeness of received data before reassembling files.

Our team also recognized the importance of error handling, especially in asynchronous and distributed environments. Relying solely on broadcast messaging often meant dealing with duplicate chunks, missing chunks, or metadata arriving out of order. These scenarios taught us the value of maintaining a clear download state and only committing to final file assembly when all required data is confirmed. Moreover, we found it essential to handle partial or incomplete downloads gracefully and to avoid leaving users with empty or corrupt files.

From a development standpoint, working with libp2p, Diesel or SQLx, and integrating various components into a unified system reinforced the value of modular code design, consistent naming conventions, and thorough testing. The complexity of asynchronous code and event-driven architectures taught us the importance of clear documentation, logging, and careful concurrency management to avoid race conditions and ensure predictable behavior.  

In conclusion, the project successfully achieved its objectives by creating a decentralized, secure, and efficient P2P file sharing system in Rust. The system fills a niche in the Rust ecosystem by providing a foundation that can be extended for more complex P2P applications. By using libp2p and gossipsub, the system enables users to upload, request, and distribute files chunk-by-chunk over a robust network, with features like node discovery, data redundancy, and user authentication. The lessons learned throughout the development process will inform our future work in the Rust ecosystem and beyond.
