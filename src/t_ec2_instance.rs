use aws_sdk_ec2::types::Instance;

#[derive(Debug)]
pub struct EC2Instance {
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
}
