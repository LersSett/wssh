use rusoto_core::region::Region;
use rusoto_opsworks::{DescribeInstancesRequest, DescribeStacksRequest, Instance, OpsWorks, OpsWorksClient, Stack};
use std::{fs::File, io::Write, process::exit, vec::Vec};

use helper::{home_dir, prepare_name};

#[derive(Debug, Clone)]
pub struct StackData {
  pub name:      String,
  pub endpoint:  InstanceData,
  pub instances: Vec<InstanceData>
}

#[derive(Debug, Clone)]
pub struct InstanceData {
  pub hostname:     String,
  pub ssh_key_name: String,
  pub elastic_ip:   Option<String>
}

impl StackData {
  fn to_vec_string(self) -> Vec<String> {
    let name = self.name;
    self
      .instances
      .into_iter()
      .map(|instance| {
        format!(
          "{stack_name} {endpoint_elastic_ip} {hostname} {ssh_key_name}\n",
          stack_name = name,
          endpoint_elastic_ip = instance.elastic_ip.unwrap_or("None".to_string()),
          hostname = instance.hostname,
          ssh_key_name = instance.ssh_key_name
        )
      })
      .collect::<Vec<String>>()
  }
}

pub fn stacks_data() {
  println!("Please wait...");

  let opsworks_client = OpsWorksClient::simple(Region::UsEast1);
  let stacks = fetch_stacks(&opsworks_client);
  let data = parse_stacks(&opsworks_client, stacks);
  write_data(data);

  println!("\rDone!");
}

fn fetch_stacks(opsworks_client: &OpsWorksClient) -> Vec<Stack> {
  match opsworks_client
    .describe_stacks(&DescribeStacksRequest::default())
    .sync()
  {
    Ok(result) =>
      match result.stacks {
        Some(stacks) => stacks,
        None => panic!("{}", "Zero stacks")
      },
    Err(error) => {
      println!("{:?}", error);
      exit(1);
    }
  }
}

fn parse_stacks(opsworks_client: &OpsWorksClient, stacks: Vec<Stack>) -> Vec<StackData> {
  let mut stack_data = Vec::new();

  for stack in stacks {
    let name = prepare_name(&stack);
    let ins = fetch_instances(opsworks_client, &stack);
    let instances = parse_instances(ins);
    let endpoint = match find_endpoint(&instances) {
      Some(endpoint) => endpoint.clone(),
      None => continue
    };

    stack_data.push(StackData {
      name:      name,
      endpoint:  endpoint,
      instances: instances
    });
  }

  stack_data
}

fn fetch_instances(opsworks_client: &OpsWorksClient, stack: &Stack) -> Vec<Instance> {
  let instances_request = &DescribeInstancesRequest {
    stack_id:     stack.stack_id.clone(),
    instance_ids: None,
    layer_id:     None
  };

  match opsworks_client.describe_instances(instances_request).sync() {
    Ok(instance_result) =>
      match instance_result.instances {
        Some(instances) =>
          instances
            .into_iter()
            .filter(|instance| {
              match instance.status {
                Some(ref status) => status == "online",
                None => false
              }
            })
            .collect(),
        None => panic!("{}", "Zero instances")
      },
    Err(error) => panic!("{:?}", error)
  }
}

fn parse_instances(instances: Vec<Instance>) -> Vec<InstanceData> {
  instances
    .into_iter()
    .map(|instance| {
      InstanceData {
        hostname:     instance.hostname.unwrap(),
        ssh_key_name: instance.ssh_key_name.unwrap(),
        elastic_ip:   instance.elastic_ip
      }
    })
    .collect()
}

fn find_endpoint(instances: &Vec<InstanceData>) -> Option<&InstanceData> {
  instances.into_iter().find(|instance| instance.elastic_ip.is_some())
}

fn write_data(stack_data: Vec<StackData>) {
  let path_to_wssh = home_dir().join(".wssh");
  match File::create(path_to_wssh) {
    Ok(mut file_wssh) =>
      for data in stack_data {
        match file_wssh.write_all(data.to_vec_string().join("").as_bytes()) {
          Ok(_result) => (),
          Err(error) => panic!("{:?}", error)
        }
      },
    Err(err) => panic!("{}", err)
  }
}
