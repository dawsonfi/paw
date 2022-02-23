use crate::aws::model::{ExecutionInput, StateMachine, StateMachineExecution};
use aws_config::from_env;
use aws_sdk_sfn::model::ExecutionStatus;
use aws_sdk_sfn::{Client, Error};
use chrono::{DateTime, NaiveDateTime, Utc};

pub async fn list_machines() -> Result<Vec<StateMachine>, Error> {
    let client = build_client().await;
    let req = client.list_state_machines();
    let machines = req.send().await?;
    let machine_names = machines
        .state_machines
        .unwrap()
        .into_iter()
        .map(|machine| StateMachine {
            arn: machine.state_machine_arn.unwrap(),
            name: machine.name.unwrap(),
        })
        .collect::<Vec<StateMachine>>();

    Ok(machine_names)
}

pub async fn list_failed_executions(
    machine: &StateMachine,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<Vec<StateMachineExecution>, Error> {
    let client = build_client().await;
    let req = client
        .list_executions()
        .state_machine_arn(&machine.arn)
        .max_results(1000)
        .status_filter(ExecutionStatus::Failed);
    let raw_executions = req.send().await?;
    let executions = raw_executions
        .executions
        .unwrap()
        .into_iter()
        .map(|execution| StateMachineExecution {
            arn: execution.execution_arn.unwrap(),
            machine_arn: execution.state_machine_arn.unwrap(),
            name: execution.name.unwrap(),
            start_date: to_date_time(execution.start_date.unwrap()),
            input: Option::None,
            output: Option::None,
        })
        .filter(|execution| {
            let start = start_date.unwrap_or(Utc::now());
            let end = end_date.unwrap_or(Utc::now());
            let execution_time = execution.start_date;

            if start_date.is_some() && end_date.is_some() {
                return start <= execution_time && end >= execution_time;
            } else if start_date.is_some() {
                return start <= execution_time;
            } else if end_date.is_some() {
                return end >= execution_time;
            }

            true
        })
        .collect::<Vec<StateMachineExecution>>();

    Ok(executions)
}

pub async fn describe_execution(execution_arn: String) -> Result<StateMachineExecution, Error> {
    let client = build_client().await;
    let req = client.describe_execution().execution_arn(execution_arn);

    let raw_execution = req.send().await?;
    let execution = StateMachineExecution {
        arn: raw_execution.execution_arn.unwrap(),
        machine_arn: raw_execution.state_machine_arn.unwrap(),
        name: raw_execution.name.unwrap(),
        start_date: to_date_time(raw_execution.start_date.unwrap()),
        input: raw_execution.input,
        output: raw_execution.output,
    };

    Ok(execution)
}

pub async fn start_execution(input: ExecutionInput) -> Result<(), Error> {
    let client = build_client().await;
    let req = client
        .start_execution()
        .state_machine_arn(input.machine_arn)
        .input(input.input);

    req.send().await?;
    Ok(())
}

async fn build_client() -> Client {
    let config = from_env().load().await;
    Client::new(&config)
}

fn to_date_time(date: aws_smithy_types::DateTime) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(date.secs(), date.subsec_nanos()),
        Utc,
    )
}
