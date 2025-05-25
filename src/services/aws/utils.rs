use aws_sdk_ec2::{
    self as ec2,
    types::{Instance, InstanceStateName},
};
use log::info;
use reqwest::StatusCode;

pub async fn create_ec2_client() -> ec2::Client {
    let ec2_config = aws_config::load_from_env().await;
    let client = ec2::Client::new(&ec2_config);

    return client;
}

pub async fn wait_for_healthy_machine(http_client: reqwest::Client, ip: &String) -> () {
    let formatted_domain = format!("http://{}", ip);

    loop {
        info!("Waiting for machine to be healthy");

        match http_client.get(&formatted_domain).send().await {
            Ok(res) => {
                let status = res.status();

                if StatusCode::is_success(&status) {
                    break;
                }

                info!("Status: {}", status);
            }
            Err(_) => {
                info!("Polling")
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
}

pub async fn wait_for_running_machine(
    client: &ec2::Client,
    instance_id: &String,
) -> Result<Instance, String> {
    let instance = loop {
        let running_instance = client
            .describe_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .reservations
            .into_iter()
            .flatten()
            .flat_map(|r| r.instances.unwrap_or_default())
            .next()
            .unwrap();

        if let Some(state) = running_instance.state().unwrap().name() {
            if *state == InstanceStateName::Running {
                break running_instance;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    };

    Ok(instance)
}
