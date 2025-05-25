use base64::{Engine as _, engine::general_purpose};

use crate::services::user_data::MachineUserData;

pub struct AWSMachineUserData(pub MachineUserData);

impl ToString for AWSMachineUserData {
    fn to_string(&self) -> String {
        let formatted_string = format!(
            "
        #!/bin/bash
        yum update -y

        sudo amazon-linux-extras enable docker
        sudo yum install -y docker

        sudo systemctl start docker
        sudo systemctl enable docker

        sudo usermod -a -G docker ec2-user

        sudo docker run -d --restart=always -p 80:80 {}

        ",
            self.0.docker_image
        );

        let encoded = general_purpose::STANDARD.encode(formatted_string);

        return encoded;
    }
}
