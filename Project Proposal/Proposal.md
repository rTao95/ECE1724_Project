## Motivation

In today’s data-driven world, managing and securing large volumes of data efficiently is a significant challenge. Distributed file storage systems offer a way to address these challenges by storing data across multiple nodes, ensuring reliability and accessibility even if individual nodes go offline. However, many existing solutions lack the flexibility and performance needed for seamless integration into varied environments, especially when built from scratch in a language like Rust.

This project seeks to develop a distributed file storage system that combines the resilience of peer-to-peer networking with the stability of a coordinating master node. By using a hybrid approach, the system enables nodes to discover and communicate with each other while efficiently managing data flow. We also incorporate file chunking to handle large files, splitting them into smaller pieces to distribute across nodes, enhancing access speed and storage balance.

Our goal is to create a Rust-based system that goes beyond storage, integrating data redundancy for reliability, user authentication for secure access, and a simple command-line interface for ease of use. These features align with Rust’s strengths in memory safety and concurrency, allowing us to build a tool that’s reliable, secure, and user-friendly.

This project not only fills a current gap in the Rust ecosystem by offering a foundational distributed storage system but also presents a rewarding opportunity for our team to deepen our knowledge of Rust and distributed systems. Our team are excited to bring a fresh, practical tool to the ecosystem, designed to help developers explore and implement distributed storage with ease.

## Objective

The objective of this project is to develop a scalable, distributed file storage system in Rust that combines peer-to-peer networking with distributed data chunking and redundancy. This system will fill a gap in the Rust ecosystem by providing a native, fault-tolerant solution for storing and managing large files across multiple nodes with minimal central control. Utilizing Rust's high performance and memory safety features, the project aims to offer a robust and reliable platform suited for use cases requiring resilient, decentralized storage.

## Key Features

### 1. Node Discovery and P2P Networking
- **Purpose**: Enable decentralized communication between nodes for scalability and resilience.
- **Implementation**: 
  - Utilize **Libp2p**, a flexible library facilitating peer-to-peer network communication, to allow nodes to discover and connect without centralized control.
- **Workflow**: 
  - New nodes initially register with a master node to get details of existing peers but will thereafter communicate directly with these peers.

### 2. File Chunking and Distribution
- **Purpose**: Improve data transfer efficiency and allow for distributed, redundant storage across nodes.
- **Implementation**: 
  - Break down files into smaller chunks for distribution across the network.
  - Use algorithms that manage chunk integrity and order, such as hashing for verification.
- **Advantage**: 
  - Parallel uploads/downloads.
  - Reduced strain on individual nodes.

### 3. Data Redundancy and Reliability
- **Purpose**: Ensure data integrity and availability if a node fails.
- **Implementation**: 
  - Employ redundancy strategies, such as replication or erasure coding. 
  - Implement a unique naming scheme to track the status and security of file chunks.
- **Master Node Role**: 
  - Monitor node health and redistribute data from unavailable nodes to maintain reliability.

### 4. User Authentication and Access Control
- **Purpose**: Secure the system and limit access to authorized users.
- **Implementation**: 
  - Basic authentication protocols.
  - Role-based or user-specific access control methods to manage permissions for file operations.

### 5. Master Node for Initial Coordination
- **Purpose**: Simplify node registration and manage network metadata.
- **Functionality**:
  - **Node Registration**: Provides peer node addresses upon new node registration.
  - **Monitoring and Metadata Management**: Maintains the network structure and node status to coordinate recovery procedures.
  - **Failover Mechanism**: Offers alternative peer connections when existing ones fail.

### 6. Front-End Interface (CLI)
- **Purpose**: Provide a user-friendly interface for interaction and administration.
- **Implementation**: 
  - Develop a command-line interface allowing operations like uploading, downloading, listing files, and monitoring node health.
- **Functionality**: 
  - Command support for all core features to ensure system accessibility even without a graphical interface.
 
## Tentative Plan for Distributed File Storage System Project

### Week 1: Setup and Initial Implementation
- **Project Setup**
  - Initiate a Rust project repository with Git for version control.
  - Research and select libraries: Libp2p for networking, Diesel/SQLx for database.
  - Set up a local development environment to simulate nodes.

- **Node Discovery and P2P Networking**
  - Begin implementing Libp2p for peer-to-peer networking.
  - Implement node registration and discovery mechanisms.

- **File Chunking Setup**
  - Develop a module to break files into small chunks for distribution.

*Responsibility*: All team members collaborate initially to familiarize everyone with the codebase and Rust environment.

### Week 2: Core Functionality and Data Redundancy
- **File Distribution**
  - Implement distribution of file chunks across nodes.
  - Develop parallel upload/download methods for efficiency.

- **Data Redundancy and Monitoring**
  - Implement redundancy strategies like replication or erasure coding.
  - Establish monitoring processes for node health via the master node.

*Responsibility*:  
- **Developer A**: Focus on file distribution.
- **Developer B**: Implement redundancy mechanisms.
- **Developer C**: Develop monitoring features for node health.

### Week 3: Security and Master Node Enhancements
- **User Authentication and Access Control**
  - Build authentication methods for file operation security.
  - Implement role-based or user-specific access controls.

- **Master Node Coordination**
  - Enhance master node functions for node registration and metadata management.
  - Develop failover mechanisms to maintain reliable connections.

*Responsibility*:  
- **Developer A**: Security features.
- **Developer B**: Enhance master node functionalities.
- **Developer C**: Integrate failover mechanisms and test security.

### Week 4: Front-End Interface (CLI) and Testing
- **Command-Line Interface (CLI)**
  - Develop a user-friendly CLI for file operations.
  - Integrate commands for uploading, downloading, and node health monitoring.

- **Testing and Optimization**
  - Conduct comprehensive testing and optimize for performance.
  - Address bugs and refine features.

*Responsibility*:  
- **Developer A**: Develop CLI.
- **Developer B**: Testing.
- **Developer C**: Optimize based on test results.

### Week 5: Finalization and Documentation
- **Documentation and Deployment**
  - Document setup, usage, and maintenance processes.
  - Prepare a final project presentation and demo.
  - Explore potential deployment on AWS for scalability.

*Responsibility*: All team members collaborate on documentation, finalization, and preparation for deployment.

Yige Tao 1000741094
Zhouhan Jin 1006146699
