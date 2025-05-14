# RabbitMQ Workspace

A Rust workspace for working with RabbitMQ messaging.

## Overview

This project provides a collection of Rust applications for interacting with RabbitMQ, demonstrating how to implement messaging patterns using the AMQP protocol.

## Project Structure

The workspace consists of multiple crates, each focusing on specific RabbitMQ functionality:

- Publishers and consumers
- Message patterns (request-reply, pub-sub, etc.)
- Error handling and reconnection strategies
- UI tools for monitoring and visualization

## Dependencies

This project uses the following key dependencies:

- **lapin** (2.5.3) - Rust AMQP client library
- **tokio** (1.45.0) - Asynchronous runtime
- **serde** (1.0.219) and **serde_json** (1.0.140) - Serialization/deserialization
- **eframe** (0.24.1) - GUI framework for monitoring tools
- **futures-util** (0.3.31) - Utilities for working with asynchronous code
- **thiserror** (1.0.69) - Error handling
- **env_logger** (0.11.8) - Logging

## Getting Started

### Prerequisites

- Rust 1.86.0 or later
- RabbitMQ server (local installation or Docker)

### Setup

1. Clone the repository:
   ```
   git clone https://github.com/dweese/rabbitmq_workspace.git
   cd rabbitmq_workspace
   ```

2. Build the workspace:
   ```
   cargo build
   ```

3. Run a specific example:
   ```
   cargo run -p <crate-name>
   ```

## Configuration

RabbitMQ connection settings can be configured through environment variables or configuration files.

## License

[Specify your license here]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.