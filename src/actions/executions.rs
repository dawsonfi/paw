use crate::aws::model::{ExecutionInput, StateMachineExecution};
use crate::aws::step_functions::StepFunctionsMachine;
use aws_sdk_sfn::Error;
use chrono::{DateTime, ParseError, Utc};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};

pub async fn retry_failed_executions() -> Result<(), Error> {
    let term = Term::buffered_stderr();
    let theme = ColorfulTheme::default();

    let machine = StepFunctionsMachine::new().await;

    let machines = machine.list_machines().await?;
    let selected_machine = Select::with_theme(&theme)
        .with_prompt("Select the Machine:")
        .items(&machines)
        .interact_on(&term)
        .unwrap();

    let start_date = get_user_date_input("Start Date (ex. 1989-09-30 22:10:32 -03:00): ");
    let end_date = get_user_date_input("End Date (ex. 1989-09-30 23:15:00 -03:00): ");

    let failed_executions = machine
        .list_failed_executions(&machines[selected_machine], start_date, end_date)
        .await?;
    let checked_executions: Vec<(StateMachineExecution, bool)> = failed_executions
        .iter()
        .map(|execution| (execution.clone(), true))
        .collect();

    // TODO: check empty before going to multiselect

    if !checked_executions.is_empty() {
        let selected_executions_to_retry = MultiSelect::with_theme(&theme)
            .with_prompt("Select the executions to retry:")
            .items_checked(&checked_executions)
            .interact_on(&term)
            .unwrap();
        return retry_selected_failed_executions(selected_executions_to_retry, failed_executions)
            .await;
    }

    Ok(())
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
    let machine = StepFunctionsMachine::new().await;
    let progress_bar = ProgressBar::new(selected_executions_to_retry.len() as u64);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}]({pos} of {len}) ID: {msg}")
        .progress_chars("#>-"));

    for index in selected_executions_to_retry.into_iter() {
        let execution = &failed_executions[index];
        let full_execution = machine.describe_execution(execution.arn.clone()).await?;

        progress_bar.set_message(format!("{}", full_execution.name));
        progress_bar.inc(1);

        machine
            .start_execution(ExecutionInput {
                machine_arn: full_execution.machine_arn,
                input: full_execution.input.unwrap(),
            })
            .await?;
    }

    Ok(())
}
