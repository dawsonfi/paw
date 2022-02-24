use crate::aws::model::{ExecutionInput, StateMachine, StateMachineExecution};
use aws_config::from_env;
use aws_sdk_sfn::model::ExecutionStatus;
use aws_sdk_sfn::{Client, Error};
use chrono::{DateTime, NaiveDateTime, Utc};
use async_trait::async_trait;

pub struct StepFunctionsMachine {
    pub client: Client
}

impl StepFunctionsMachine {
    pub async fn new() -> StepFunctionsMachine {
        let config = from_env().load().await;
        StepFunctionsMachine {
            client: Client::new(&config)
        }
    }

    fn to_date_time(date: aws_smithy_types::DateTime) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(date.secs(), date.subsec_nanos()),
            Utc,
        )
    }
}

#[async_trait]
pub trait StepMachine {
    async fn list_machines(&self) -> Result<Vec<StateMachine>, Error>;
    async fn list_failed_executions(
        &self,
        machine: &StateMachine,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<StateMachineExecution>, Error>;
    async fn describe_execution(&self, execution_arn: String) -> Result<StateMachineExecution, Error>;
    async fn start_execution(&self, input: ExecutionInput) -> Result<(), Error>;
}

#[async_trait]
impl StepMachine for StepFunctionsMachine {
    async fn list_machines(&self) -> Result<Vec<StateMachine>, Error> {        
        let req = self.client.list_state_machines();
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

    async fn list_failed_executions(
        &self,
        machine: &StateMachine,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<StateMachineExecution>, Error> {    
        let mut executions: Vec<StateMachineExecution> = vec![];
        let mut next_token: Option<String> = None;
        loop {
            let mut req = self.client
                .list_executions()
                .state_machine_arn(&machine.arn)
                .max_results(1000)
                .status_filter(ExecutionStatus::Failed);
    
            if next_token.is_some() {
                req = req.next_token(next_token.unwrap());
            }
    
            let raw_executions = req.send().await?;
    
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

    async fn describe_execution(&self, execution_arn: String) -> Result<StateMachineExecution, Error> {        
        let req = self.client.describe_execution().execution_arn(execution_arn);
    
        let raw_execution = req.send().await?;
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

    async fn start_execution(&self, input: ExecutionInput) -> Result<(), Error> {        
        let req = self.client
            .start_execution()
            .state_machine_arn(input.machine_arn)
            .input(input.input);
    
        req.send().await?;
        Ok(())
    }
}