# rabbitmq-config

A Rust library for interacting with RabbitMQ message broker, providing a clean, async API for common RabbitMQ operations.

> **Learning Notes**: This library was developed as a self-teaching project to explore Rust while leveraging prior experience with messaging systems in C and Java. It demonstrates idiomatic Rust approaches to async programming, error handling, and API design.

## Project Structure

The library is organized into several modules:

- **client.rs** - The main `RabbitMQClient` implementation for connecting to RabbitMQ and performing operations
- **config.rs** - Configuration structs including `RabbitMQConfig` and `ConsumerConfig`
- **error.rs** - Custom error types via `RabbitMQError` enum using thiserror
- **models.rs** - Data models like `QueueInfo`, `ExchangeInfo`, and `RabbitMQMessage`
- **topology.rs** - Types and functions for working with RabbitMQ topology

## Features

- **Async API** - Built on tokio for non-blocking operations
- **Connection Management** - Automatic connection establishment with timeout handling
- **Queue & Exchange Operations** - Create, bind, and delete queues and exchanges
- **Message Publishing** - Send messages with custom properties
- **Error Handling** - Detailed error types for all operations
- **Serialization** - serde support for configuration and message structures

## Key Concepts

### Connection Management

The library handles RabbitMQ connections asynchronously with proper error handling and timeouts. It manages both the connection and channel lifecycle.

### Entity Management

RabbitMQ entities (queues, exchanges, bindings) can be created, modified, and deleted through a clean API that abstracts the underlying AMQP protocol details.

### Error Handling

Using thiserror, the library provides a rich