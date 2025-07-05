## AI Assistant
# 🏗️ **RabbitMQ Workspace - Enterprise Messaging & Security Infrastructure**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/username/rabbitmq_workspace)

> **A comprehensive Rust workspace for enterprise messaging infrastructure with hardware-secured database access**

## 🎯 **Overview**

The **RabbitMQ Workspace** is a modular ecosystem that combines **RabbitMQ messaging management** with **hardware-secured database access**. Built in Rust for performance and safety, it provides enterprise-grade tooling for messaging infrastructure, database security, and operational management.

## 🌟 **Key Features**

- 🔐 **Hardware Security** - Yubikey authentication for database connections
- 🐰 **RabbitMQ Management** - Comprehensive monitoring and configuration tools
- 🎨 **Cross-Platform UI** - Native GUI applications with egui
- 📊 **Audit & Compliance** - Session tracking and security logging
- 🛠️ **Developer Tools** - CLI utilities and reusable libraries
- 🏗️ **Modular Architecture** - Clean separation of concerns

## 📦 **Workspace Components**

### **🐰 Messaging Infrastructure**
| Component | Purpose | Status |
|-----------|---------|--------|
| **`messaging_commands`** | Core messaging library and protocol definitions | ✅ Core |
| **`messaging_cli`** | Command-line interface for messaging operations | 🚧 Development |
| **`rabbitmq-config`** | Configuration management and validation | ✅ Core |
| **`rabbitmq-info`** | Monitoring, metrics, and information gathering | ✅ Core |
| **`rabbitmq-ui`** | GUI application for RabbitMQ management | 🚧 Development |

### **🔐 Security & Database Layer**
| Component | Purpose | Status |
|-----------|---------|--------|
| **`pg_vault`** | PostgreSQL with Yubikey hardware authentication | 🔥 **Featured** |
| **`yak_json`** | JSON processing and validation utilities | ✅ Core |

### **🎨 User Interface Layer**
| Component | Purpose | Status |
|-----------|---------|--------|
| **`egui-components`** | Reusable GUI components for cross-platform apps | 🚧 Development |

### **📋 Supporting Infrastructure**
- **`artifacts/`** - Sample databases, configurations, and schemas
- **`tests/`** - Integration and end-to-end tests
- **Scripts** - Build automation and deployment tools

## 🚀 **Quick Start**

### **Prerequisites**
```shell script
# Install Rust (1.70+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# For hardware authentication (optional)
sudo apt install pcscd libpcsclite-dev  # Ubuntu/Debian
sudo dnf install pcsc-lite-devel        # Fedora/RHEL
```


### **Build the Workspace**
```shell script
git clone https://github.com/username/rabbitmq_workspace.git
cd rabbitmq_workspace

# Build all components
cargo build --workspace

# Build with hardware features
cargo build --workspace --features hardware-yubikey

# Run tests
cargo test --workspace
```


### **Quick Demo**
```shell script
# Test RabbitMQ monitoring
cd rabbitmq-info
cargo run -- status --json

# Test secure database connection (requires Yubikey)
cd pg_vault
cargo run --features hardware-yubikey -- test-yubikey

# Launch RabbitMQ UI
cd rabbitmq-ui
cargo run
```


## 🎪 **Usage Examples**

### **For System Administrators**
```shell script
# Monitor RabbitMQ cluster health
rabbitmq-info status --cluster production --output json

# Secure database maintenance with hardware authentication
pg_vault connect --require-yubikey --audit-log --database production
```


### **For Developers**
```rust
use pg_vault::prelude::*;

// Hardware-secured database connections
let vault = Vault::new_with_hardware(auth_config, db_config, vault_config)?;
let connection = vault.connect().await?;

// Execute queries with automatic session tracking
let client = connection.client();
let rows = client.query("SELECT * FROM users WHERE active = true", &[]).await?;
```


### **For Operations Teams**
```shell script
# Export RabbitMQ configuration
rabbitmq-config export --format json --output backup.json

# Import configuration to new cluster
rabbitmq-config import --config backup.json --cluster staging
```


## 🏗️ **Architecture**

```
┌─────────────────────────────────────────────────────────────┐
│                    RabbitMQ Workspace                      │
├─────────────────────────────────────────────────────────────┤
│  CLI Tools          │  GUI Applications  │  Core Libraries  │
│  ──────────         │  ────────────────  │  ─────────────   │
│  messaging_cli      │  rabbitmq-ui       │  messaging_commands │
│  pg_vault           │  egui-components   │  yak_json        │
├─────────────────────────────────────────────────────────────┤
│              Infrastructure & Security Layer                │
│  ───────────────────────────────────────────────────────   │
│  • Hardware Authentication (Yubikey)                       │
│  • Session Management & Audit Trails                       │
│  • Configuration Management                                 │
│  • Monitoring & Metrics Collection                         │
└─────────────────────────────────────────────────────────────┘
```


## 🔧 **Configuration**

### **Workspace Configuration**
```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "messaging_commands",
    "messaging_cli", 
    "pg_vault",
    "rabbitmq-config",
    "rabbitmq-info",
    "rabbitmq-ui",
    "egui-components",
    "yak_json"
]
```


### **Database Security Setup**
```toml
# ~/.config/pg_vault/config.toml
[auth]
primary_serial = "12345678"
backup_serials = ["87654321"]
require_multi_factor = true

[database]
host = "localhost"
port = 5432
ssl_mode = "require"

[vault]
max_session_idle_minutes = 30
require_token_per_connection = true
```


## 🛠️ **Development**

### **Adding New Components**
```shell script
# Create new crate in workspace
cargo new --lib my_component
cd my_component

# Add to workspace Cargo.toml
# Add dependencies to other workspace crates
```


### **Testing Strategy**
```shell script
# Unit tests
cargo test --package pg_vault

# Integration tests
cargo test --test integration

# Hardware tests (requires physical device)
cargo test --features hardware-yubikey --test hardware_integration
```


## 📊 **Roadmap**

### **Phase 1: Core Infrastructure** ✅
- [x] Basic workspace structure
- [x] Core messaging library
- [x] PostgreSQL vault with mock authentication
- [x] Configuration management

### **Phase 2: Security Integration** 🔄
- [x] Yubikey hardware authentication
- [ ] Multi-device backup support
- [ ] Advanced session management
- [ ] Audit log improvements

### **Phase 3: User Interfaces** 🔄
- [ ] CLI tool completion
- [ ] GUI application stability
- [ ] Cross-platform packaging
- [ ] Documentation improvements

### **Phase 4: Enterprise Features** 📋
- [ ] Clustering support
- [ ] Metrics and monitoring
- [ ] Deployment automation
- [ ] Performance optimization

## 🤝 **Contributing**

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** and add tests
4. **Run the test suite**: `cargo test --workspace`
5. **Submit a pull request**

### **Development Guidelines**
- Follow Rust conventions and `rustfmt`
- Add tests for new functionality
- Update documentation
- Consider security implications

## 📜 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 **Support**

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/username/rabbitmq_workspace/issues)
- **Discussions**: [GitHub Discussions](https://github.com/username/rabbitmq_workspace/discussions)

## 🏆 **Acknowledgments**

- **RabbitMQ** team for the excellent messaging platform
- **Yubico** for hardware security standards
- **Rust community** for the amazing ecosystem
- **egui** for cross-platform GUI capabilities

---

**Built with ❤️ and 🦀 by the RabbitMQ Workspace team**

*Secure infrastructure tooling for the modern enterprise*
