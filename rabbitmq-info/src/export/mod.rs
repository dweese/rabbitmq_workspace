// rabbitmq-info/src/export/mod.rs

use crate::RabbitMQInfo;
use serde_json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub enum ExportFormat {
    Json,
    Csv,
    // Add more formats as needed
}

pub struct RabbitMQExporter;

impl RabbitMQExporter {
    pub fn export_to_file(
        info: &RabbitMQInfo,
        path: &Path,
        format: ExportFormat,
    ) -> Result<(), crate::InfoError> {
        match format {
            ExportFormat::Json => Self::export_to_json(info, path),
            ExportFormat::Csv => Self::export_to_csv(info, path),
        }
    }

    fn export_to_json(info: &RabbitMQInfo, path: &Path) -> Result<(), crate::InfoError> {
        let json = serde_json::to_string_pretty(info)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn export_to_csv(info: &RabbitMQInfo, path: &Path) -> Result<(), crate::InfoError> {
        // Basic implementation for CSV export
        let mut content = String::new();

        // Header for server info
        content.push_str("# Server Information\n");
        content
            .push_str("version,erlang_version,cluster_name,management_version,uptime,node_name\n");
        content.push_str(&format!(
            "{},{},{},{},{},{}\n\n",
            info.server.version,
            info.server.erlang_version,
            info.server.cluster_name,
            info.server.management_version,
            info.server.uptime,
            info.server.node_name
        ));

        // Header for exchanges
        content.push_str("# Exchanges\n");
        content.push_str("name,vhost,type,durable,auto_delete,internal\n");
        for ex in &info.exchanges {
            content.push_str(&format!(
                "{},{},{},{},{},{}\n",
                ex.name, ex.vhost, ex.exchange_type, ex.durable, ex.auto_delete, ex.internal
            ));
        }
        content.push('\n'); // Changed from content.push_str("\n");

        // Header for queues
        content.push_str("# Queues\n");
        content.push_str("name,vhost,durable,auto_delete,exclusive,messages,messages_ready,messages_unacknowledged\n");
        for q in &info.queues {
            content.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                q.name,
                q.vhost,
                q.durable,
                q.auto_delete,
                q.exclusive,
                q.messages.unwrap_or(0),
                q.messages_ready.unwrap_or(0),
                q.messages_unacknowledged.unwrap_or(0)
            ));
        }
        content.push('\n'); // Changed from content.push_str("\n");

        // Header for bindings
        content.push_str("# Bindings\n");
        content.push_str("source,destination,destination_type,routing_key,vhost\n");
        for b in &info.bindings {
            content.push_str(&format!(
                "{},{},{},{},{}\n",
                b.source, b.destination, b.destination_type, b.routing_key, b.vhost
            ));
        }

        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
