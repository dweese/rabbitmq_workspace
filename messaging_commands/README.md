# messaging_commands

A versatile Rust library providing a unified interface for interacting with various messaging protocols including AMQP (RabbitMQ), MQTT, and STOMP.

> **Learning Notes**: This library extends the RabbitMQ-specific functionality into a more generalized messaging framework. It demonstrates how to apply Rust's trait system to create protocol-agnostic interfaces while maintaining the performance benefits of specific implementations.

## Project Structure

The library is organized into several modules:

- **client/** - Base client interfaces and shared functionality
- **clients/** - Protocol-specific implementations
    - **amqp/** - Implementation for AMQP protocol (RabbitMQ)
    - **mqtt/** - Implementation for MQTT protocol
    - **stomp/** - Implementation for STOMP protocol
- **common/** - Shared data structures and utilities
- **config/** - Configuration structures for various protocols
- **error.rs** - Unified error handling across all protocols
- **protocol/** - Protocol-specific message formats and conversions
- **traits/** - Core trait definitions for protocol abstraction
- **utils/** - Helper utilities and common functionality
- **version/** - Version management and compatibility

## Features

- **Protocol Abstraction** - Common interface across different messaging protocols
- **Unified Error Handling** - Consistent error types regardless of underlying protocol
- **Async First** - Built on tokio for non-blocking operations
- **Configuration Management** - Strongly-typed configuration for all supported protocols
- **Message Transformation** - Tools for converting between protocol-specific message formats
- **Command Pattern** - Implementation of messaging operations as commands for better testability

## Key Concepts

### Protocol Abstraction

The library uses Rust's trait system to create a unified interface across different messaging protocols:

- Common traits define the basic operations (connect, publish, subscribe, etc.)
- Protocol-specific implementations handle the details
- Users can write code against the traits without worrying about protocol specifics

### Messaging Client Architecture

Clients are structured in a layered approach:

1. **Transport Layer** - Handles the low-level protocol communication
2. **Session Layer** - Manages connection state and reconnection logic
3. **Command Layer** - Implements specific messaging operations
4. **Utility Layer** - Provides helper functions and data transformations

### Error Handling Strategy

A unified error handling approach that:

- Provides protocol-specific context when needed
- Abstracts away protocol-specific errors when appropriate
- Uses thiserror for clear, descriptive error messages
- Enables consistent handling of errors regardless of protocol

### Command Pattern Implementation

Operations are implemented as commands, which:

- Encapsulate all data needed for an operation
- Can be executed, canceled, or retried
- Provide audit logs of operations
- Enable testing without actual protocol connections

## Protocol Support

### AMQP (RabbitMQ)

Full support for RabbitMQ operations including:
- Connection management
- Queue and exchange operations
- Message publishing and consuming
- Advanced features like publisher confirms

### MQTT

Support for MQTT protocol features including:
- Connection with QoS levels
- Topic subscription and publishing
- Retained messages
- Last Will and Testament

### STOMP

Support for STOMP messaging:
- Connection and session management
- Frame-based communication
- Subscription handling
- Transaction support

## Integration with Existing Components

This library builds upon the foundation established in the rabbitmq-config project, generalizing many concepts to work across multiple protocols while maintaining the specific optimizations for each.

## Usage Examples

Examples of how to use the unified interface will be added as implementation progresses.

## Dependencies

- **tokio** - Async runtime
- **serde** - Serialization/deserialization
- **thiserror** - Error handling
- **various protocol libraries** - For specific protocol implementations