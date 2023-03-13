// https://awslabs.github.io/aws-sdk-rust/

use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use aws_sdk_sts;
use aws_sdk_sts::output::GetCallerIdentityOutput;
use aws_sdk_ec2;
use aws_sdk_ec2::model::Instance;

use crate::error::jaws_error::JawsError;

// ***********************************************************************************************
// STS
// ***********************************************************************************************

/// Return the current context's STS caller identity.
pub async fn sts_get_caller_identity() -> Result<GetCallerIdentityOutput, JawsError> {
    let res = aws_sdk_sts::Client::new(&aws_config::load_from_env().await)
        .get_caller_identity().send().await;


    match res {
        Ok(output) => Ok(output),
        Err(e) => Err(JawsError::new(format!("Ensure your AWS credentials are set correctly in the environment. \nUnderlying error is: {:?}", e.into_service_error().message().unwrap())))
    }
}

// ***********************************************************************************************
// EC2
// ***********************************************************************************************


pub async fn ec2_get_all() -> Result<Vec<Instance>, JawsError> {
    let mut instances: Vec<Instance> = Vec::new();

    let client = aws_sdk_ec2::Client::new(&aws_config::load_from_env().await);

    let resp_result = client
        .describe_instances()
        .send()
        .await;

    match resp_result {
        Ok(resp) => {
            'res: for reservation in resp.reservations().unwrap_or_default() {
                for instance in reservation.instances().unwrap_or_default() {
                    // println!("{:?}", instance);
                    instances.push( instance.to_owned() );
                }
            }

            Ok(instances)
        },
        Err(error) => Err(JawsError::new(format!("{}", error)))
    }
}