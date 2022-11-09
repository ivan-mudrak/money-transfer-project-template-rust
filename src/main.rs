use money_transfer_project_template_rust::shared::{MONEY_TRANSFER_TASK_QUEUE_NAME, NAMESPACE, WORKFLOW_NAME, PaymentDetails};
use std::sync::Arc;
use temporal_client::{
    ClientOptionsBuilder, WfClientExt, WorkflowClientTrait, WorkflowExecutionResult,
    WorkflowOptions,
};
use temporal_sdk_core::Url;
mod banking_client;
use log::{error, info, LevelFilter};
use simple_logger::SimpleLogger;

use temporal_sdk_core_protos::{
    coresdk::{AsJsonPayloadExt}
};

#[tokio::main]
pub async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let client_options = ClientOptionsBuilder::default()
        .client_name("client_name")
        .client_version("0.1.0")
        .target_url(Url::try_from("http://localhost:7233").unwrap())
        .build()
        .unwrap();

    let client = Arc::new(
        client_options
            .connect(NAMESPACE, None, None)
            .await
            .expect("Must connect"),
    );

    let input = PaymentDetails {
        source_account: "85-150".to_string(),
        target_account: "43-812".to_string(),
        amount: 250,
    };

    info!("Starting transfer of {} from account {} to account {}", input.amount, input.source_account, input.target_account);

    let wf_exec_res = client
        .start_workflow(
            vec!(input.as_json_payload().expect("serializes fine")),
            MONEY_TRANSFER_TASK_QUEUE_NAME.to_string(),
            WORKFLOW_NAME.to_string(),
            WORKFLOW_NAME.to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await;

    match wf_exec_res {
        Ok(wf_res) => {
            let handle = client.get_untyped_workflow_handle(WORKFLOW_NAME, wf_res.run_id);
            if let WorkflowExecutionResult::Succeeded(_) = handle
                .get_workflow_result(Default::default())
                .await
                .unwrap()
            {
                info!("Workflow executed sucessfully")
            } else {
                error!("Workflow failed");
            }
        }
        Err(status) => {
            error!("Error executing workflow: {}", status);
        }
    }
}
