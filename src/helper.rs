use regex;
use rusoto_opsworks::Stack;
use std::{env, path::PathBuf};

pub fn prepare_name(stack: &Stack) -> String {
  let name = match stack.name {
    Some(ref name) => name,
    None => panic!("{:?}", "Not name")
  };

  let re1 = regex::Regex::new(r"\(|\)").unwrap();
  let re2 = regex::Regex::new(r"\s{1,}").unwrap();
  let re3 = regex::Regex::new(r"-{1,}").unwrap();
  let re4 = regex::Regex::new(r":{1,}").unwrap();

  let result = re1.replace_all(name, "");
  let result = re2.replace_all(&result, "-");
  let result = re3.replace_all(&result, "-");
  let result = re4.replace_all(&result, "");

  result.to_string().to_lowercase()
}

pub fn home_dir() -> PathBuf {
  match env::home_dir() {
    Some(path) => path,
    None => panic!("{:?}", "home_dir empty")
  }
}
