Objective
The objective of this project is to develop a scalable, distributed file storage system in Rust that combines peer-to-peer networking with distributed data chunking and redundancy. This system will fill a gap in the Rust ecosystem by providing a native, fault-tolerant solution for storing and managing large files across multiple nodes with minimal central control. Utilizing Rust's high performance and memory safety features, the project aims to offer a robust and reliable platform suited for use cases requiring resilient, decentralized storage.

Key Features
1. Node Discovery and P2P Networking
Purpose: Enable decentralized communication between nodes for scalability and resilience.
Implementation:
Utilize Libp2p, a flexible library facilitating peer-to-peer network communication, to allow nodes to discover and connect without centralized control.
Workflow:
New nodes initially register with a master node to get details of existing peers but will thereafter communicate directly with these peers.
2. File Chunking and Distribution
Purpose: Improve data transfer efficiency and allow for distributed, redundant storage across nodes.
Implementation:
Break down files into smaller chunks for distribution across the network.
Use algorithms that manage chunk integrity and order, such as hashing for verification.
Advantage:
Parallel uploads/downloads.
Reduced strain on individual nodes.
3. Data Redundancy and Reliability
Purpose: Ensure data integrity and availability if a node fails.
Implementation:
Employ redundancy strategies, such as replication or erasure coding.
Implement a unique naming scheme to track the status and security of file chunks.
Master Node Role:
Monitor node health and redistribute data from unavailable nodes to maintain reliability.
4. User Authentication and Access Control
Purpose: Secure the system and limit access to authorized users.
Implementation:
Basic authentication protocols.
Role-based or user-specific access control methods to manage permissions for file operations.
5. Master Node for Initial Coordination
Purpose: Simplify node registration and manage network metadata.
Functionality:
Node Registration: Provides peer node addresses upon new node registration.
Monitoring and Metadata Management: Maintains the network structure and node status to coordinate recovery procedures.
Failover Mechanism: Offers alternative peer connections when existing ones fail.
6. Front-End Interface (CLI)
Purpose: Provide a user-friendly interface for interaction and administration.
Implementation:
Develop a command-line interface allowing operations like uploading, downloading, listing files, and monitoring node health.
Functionality:
Command support for all core features to ensure system accessibility even without a graphical interface.
