use crate::wit;

pub(crate) struct HostLogger(pub(crate) wit::SystemLogger);

impl log::Log for HostLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        let configured_level = match crate::component::guest_log_level() {
            l if l == log::Level::Error as u8 => log::Level::Error,
            l if l == log::Level::Warn as u8 => log::Level::Warn,
            l if l == log::Level::Info as u8 => log::Level::Info,
            l if l == log::Level::Debug as u8 => log::Level::Debug,
            l if l == log::Level::Trace as u8 => log::Level::Trace,
            _ => unreachable!("we do not have other levels"),
        };

        metadata.level() <= configured_level
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut kv_visitor = KvVisitor::new();

        let _ = record.key_values().visit(&mut kv_visitor);

        // 3. CONSTRUCT THE LOG ENTRY RECORD
        let entry = wit::LogEntry {
            level: record.level().into(),
            target: record.target().to_string(),
            message: record.args().to_string(),
            fields: kv_visitor.fields,
        };

        self.0.log(&entry);
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

impl log::kv::Visitor<'_> for KvVisitor {
    fn visit_pair(&mut self, key: log::kv::Key<'_>, value: log::kv::Value<'_>) -> Result<(), log::kv::Error> {
        self.fields.push((key.to_string(), value.to_string()));

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
