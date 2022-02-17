mod actions;
mod aws;

use actions::executions::retry_failed_executions;
use aws_sdk_sfn::Error;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let actions = vec!["Retry Failed Executions"];

    let selected_action = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the Action:")
        .items(&actions)
        .default(0)
        .interact_on(&Term::buffered_stderr())
        .unwrap();

    match actions[selected_action] {
        "Retry Failed Executions" => match retry_failed_executions().await {
            Ok(_) => println!("Success"),
            Err(error) => println!("Error on processing action: {}", error),
        },
        _ => {}
    }

    Ok(())
}
