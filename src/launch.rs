mod services;

use clap::Parser;
use services::machine_orchestrator::MachineOrchestrator;
use services::{
    aws::{
        aws_machine_orchestrator::AWSMachineOrchestrator,
        aws_machine_user_data::AWSMachineUserData, utils::create_ec2_client,
    },
    user_data::MachineUserData,
};

#[derive(Parser, Debug)]
#[command(version,about,long_about=None)]
struct Args {
    #[arg(short, long)]
    docker_image: String,
}

#[tokio::main]
async fn main() -> () {
    env_logger::init();

    let args = Args::parse();

    let client = create_ec2_client().await;

    let orch = AWSMachineOrchestrator {
        client,
        user_data: AWSMachineUserData(MachineUserData {
            docker_image: args.docker_image,
        }),
    };

    orch.create_machine().await.unwrap();

    println!("Hello, World!");
    ()
}
