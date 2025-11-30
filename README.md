# RabbitMQ Workspace

This workspace contains a professional suite of tools for designing, deploying, monitoring, and testing a RabbitMQ-based messaging architecture.

## Core Philosophy: Emanate from the Tree

This project follows a strict design principle: the entire system topology should **emanate from a single source of truth**. That source of truth is the `artifacts/message_types.json` file.

This design document is not just a diagram; it is the machine-readable spec that drives the entire system. All tools in this workspace are designed to be consumers of this spec, which ensures that our live, running system never drifts from our intended design.

## Crate Breakdown

This workspace is organized into several distinct crates, each with a specific purpose:

-   `rabbitmq-config`: A core library crate that provides shared logic for configuration management and a high-level RabbitMQ client for connecting to the server.
-   `rabbitmq-mon`: A terminal-based UI (TUI) application for monitoring the health and status of the RabbitMQ server, including queue depths and consumer counts.
-   `topology-creator` (binary within `rabbitmq-config`): A command-line tool that reads `artifacts/message_types.json` and programmatically declares all exchanges and queues on the server.
-   `messaging_tests`: A full integration testing framework, complete with `test-producer` and `test-consumer` utilities and a suite of automated test scripts.
-   *(Other crates like `egui-components`, `messaging_cli`, etc. serve as components for other experiments within the workspace.)*

## Key Workflows

All commands should be run from the workspace root.

### Apply the Topology

To apply the design from `artifacts/message_types.json` to the RabbitMQ server, run the `topology-creator`:

```sh
cargo run -p rabbitmq-config --bin topology-creator
```

### Monitor the Server

To launch the terminal-based monitoring application:

```sh
./run_monitor.sh
```

### Run the Test Suite

To run the entire suite of automated integration tests, which validates the end-to-end message flow for our defined topology:

```sh
./messaging_tests/run_all_tests.sh
```
