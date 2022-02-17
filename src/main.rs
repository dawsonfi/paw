mod aws;

use aws::model::{ExecutionInput, StateMachineExecution};
use aws::step_functions::{
    describe_execution, list_failed_executions, list_machines, start_execution,
};
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

    let failed_executions = list_failed_executions(&machines[selected], None, None).await?;
    let checked_executions: Vec<(StateMachineExecution, bool)> = failed_executions
        .iter()
        .map(|execution| (execution.clone(), true))
        .collect();

    let result = MultiSelect::with_theme(&theme)
        .with_prompt("Select the executions to retry:")
        .items_checked(&checked_executions)
        .interact_on(&term)
        .unwrap();

    let mut count = 0;
    let result_len = result.len();
    for index in result.into_iter() {
        let execution = &failed_executions[index];
        count = count + 1;
        println!(
            "Starting execution: {} ({} of {})",
            execution.name, count, result_len
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
