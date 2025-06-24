use std::{collections::BTreeMap, path::PathBuf};

use dashmap::Entry;
use rolling_logger::RotateStrategy;
use wasmtime::component::Resource;

use crate::WasiState;

pub use super::grafbase::sdk::logger::*;

impl Host for WasiState {}

impl HostFileLogger for WasiState {
    async fn init(&mut self, options: FileLoggerOptions) -> wasmtime::Result<Result<Resource<FileLogger>, String>> {
        let logger = match self.file_loggers().entry(options.path.clone()) {
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

        Ok(Ok(self.push_resource(logger)?))
    }

    async fn log(&mut self, self_: Resource<FileLogger>, data: Vec<u8>) -> wasmtime::Result<Result<(), String>> {
        let this = self.get_mut(&self_)?;

        match this.send(data).map_err(|e| e.to_string()) {
            Ok(()) => Ok(Ok(())),
            Err(e) => Ok(Err(e.to_string())),
        }
    }

    async fn drop(&mut self, rep: Resource<FileLogger>) -> wasmtime::Result<()> {
        let logger = self.get(&rep)?;
        logger.graceful_shutdown().await;

        Ok(())
    }
}

impl HostSystemLogger for WasiState {
    async fn log(&mut self, LogEntry { level, message, fields }: LogEntry) -> wasmtime::Result<()> {
        let guest_fields: BTreeMap<_, _> = fields.into_iter().collect();

        match level {
            LogLevel::Trace => {
                tracing::trace!(
                    target: "extension",
                    extension = %self.extension_name(),
                    message = %message,
                    fields = ?guest_fields,
                );
            }
            LogLevel::Debug => {
                tracing::debug!(
                    target: "extension",
                    extension = %self.extension_name(),
                    message = %message,
                    fields = ?guest_fields,
                );
            }
            LogLevel::Info => {
                tracing::info!(
                    target: "extension",
                    extension = %self.extension_name(),
                    message = %message,
                    fields = ?guest_fields,
                );
            }
            LogLevel::Warn => {
                tracing::warn!(
                    target: "extension",
                    extension = %self.extension_name(),
                    message = %message,
                    fields = ?guest_fields,
                );
            }
            LogLevel::Error => {
                tracing::error!(
                    target: "extension",
                    extension = %self.extension_name(),
                    message = %message,
                    fields = ?guest_fields,
                );
            }
        }

        Ok(())
    }

    async fn drop(&mut self, _: wasmtime::component::Resource<SystemLogger>) -> wasmtime::Result<()> {
        // Nothing to do here, it's a singleton.
        Ok(())
    }
}
