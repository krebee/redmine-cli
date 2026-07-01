mod config;
mod context;
mod issues;
mod projects;

use crate::cli::{
    Cli, Command, ConfigSubcommand, IssuesSubcommand, OutputFormat, ProjectsSubcommand,
};
use crate::error::AgentError;
use crate::output::{print_result, CommandResult};

use self::context::{ClientContext, CommandOutput};

pub async fn run(cli: Cli) -> i32 {
    let operation = operation(&cli.command);
    let format = output_format(&cli);

    match run_inner(cli).await {
        Ok(result) => match print_result(&result, &format) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("{}", error.user_message());
                1
            }
        },
        Err(error) => {
            let result = CommandResult::failure(operation, &error);
            if print_result(&result, &format).is_err() {
                eprintln!("{}", error.user_message());
            }
            1
        }
    }
}

fn output_format(cli: &Cli) -> OutputFormat {
    if cli.json {
        OutputFormat::Json
    } else {
        cli.format.clone()
    }
}

const fn operation(command: &Command) -> &'static str {
    match command {
        Command::Config(command) => match command.command {
            ConfigSubcommand::Init { .. } => "config.init",
            ConfigSubcommand::Show => "config.show",
        },
        Command::Projects(command) => match command.command {
            ProjectsSubcommand::List { .. } => "projects.list",
            ProjectsSubcommand::Get { .. } => "projects.get",
        },
        Command::Issues(command) => match command.command {
            IssuesSubcommand::Get { .. } => "issues.get",
            IssuesSubcommand::List { .. } => "issues.list",
            IssuesSubcommand::Create { .. } => "issues.create",
            IssuesSubcommand::Update { .. } => "issues.update",
            IssuesSubcommand::Comment { .. } => "issues.comment",
        },
    }
}

async fn run_inner(cli: Cli) -> Result<CommandResult, AgentError> {
    let operation = operation(&cli.command);

    let output = match cli.command {
        Command::Config(command) => CommandOutput::local(config::run(command.command)?),
        Command::Projects(command) => {
            let context = ClientContext::load(cli.profile.as_deref(), cli.timeout_ms)?;
            context.output(projects::run(&context.client, command.command).await?)
        }
        Command::Issues(command) => {
            let context = ClientContext::load(cli.profile.as_deref(), cli.timeout_ms)?;
            context.output(issues::run(&context.client, &context.profile, command.command).await?)
        }
    };

    Ok(output.into_result(operation))
}
