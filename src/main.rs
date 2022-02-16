mod aws;

use aws::step_functions::{list_failed_executions, list_machines};
use aws_sdk_sfn::Error;
use console::Term;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let machines = list_machines().await?;
    let term = Term::buffered_stderr();
    let theme = ColorfulTheme::default();

    let selected = Select::with_theme(&theme)
        .with_prompt("Select the Machine:")
        .items(&machines)
        .interact_on(&term)
        .unwrap();

    let failed_executions = list_failed_executions(&machines[selected]).await?;

    MultiSelect::with_theme(&theme)
        .with_prompt("Select the executions to retry:")
        .items(&failed_executions)
        .interact_on(&term)
        .unwrap();

    Ok(())
}
