use crate::create::CreateArguments;
use clap::{arg, command, CommandFactory, Parser};
use clap_complete::{shells, Generator};
use common::consts::{DEFAULT_LOG_FILTER, TRACE_LOG_FILTER};
use std::{fmt, path::PathBuf};

const DEFAULT_PORT: u16 = 4000;

#[derive(Debug, Parser)]
pub struct DevCommand {
    /// Use a specific port
    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
    /// If a given port is unavailable, search for another
    #[arg(short, long)]
    pub search: bool,
    /// Do not listen for schema changes and reload
    #[arg(long)]
    pub disable_watch: bool,
}

#[derive(Debug, Parser, Clone, Copy)]
pub enum Shell {
    /// Generate completions for bash
    Bash,
    /// Generate completions for fish
    Fish,
    /// Generate completions for zsh
    Zsh,
    /// Generate completions for elvish
    Elvish,
    /// Generate completions for powershell
    #[command(name = "powershell")]
    PowerShell,
}

impl Shell {
    pub fn completions(self) {
        match self {
            Shell::Bash => Self::completions_for_shell(shells::Bash),
            Shell::Fish => Self::completions_for_shell(shells::Fish),
            Shell::Zsh => Self::completions_for_shell(shells::Zsh),
            Shell::Elvish => Self::completions_for_shell(shells::Elvish),
            Shell::PowerShell => Self::completions_for_shell(shells::PowerShell),
        }
    }

    fn completions_for_shell(generator: impl Generator) {
        clap_complete::generate(generator, &mut Args::command(), "grafbase", &mut std::io::stdout());
    }
}

#[derive(Debug, Parser)]
pub struct CompletionsCommand {
    /// The shell to generate completions for
    #[command(subcommand)]
    pub shell: Shell,
}

#[derive(Debug, Parser)]
pub struct InitCommand {
    /// The name of the project to create
    pub name: Option<String>,
    /// The name or GitHub URL of the template to use for the new project
    #[arg(short, long)]
    pub template: Option<String>,
}

impl InitCommand {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn template(&self) -> Option<&str> {
        self.template.as_deref()
    }
}

#[derive(Debug, clap::Args)]
#[group(required = true, requires_all = ["name", "account", "regions"])]
pub struct CreateArgs {
    /// The name to use for the new project
    #[arg(short, long)]
    pub name: String,
    /// The slug of the account in which the new project should be created
    #[arg(short, long, value_name = "SLUG")]
    pub account: String,
    /// The regions in which the database for the new project should be created
    #[arg(short, long, value_name = "REGION")]
    pub regions: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct CreateCommand {
    #[command(flatten)]
    arguments: Option<CreateArgs>,
}

impl CreateCommand {
    pub fn create_arguments(&self) -> Option<CreateArguments<'_>> {
        self.arguments.as_ref().map(|args| CreateArguments {
            name: &args.name,
            account_slug: &args.account,
            regions: &args.regions,
        })
    }
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    /// Run your Grafbase project locally
    Dev(DevCommand),
    /// Output completions for the chosen shell to use, write the output to the
    /// appropriate location for your shell
    Completions(CompletionsCommand),
    /// Sets up the current or a new project for Grafbase
    Init(InitCommand),
    /// Resets the local database for the current project
    Reset,
    /// Logs into your Grafbase account
    Login,
    /// Logs out of your Grafbase account
    Logout,
    /// Set up and deploy a new project
    Create(CreateCommand),
    /// Deploy your project
    Deploy,
    /// Connect a local project to a remote project
    Link,
    /// Disconnect a local project from a remote project
    Unlink,
}

// TODO see if there's a way to do this automatically (https://github.com/clap-rs/clap/discussions/4921)
pub trait ArgumentNames {
    /// returns the argument names used in a specific invocation of the CLI
    fn argument_names(&self) -> Option<Vec<&'static str>>;
}

/// returns a tuple of (bool, &'static str), 0 being whether the condition on the field is met, 1 being the field name
macro_rules! argument_exists {
    ($self:ident, $field:ident$($condition:tt)+) => {
        ($self.$field$($condition)+, stringify!($field))
    };
    ($self:ident, $field:ident) => {
        ($self.$field, stringify!($field))
    };
}

fn filter_existing_arguments(arguments: &[(bool, &'static str)]) -> Option<Vec<&'static str>> {
    let arguments = arguments
        .iter()
        .filter(|arguments| arguments.0)
        .map(|arguments| arguments.1)
        .collect::<Vec<_>>();
    if arguments.is_empty() {
        None
    } else {
        Some(arguments)
    }
}

impl ArgumentNames for DevCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        filter_existing_arguments(&[
            argument_exists!(self, port != DEFAULT_PORT),
            argument_exists!(self, search),
            argument_exists!(self, disable_watch),
        ])
    }
}

impl ArgumentNames for InitCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        filter_existing_arguments(&[
            argument_exists!(self, name.is_some()),
            argument_exists!(self, template.is_some()),
        ])
    }
}

impl ArgumentNames for CreateCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        let arguments = [(self.arguments.is_some(), vec!["name", "account", "regions"])]
            .iter()
            .filter(|arguments| arguments.0)
            .flat_map(|arguments| arguments.1.clone())
            .collect::<Vec<_>>();
        if arguments.is_empty() {
            None
        } else {
            Some(arguments)
        }
    }
}

impl ArgumentNames for SubCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        match self {
            SubCommand::Dev(command) => command.argument_names(),
            SubCommand::Init(command) => command.argument_names(),
            SubCommand::Create(command) => command.argument_names(),
            SubCommand::Reset
            | SubCommand::Login
            | SubCommand::Logout
            | SubCommand::Deploy
            | SubCommand::Link
            | SubCommand::Unlink
            | SubCommand::Completions(_) => None,
        }
    }
}

impl SubCommand {
    pub(crate) fn in_project_context(&self) -> bool {
        matches!(
            self,
            Self::Dev(_) | Self::Create(_) | Self::Deploy | Self::Link | Self::Unlink | Self::Reset
        )
    }
}

impl AsRef<str> for SubCommand {
    fn as_ref(&self) -> &str {
        match self {
            SubCommand::Dev(_) => "dev",
            SubCommand::Completions(_) => "completions",
            SubCommand::Init(_) => "init",
            SubCommand::Reset => "reset",
            SubCommand::Login => "login",
            SubCommand::Logout => "logout",
            SubCommand::Create(_) => "create",
            SubCommand::Deploy => "deploy",
            SubCommand::Link => "link",
            SubCommand::Unlink => "unlink",
        }
    }
}

impl fmt::Display for SubCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Parser)]
#[command(name = "Grafbase CLI", version)]
/// The Grafbase command line interface
pub struct Args {
    /// Set the tracing level
    #[arg(short, long, default_value_t = 0)]
    pub trace: u16,
    #[command(subcommand)]
    pub command: SubCommand,
    /// An optional replacement path for the home directory
    #[arg(long)]
    pub home: Option<PathBuf>,
}

impl Args {
    pub fn log_filter(&self) -> &str {
        if self.trace >= 1 {
            TRACE_LOG_FILTER
        } else {
            DEFAULT_LOG_FILTER
        }
    }
}
