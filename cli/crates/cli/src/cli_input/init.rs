use clap::{Parser, ValueEnum};

use super::{filter_existing_arguments, ArgumentNames};

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "lowercase")]
pub enum GraphType {
    /// Creates a federated graph
    Federated,
    /// Creates a standalone graph
    Standalone,
}

#[derive(Debug, Parser)]
pub struct InitCommand {
    /// The name of the project to create
    pub name: Option<String>,
    /// The name or GitHub URL of the template to use for the new project
    #[arg(short, long)]
    pub template: Option<String>,
    /// What graph type (federated or standalone) to initialize the project with
    #[arg(short, long)]
    pub graph: Option<GraphType>,
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
