use crate::aws::model::{StateMachine, StateMachineExecution};
use aws_config::from_env;
use aws_sdk_sfn::model::ExecutionStatus;
use aws_sdk_sfn::{Client, Error};
use aws_smithy_types::date_time::Format;

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

//TODO: implement pagination
pub async fn list_failed_executions(
    machine: &StateMachine,
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
            arn: execution.state_machine_arn.unwrap(),
            name: execution.name.unwrap(),
            status: execution.status.unwrap(),
            start_date: execution.start_date.unwrap().fmt(Format::DateTime).unwrap(),
            input: Option::None,
            output: Option::None,
        })
        .collect::<Vec<StateMachineExecution>>();

    Ok(executions)
}

pub async fn describe_execution(execution_arn: String) -> Result<StateMachineExecution, Error> {
    let client = build_client().await;
    let req = client.describe_execution().execution_arn(execution_arn);

    let raw_execution = req.send().await?;
    let execution = StateMachineExecution {
        arn: raw_execution.state_machine_arn.unwrap(),
        name: raw_execution.name.unwrap(),
        status: raw_execution.status.unwrap(),
        start_date: raw_execution
            .start_date
            .unwrap()
            .fmt(Format::DateTime)
            .unwrap(),
        input: raw_execution.input,
        output: raw_execution.output,
    };

    Ok(execution)
}

async fn build_client() -> Client {
    let config = from_env().load().await;
    Client::new(&config)
}
