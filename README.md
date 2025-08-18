# 🦀 RabbitMQ Workspace

A collection of tools for managing RabbitMQ and experimenting with database security, all built as part of my journey learning Rust.

---

## What is this?

This is a **Cargo workspace** containing a set of related crates that I'm building to learn and apply different concepts in Rust. The primary focus is on creating practical tools for managing a **RabbitMQ** server, with a secondary project for exploring hardware authentication using **YubiKey**.

While this is a learning project, the goal is to build useful and robust tools.

---

## Crates in this Workspace

### 🐰 RabbitMQ Tools
| Crate | Purpose | Progress |
| :--- | :--- | :--- |
| **`rabbitmq-config`** | A library for managing RabbitMQ server topology. | ✅ Usable |
| **`rabbitmq-info`** | A tool for gathering metrics and server status. | ✅ Usable |
| **`rabbitmq-ui`** | A graphical UI for managing RabbitMQ. | 🚧 In Progress |

### 🔐 Security & Utilities
| Crate | Purpose | Progress |
| :--- | :--- | :--- |
| **`pg_vault`** | An app to manage PostgreSQL connections using a YubiKey. | ✅ Usable |
| **`messaging_commands`**| A shared library for messaging protocol logic. | ✅ Usable |
| **`messaging_cli`** | A command-line tool for messaging. | 🚧 In Progress |
| **`yak_json`** | A simple utility for JSON processing. | ✅ Usable |
| **`egui-components`** | Shared UI components for the `egui` framework. | 🚧 In Progress |

---

## 🚀 Getting Started

### Prerequisites
You'll need the Rust toolchain (1.70+).
```shell
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh