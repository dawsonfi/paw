use crate::aws::model::{ExecutionInput, StateMachine, StateMachineExecution};
use aws_config::from_env;
use aws_sdk_sfn::{
    error::{
        DescribeExecutionError, ListExecutionsError, ListStateMachinesError, StartExecutionError,
    },
    model::ExecutionStatus,
    output::{
        DescribeExecutionOutput, ListExecutionsOutput, ListStateMachinesOutput,
        StartExecutionOutput,
    },
    Client, Error,
};
use aws_smithy_http::result::SdkError;
use chrono::{DateTime, NaiveDateTime, Utc};

struct StepFunctionsClient {
    pub client: Client,
}

impl StepFunctionsClient {
    async fn new() -> StepFunctionsClient {
        let config = from_env().load().await;
        StepFunctionsClient {
            client: Client::new(&config),
        }
    }

    async fn list_state_machines(
        &self,
    ) -> Result<ListStateMachinesOutput, SdkError<ListStateMachinesError>> {
        self.client.list_state_machines().send().await
    }

    async fn list_failed_executions(
        &self,
        state_machine_arn: String,
        next_token: Option<String>,
    ) -> Result<ListExecutionsOutput, SdkError<ListExecutionsError>> {
        let mut req = self
            .client
            .list_executions()
            .state_machine_arn(state_machine_arn)
            .max_results(1000)
            .status_filter(ExecutionStatus::Failed);

        if next_token.is_some() {
            req = req.next_token(next_token.unwrap());
        }

        req.send().await
    }

    async fn describe_execution(
        &self,
        execution_arn: String,
    ) -> Result<DescribeExecutionOutput, SdkError<DescribeExecutionError>> {
        self.client
            .describe_execution()
            .execution_arn(execution_arn)
            .send()
            .await
    }

    async fn start_execution(
        &self,
        state_machine_arn: String,
        input: String,
    ) -> Result<StartExecutionOutput, SdkError<StartExecutionError>> {
        self.client
            .start_execution()
            .state_machine_arn(state_machine_arn)
            .input(input)
            .send()
            .await
    }
}

pub struct StepFunctionsMachine {
    client: StepFunctionsClient,
}

impl StepFunctionsMachine {
    pub async fn new() -> StepFunctionsMachine {
        StepFunctionsMachine {
            client: StepFunctionsClient::new().await,
        }
    }

    fn to_date_time(date: aws_smithy_types::DateTime) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(date.secs(), date.subsec_nanos()),
            Utc,
        )
    }
}

impl StepFunctionsMachine {
    pub async fn list_machines(&self) -> Result<Vec<StateMachine>, Error> {
        let machines = self.client.list_state_machines().await?;
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
        &self,
        machine: &StateMachine,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<StateMachineExecution>, Error> {
        let mut executions: Vec<StateMachineExecution> = vec![];
        let mut next_token: Option<String> = None;

        loop {
            let raw_executions = self
                .client
                .list_failed_executions(machine.arn.clone(), next_token)
                .await?;

            next_token = raw_executions.next_token;

            let mut partial_executions = raw_executions
                .executions
                .unwrap()
                .into_iter()
                .map(|execution| StateMachineExecution {
                    arn: execution.execution_arn.unwrap(),
                    machine_arn: execution.state_machine_arn.unwrap(),
                    name: execution.name.unwrap(),
                    start_date: StepFunctionsMachine::to_date_time(execution.start_date.unwrap()),
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

            executions.append(&mut partial_executions);

            if next_token.is_none() {
                break;
            }
        }

        Ok(executions)
    }

    pub async fn describe_execution(
        &self,
        execution_arn: String,
    ) -> Result<StateMachineExecution, Error> {
        let raw_execution = self.client.describe_execution(execution_arn).await?;

        let execution = StateMachineExecution {
            arn: raw_execution.execution_arn.unwrap(),
            machine_arn: raw_execution.state_machine_arn.unwrap(),
            name: raw_execution.name.unwrap(),
            start_date: StepFunctionsMachine::to_date_time(raw_execution.start_date.unwrap()),
            input: raw_execution.input,
            output: raw_execution.output,
        };

        Ok(execution)
    }

    pub async fn start_execution(&self, input: ExecutionInput) -> Result<(), Error> {
        self.client
            .start_execution(input.machine_arn, input.input)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    // use mockall::mock;
    // use super::*;

    #[test]
    fn test() {
        // let client = MockClient::new();
        // let machine = StepFunctionsMachine::new_with_client(client);
    }
}
