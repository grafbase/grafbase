use clap::{Parser, ValueEnum};

use super::{filter_existing_arguments, ArgumentNames};

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum ConfigFormat {
    /// Adds a TypeScript configuration file
    TypeScript,
    /// Adds a GraphQL configuration file
    GraphQL,
}

#[derive(Debug, Parser)]
pub struct InitCommand {
    /// The name of the project to create
    pub name: Option<String>,
    /// The name or GitHub URL of the template to use for the new project
    #[arg(short, long)]
    pub template: Option<String>,
    /// The format used for the Grafbase configuration file
    #[arg(short, long)]
    pub config_format: Option<ConfigFormat>,
}

impl InitCommand {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn template(&self) -> Option<&str> {
        self.template.as_deref()
    }
}

impl ArgumentNames for InitCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        filter_existing_arguments(&[(self.name.is_some(), "name"), (self.template.is_some(), "template")])
    }
}
