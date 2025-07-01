use env_filter::Filter;

use crate::wit;

pub(crate) struct HostLogger {
    pub(crate) filter: Filter,
}

impl log::Log for HostLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.filter.matches(record) {
            return;
        }

        let mut kv_visitor = KvVisitor::new();
        let _ = record.key_values().visit(&mut kv_visitor);

        wit::SystemLogger::log(&wit::LogEntry {
            level: record.level().into(),
            message: record.args().to_string(),
            fields: kv_visitor.fields,
        });
    }

    fn flush(&self) {
        // We send logs immediately, so no-op.
    }
}

struct KvVisitor {
    fields: Vec<(String, String)>,
}

impl KvVisitor {
    fn new() -> Self {
        Self { fields: Vec::new() }
    }
}

struct ValueVisitor {
    result: Option<String>,
}

impl<'v> log::kv::VisitValue<'v> for ValueVisitor {
    fn visit_any(&mut self, value: log::kv::Value<'_>) -> Result<(), log::kv::Error> {
        // Fallback for any type we don't handle specifically
        self.result = Some(value.to_string());
        Ok(())
    }

    fn visit_null(&mut self) -> Result<(), log::kv::Error> {
        // Don't include null/None values
        self.result = None;
        Ok(())
    }

    fn visit_bool(&mut self, value: bool) -> Result<(), log::kv::Error> {
        self.result = Some(value.to_string());
        Ok(())
    }

    fn visit_str(&mut self, value: &str) -> Result<(), log::kv::Error> {
        self.result = Some(value.to_string());
        Ok(())
    }

    fn visit_borrowed_str(&mut self, value: &'v str) -> Result<(), log::kv::Error> {
        self.result = Some(value.to_string());
        Ok(())
    }

    fn visit_i64(&mut self, value: i64) -> Result<(), log::kv::Error> {
        self.result = Some(value.to_string());
        Ok(())
    }

    fn visit_u64(&mut self, value: u64) -> Result<(), log::kv::Error> {
        self.result = Some(value.to_string());
        Ok(())
    }

    fn visit_i128(&mut self, value: i128) -> Result<(), log::kv::Error> {
        self.result = Some(value.to_string());
        Ok(())
    }

    fn visit_u128(&mut self, value: u128) -> Result<(), log::kv::Error> {
        self.result = Some(value.to_string());
        Ok(())
    }

    fn visit_f64(&mut self, value: f64) -> Result<(), log::kv::Error> {
        self.result = Some(value.to_string());
        Ok(())
    }
}

impl log::kv::Visitor<'_> for KvVisitor {
    fn visit_pair(&mut self, key: log::kv::Key<'_>, value: log::kv::Value<'_>) -> Result<(), log::kv::Error> {
        let mut value_visitor = ValueVisitor { result: None };
        value.visit(&mut value_visitor)?;

        if let Some(value_str) = value_visitor.result {
            self.fields.push((key.to_string(), value_str));
        }

        Ok(())
    }
}

impl From<wit::LogLevel> for log::Level {
    fn from(value: wit::LogLevel) -> Self {
        match value {
            wit::LogLevel::Trace => log::Level::Trace,
            wit::LogLevel::Debug => log::Level::Debug,
            wit::LogLevel::Info => log::Level::Info,
            wit::LogLevel::Warn => log::Level::Warn,
            wit::LogLevel::Error => log::Level::Error,
        }
    }
}

impl From<log::Level> for wit::LogLevel {
    fn from(value: log::Level) -> Self {
        match value {
            log::Level::Error => wit::LogLevel::Error,
            log::Level::Warn => wit::LogLevel::Warn,
            log::Level::Info => wit::LogLevel::Info,
            log::Level::Debug => wit::LogLevel::Debug,
            log::Level::Trace => wit::LogLevel::Trace,
        }
    }
}
