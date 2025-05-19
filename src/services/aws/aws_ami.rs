pub enum Ami {
    /// (04-05-2025) Amazon Linux 2, 64 bit, ARM
    AmazonLinux64BitArm,
}

impl ToString for Ami {
    fn to_string(&self) -> String {
        match self {
            Self::AmazonLinux64BitArm => "ami-0400ee32fb141782f".into(),
        }
    }
}
