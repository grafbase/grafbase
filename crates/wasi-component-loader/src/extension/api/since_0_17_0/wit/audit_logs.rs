use wasmtime::component::Resource;

use crate::WasiState;

pub use super::grafbase::sdk::audit_logs::*;

impl Host for WasiState {}

impl HostAuditLogs for WasiState {
    async fn push(&mut self, _: Resource<AuditLogs>, _: Vec<u8>) -> wasmtime::Result<()> {
        todo!("this is not implemented yet in the host")
    }

    async fn pop(&mut self, _: Resource<AuditLogs>) -> wasmtime::Result<Option<LogEntry>> {
        todo!("this is not implemented yet in the host")
    }

    async fn drop(&mut self, res: Resource<AuditLogs>) -> wasmtime::Result<()> {
        self.table.delete(res)?;
        Ok(())
    }
}
