use colored::Colorize;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

pub struct OutputLayerEventFormatter;

impl<S, N> FormatEvent<S, N> for OutputLayerEventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();

        let level = match *metadata.level() {
            tracing::Level::ERROR => "error".red().bold(),
            tracing::Level::WARN => "warn".yellow().bold(),
            tracing::Level::INFO => "info".green().bold(),
            tracing::Level::DEBUG => "debug".purple().bold(),
            tracing::Level::TRACE => "trace".blue(),
        };

        match *metadata.level() {
            tracing::Level::ERROR => write!(writer, "{} - ", level)?,
            tracing::Level::WARN => write!(writer, "{}  - ", level)?,
            tracing::Level::INFO => write!(writer, "{}  - ", level)?,
            tracing::Level::DEBUG => write!(writer, "{} - ", level)?,
            tracing::Level::TRACE => write!(writer, "{} - ", level)?,
        };

        ctx.format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}
