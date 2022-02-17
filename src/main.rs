mod aws;

use aws::model::{ExecutionInput, StateMachineExecution};
use aws::step_functions::{
    describe_execution, list_failed_executions, list_machines, start_execution,
};
use aws_sdk_sfn::Error;
use chrono::{DateTime, ParseError, Utc};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let term = Term::buffered_stderr();
    let theme = ColorfulTheme::default();
    let machines = list_machines().await?;

    let selected_machine = Select::with_theme(&theme)
        .with_prompt("Select the Machine:")
        .items(&machines)
        .interact_on(&term)
        .unwrap();

    let start_date = get_user_date_input("Start Date (ex. 1989-09-30 22:10:32 -03:00): ");
    let end_date = get_user_date_input("End Date (ex. 1989-09-30 23:15:00 -03:00): ");

    let failed_executions =
        list_failed_executions(&machines[selected_machine], start_date, end_date).await?;
    let checked_executions: Vec<(StateMachineExecution, bool)> = failed_executions
        .iter()
        .map(|execution| (execution.clone(), true))
        .collect();

    let selected_executions_to_retry = MultiSelect::with_theme(&theme)
        .with_prompt("Select the executions to retry:")
        .items_checked(&checked_executions)
        .interact_on(&term)
        .unwrap();
    retry_selected_failed_executions(selected_executions_to_retry, failed_executions).await
}

fn parse_utc_date_time(raw_date_time: String) -> Result<Option<DateTime<Utc>>, ParseError> {
    if raw_date_time.is_empty() {
        Ok(None)
    } else {
        let parsed_date = DateTime::parse_from_str(&raw_date_time, "%Y-%m-%d %H:%M:%S %z");
        match parsed_date {
            Ok(date) => Ok(Some(date.with_timezone(&Utc))),
            Err(error) => Err(error),
        }
    }
}

fn get_user_date_input(prompt_message: &str) -> Option<DateTime<Utc>> {
    loop {
        let date_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt_message)
            .allow_empty(true)
            .interact_on(&Term::buffered_stderr())
            .unwrap();
        let parsed_date = parse_utc_date_time(date_str);

        match parsed_date {
            Ok(date) => return date,
            Err(error) => println!("Invalid date ({}). Please try again!", error),
        }
    }
}

async fn retry_selected_failed_executions(
    selected_executions_to_retry: Vec<usize>,
    failed_executions: Vec<StateMachineExecution>,
) -> Result<(), Error> {
    let mut count = 0;
    let selected_executions_len = selected_executions_to_retry.len();
    for index in selected_executions_to_retry.into_iter() {
        let execution = &failed_executions[index];
        count = count + 1;
        println!(
            "Starting execution: {} ({} of {})",
            execution.name, count, selected_executions_len
        );

        let full_execution = describe_execution(execution.arn.clone()).await?;
        start_execution(ExecutionInput {
            machine_arn: full_execution.machine_arn,
            input: full_execution.input.unwrap(),
        })
        .await?;
    }

    Ok(())
}
