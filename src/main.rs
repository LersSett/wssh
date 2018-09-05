extern crate clap;
extern crate regex;
extern crate rusoto_core;
extern crate rusoto_opsworks;

mod cli;
mod helper;
mod update;

use std::{
  fs::File,
  io::{BufRead, BufReader},
  process::{exit, Command}
};

use update::StackData;

fn main() {
  let matches = cli::args();
  let port: &str;

  if matches.is_present("force") {
    update::stacks_data()
  }

  port = matches.value_of("port").unwrap_or("22");

  let data = parse_file();
  match matches.value_of("STACK_NAME") {
    Some(stack_name) => {
      let stack_data = data.iter().find(|stack_data| stack_data.name == stack_name).unwrap();

      match matches.value_of("INSTANCE_NAME") {
        Some(instance_name) => {
          let instance = stack_data
            .instances
            .iter()
            .find(|instance| instance.hostname == instance_name)
            .unwrap();
          let user = matches.value_of("user").unwrap_or("ubuntu");
          let key_path = matches
            .value_of("key")
            .unwrap_or(
              helper::home_dir()
                .join(".ssh")
                .join(format!("{}.pem", instance.ssh_key_name.clone()))
                .to_str()
                .unwrap()
            ).to_string();

          let proxy_command = format!(
            "ProxyCommand=ssh -i {key_path} -p {port} -W %h:%p {user}@{elastic_ip}",
            key_path = key_path,
            user = user,
            elastic_ip = stack_data.clone().endpoint.elastic_ip.unwrap(),
            port = port
          );

          let result = Command::new("ssh")
            .arg("-o")
            .arg(proxy_command)
            .arg("-i")
            .arg(key_path)
            .arg("-p")
            .arg(port)
            .arg(format!("{user}@{hostname}", user = user, hostname = instance.hostname))
            .spawn();

          match result {
            Ok(child) => child.wait_with_output().expect("failed to wait on child"),
            Err(error) => panic!("{:?}", error)
          };
        },
        None => panic!("{:?}", "Instance name is required")
      }
    },

    None => exit(0)
  }
}

fn parse_file() -> Vec<StackData> {
  let path_to_wssh = helper::home_dir().join(".wssh");
  match File::open(path_to_wssh) {
    Ok(f) => {
      let mut file = BufReader::new(&f);

      let mut stack_datas: Vec<StackData> = Vec::new();
      for line in file.lines() {
        let l = line.unwrap();
        let vec = l.split_whitespace().collect::<Vec<&str>>();

        match stack_datas
          .clone()
          .into_iter()
          .enumerate()
          .find(|&(_, ref sd)| sd.name == vec[0])
        {
          Some((index, mut stack_data)) =>
            if vec[1].to_string() != "None" {
              let instance = update::InstanceData {
                hostname:     vec[2].to_string(),
                elastic_ip:   Some(vec[1].to_string()),
                ssh_key_name: vec[3].to_string()
              };
              stack_data.instances.push(instance.clone());
              stack_data.endpoint = instance;
              stack_datas[index] = stack_data;
            } else {
              let instance = update::InstanceData {
                hostname:     vec[2].to_string(),
                elastic_ip:   Some(vec[1].to_string()),
                ssh_key_name: vec[3].to_string()
              };
              stack_data.instances.push(instance);
              stack_datas[index] = stack_data;
            },
          None => {
            let instance = update::InstanceData {
              hostname:     vec[2].to_string(),
              elastic_ip:   Some(vec[1].to_string()),
              ssh_key_name: vec[3].to_string()
            };

            let mut instances = Vec::new();
            instances.push(instance.clone());

            let sd = StackData {
              name:      vec[0].to_string(),
              endpoint:  instance.clone(),
              instances: instances
            };
            stack_datas.push(sd);
          }
        }
      }

      stack_datas
    },
    Err(error) => panic!("File not open: {}", error)
  }
}
