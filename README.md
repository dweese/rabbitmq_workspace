# RabbitMQ Workspace

A collection of Rust libraries and applications for working with RabbitMQ message brokers.

> **Learning Notes**: This workspace was developed as a self-teaching project to explore Rust programming while leveraging prior experience with message brokers in C and Java.

## Project Overview

This workspace demonstrates a modular approach to building a complex application in Rust, with clear separation of concerns between components:

1. **rabbitmq-config** - Core library providing RabbitMQ connectivity
2. **egui-components** - Reusable UI components for visualization
3. **rabbitmq-topology-visualizer** - Application tying everything together

## Architecture

### Core Libraries

#### rabbitmq-config

The foundation library that handles all RabbitMQ communication:
- Async client for RabbitMQ operations
- Entity management (queues, exchanges, bindings)
- Topology information retrieval
- Error handling and serialization

#### egui-components

A toolkit for building user interfaces with egui:
- Tree visualization components
- Layout managers
- Event-driven interaction patterns
- RabbitMQ topology visualization

### Applications

#### rabbitmq-topology-visualizer

The end-user application that combines the libraries:
- Interactive UI for RabbitMQ management
- Connection handling
- Entity creation and editing
- Topology visualization

## Key Design Patterns

### Trait-based API Design

The workspace uses traits extensively to create clean abstraction boundaries:
- `TopologyDataSource` trait decouples visualization from data sources
- Generic tree components work with any ID type
- Error handling uses trait objects

### Modular Architecture

Components are designed to be reusable and independent:
- Libraries have clear responsibilities
- Dependencies flow in one direction
- Public APIs are well-defined

### Error Handling Strategy

A consistent approach to error handling:
- Custom error types with context
- Error propagation through Result
- Conversion between error types
- User-friendly error presentation

### Async Programming

Leveraging Rust's async capabilities:
- Non-blocking RabbitMQ operations
- Timeout handling
- Background processing
- UI responsiveness

## Development Workflow

### Building the Workspace

The workspace uses Cargo workspaces to manage the projects:
- Shared dependencies are resolved efficiently
- Projects can be built individually or together
- Testing can be run across all components

### Testing Strategy

The codebase includes several types of tests:
- Unit tests for individual components
- Integration tests for library APIs
- End-to-end tests for the application

## Learning Outcomes

This project served as a learning vehicle for several Rust concepts:
- Ownership and borrowing in complex applications
- Async programming with tokio
- UI development with egui
- Error handling with thiserror
- Trait-based abstraction
- Workspace management