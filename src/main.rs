mod actions;
mod aws;

use actions::get_actions;
use aws_sdk_sfn::Error;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let actions = get_actions();

    let selected_action = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the Action:")
        .items(&actions)
        .default(0)
        .interact_on(&Term::buffered_stderr())
        .unwrap();

    actions[selected_action].execute().await
}
