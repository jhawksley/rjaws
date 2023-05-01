use aws_sdk_ec2::types::Instance;

#[derive(Debug)]
pub struct EC2Instance {
    pub is_extended: bool,

    pub instance: Instance,
    pub ssm: Option<bool>,
    pub az: Option<String>,
    pub instance_type: Option<String>,
    pub spec: Option<String>,
}

impl EC2Instance {
    /// Gets a name for this instance. The `name` tag is used first.  If that is not present,
    /// the tag `aws:eks:cluster-name` is used.  If that is also missing, the `None` variant
    /// is returned.
    pub fn get_name(&self) -> String {
        match self.find_tag_value("Name") {
            Some(string) => string,
            None => match self.find_tag_value("aws:eks:cluster-name") {
                Some(string) => format!("[EKS] {}", string),
                None => "Untitled".to_string()
            }
        }
    }

    pub fn find_tag_value(&self, key: &str) -> Option<String> {
        for tag in self.instance.tags.as_ref().unwrap() {
            if tag.key().unwrap() == key {
                return Some(tag.value().unwrap().to_string());
            }
        }

        None
    }

    pub fn values(&self, extended: bool) -> Vec<String> {
        let mut vec: Vec<String> = Vec::new();
        vec.push(self.instance.instance_id.as_ref().unwrap().clone());

        let name = self.get_name();
        vec.push(name);
        vec.push(self.instance.state().as_ref().unwrap().name().unwrap().as_str().to_string());


        // IP addresses may not be assigned
        vec.push(
            match &self.instance.public_ip_address {
                Some(address) => address.to_string(),
                None => "None".to_string()
            }
        );

        vec.push(
            match &self.instance.private_ip_address {
                Some(address) => address.to_string(),
                None => "None".to_string()
            }
        );

        // If this is an extended/wide display, also push the extended fields.
        if extended {
            vec.push(match self.ssm {
                Some(ssm) => if ssm { "Yes".to_string() } else { "No".to_string() },
                None => "-".to_string()
            });

            vec.push(match &self.az {
                Some(az) => az.to_string(),
                None => "-".to_string()
            });

            vec.push(match &self.instance_type {
                Some(it) => it.to_string(),
                None => "-".to_string()
            });

            vec.push(match &self.spec {
                Some(spec) => spec.to_string(),
                None => "-".to_string()
            });

        }

        vec
    }
}
