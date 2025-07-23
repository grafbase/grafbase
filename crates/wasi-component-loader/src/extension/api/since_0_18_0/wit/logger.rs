use std::path::PathBuf;

use dashmap::Entry;
use rolling_logger::RotateStrategy;
use valuable::{Valuable, Value, Visit};
use wasmtime::component::Resource;

use crate::InstanceState;

pub use super::grafbase::sdk::logger::*;

impl Host for InstanceState {}

impl HostFileLogger for InstanceState {
    async fn init(&mut self, options: FileLoggerOptions) -> wasmtime::Result<Result<Resource<FileLogger>, String>> {
        let logger = match self.file_loggers.entry(options.path.clone()) {
            Entry::Occupied(occupied_entry) => occupied_entry.get().clone(),
            Entry::Vacant(vacant_entry) => {
                let path = PathBuf::from(&options.path);

                let strategy = match options.rotate {
                    Some(FileLoggerRotation::Size(bytes)) => RotateStrategy::size(bytes),
                    Some(FileLoggerRotation::Minutely) => RotateStrategy::minutely(),
                    Some(FileLoggerRotation::Hourly) => RotateStrategy::hourly(),
                    Some(FileLoggerRotation::Daily) => RotateStrategy::daily(),
                    Some(FileLoggerRotation::Weekly) => RotateStrategy::weekly(),
                    Some(FileLoggerRotation::Monthly) => RotateStrategy::monthly(),
                    Some(FileLoggerRotation::Yearly) => RotateStrategy::yearly(),
                    None => RotateStrategy::never(),
                };

                match FileLogger::new(path, strategy) {
                    Ok(logger) => {
                        vacant_entry.insert(logger.clone());
                        logger
                    }
                    Err(err) => return Ok(Err(err.to_string())),
                }
            }
        };

        Ok(Ok(self.resources.push(logger)?))
    }

    async fn log(&mut self, self_: Resource<FileLogger>, data: Vec<u8>) -> wasmtime::Result<Result<(), String>> {
        let this = self.resources.get_mut(&self_)?;

        match this.send(data).map_err(|e| e.to_string()) {
            Ok(()) => Ok(Ok(())),
            Err(e) => Ok(Err(e.to_string())),
        }
    }

    async fn drop(&mut self, rep: Resource<FileLogger>) -> wasmtime::Result<()> {
        let logger = self.resources.get(&rep)?;
        logger.graceful_shutdown().await;

        Ok(())
    }
}

/// Wrapper for guest fields that implements Valuable for structured logging
struct GuestFields(Vec<(String, String)>);

impl Valuable for GuestFields {
    fn as_value(&self) -> Value<'_> {
        Value::Mappable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        for (key, value) in &self.0 {
            visit.visit_entry(Value::String(key.as_str()), Value::String(value.as_str()));
        }
    }
}

impl valuable::Mappable for GuestFields {
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.0.len();
        (size, Some(size))
    }
}

impl HostSystemLogger for InstanceState {
    async fn log(&mut self, LogEntry { level, message, fields }: LogEntry) -> wasmtime::Result<()> {
        match level {
            LogLevel::Trace => {
                if !fields.is_empty() {
                    let fields = GuestFields(fields);
                    tracing::trace!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                        guest_fields = fields.as_value(),
                    );
                } else {
                    tracing::trace!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                    );
                }
            }
            LogLevel::Debug => {
                if !fields.is_empty() {
                    let fields = GuestFields(fields);
                    tracing::debug!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                        guest_fields = fields.as_value(),
                    );
                } else {
                    tracing::debug!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                    );
                }
            }
            LogLevel::Info => {
                if !fields.is_empty() {
                    let fields = GuestFields(fields);
                    tracing::info!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                        guest_fields = fields.as_value(),
                    );
                } else {
                    tracing::info!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                    );
                }
            }
            LogLevel::Warn => {
                if !fields.is_empty() {
                    let fields = GuestFields(fields);
                    tracing::warn!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                        guest_fields = fields.as_value(),
                    );
                } else {
                    tracing::warn!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                    );
                }
            }
            LogLevel::Error => {
                if !fields.is_empty() {
                    let fields = GuestFields(fields);
                    tracing::error!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                        guest_fields = fields.as_value(),
                    );
                } else {
                    tracing::error!(
                        target: "extension",
                        extension = %self.extension_name(),
                        message = %message,
                    );
                }
            }
        }

        Ok(())
    }

    async fn drop(&mut self, _: wasmtime::component::Resource<SystemLogger>) -> wasmtime::Result<()> {
        // Nothing to do here, it's a singleton.
        Ok(())
    }
}
