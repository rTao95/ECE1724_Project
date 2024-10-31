# Motivation

As data storage needs continue to grow in scale and complexity, centralized storage systems increasingly struggle with challenges related to reliability, scalability, and data recovery. Distributed File Storage Systems (DFSS) offer a robust alternative by distributing data across multiple nodes, allowing for resilience against individual node failures and enabling scalability across various environments, from small clusters to large-scale networks. 

However, the Rust ecosystem currently lacks a beginner-friendly, comprehensive solution for distributed file storage that combines core functionalities such as peer-to-peer networking, file chunking and distribution, data redundancy, and user authentication. Building this project in Rust offers a unique opportunity to bridge this gap, leveraging Rust's memory safety and concurrency strengths to create a highly reliable, performant system.

Our design includes peer-to-peer networking for node discovery using Rust’s `libp2p` library, allowing nodes to locate and communicate directly without needing a central server. File chunking and distribution will allow files to be split and stored across nodes, ensuring efficient, parallel data retrieval. Coupled with data redundancy, this design maintains access even during node failures. Besides, 
By implementing user authentication and access control using Rust libraries like `Diesel` or `SQLx` for secure database interactions, and a simple command-line interface for ease of use, we aim to provide a scalable, secure, and accessible tool. 

Altogether, this project will be both a valuable addition to the Rust ecosystem and an engaging learning experience for our team.
