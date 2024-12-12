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