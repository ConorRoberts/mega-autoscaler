use aws_sdk_ec2 as ec2;

pub async fn create_ec2_client() -> ec2::Client {
    let ec2_config = aws_config::load_from_env().await;
    let client = ec2::Client::new(&ec2_config);

    return client;
}
