use std::thread;
use std::time::Instant;
use async_trait::async_trait;
use subprocess::PopenConfig;
use signal_hook::{consts::SIGTSTP, consts::SIGINT, iterator::Signals};

use crate::commands::{Command, notify, to_hms};
use crate::errors::jaws_error::JawsError;
use crate::Options;

pub struct SSMCommand {
    instance_id: String,
}

impl SSMCommand {
    pub fn new(instance_id: &String) -> Self {
        Self {
            instance_id: instance_id.to_string()
        }
    }

    /// Set up ctrl-C and ctrl-Z signal handlers so they are passed to the subprocess
    fn set_signal_handlers() {
        let mut signals = Signals::new(&[SIGTSTP, SIGINT]).expect("Couldn't create Signals instance.");

        thread::spawn(move || {
            for _sig in signals.forever() {
                // println!("Received signal {:?}", sig);
                // We don't need to do anything.
            }
        });
    }
}

#[async_trait]
impl Command for SSMCommand {
    async fn run(&mut self, _options: &Options) -> Result<(), JawsError> {
        notify(format!("Opening SSM session with {}\n", self.instance_id));

        let start_time = Instant::now();

        // The implementation of this should really be to get the WSS urls
        // and attach them to stdin/stdout.
        // The protocol is not trivial.  After sending a json header message, the protocol
        // switches to a binary format, described here:
        // https://github.com/aws/amazon-ssm-agent/blob/c65d8ac29a8bbe6cd3f7cea778c1eeb1b06d49a3/agent/session/contracts/agentmessage.go

        // To get something running, we use the old Jaws 2 way of spawning SSM - spawn the AWS
        // SSM module.

        let cmd_string = &["aws", "ssm", "start-session", "--target", &self.instance_id];
        Self::set_signal_handlers();
        let popen_res = subprocess::Popen::create(cmd_string, PopenConfig::default());
        popen_res.expect("Couldn't open the AWS SSM module, ensure it is installed.");

        // Session is complete here
        let session_length = start_time.elapsed().as_secs();
        notify(format!("Session closed, duration: {}\n", to_hms(session_length)));

        Ok(())
    }
}