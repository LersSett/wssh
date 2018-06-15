use clap::{App, Arg, ArgMatches};

pub fn args() -> ArgMatches<'static> {
  App::new("wssh")
    .version("0.0.1")
    .about("aws opsworks ssh")
    .author("Stanislav Lapata <stanislavlapata@gmail.com>")
    .arg(
      Arg::with_name("force")
        .short("f")
        .long("force")
        .help("Force update stacks information")
    )
    .arg(
      Arg::with_name("user")
        .short("u")
        .long("user")
        .help("SSH user")
        .takes_value(true)
    )
    .arg(
      Arg::with_name("key")
        .short("k")
        .long("key")
        .help("Path to ssh key")
        .takes_value(true)
    )
    .arg(
      Arg::with_name("port")
        .short("p")
        .long("port")
        .help("SSH port")
        .takes_value(true)
    )
    .arg(Arg::with_name("STACK_NAME").help("Stack name").index(1))
    .arg(Arg::with_name("INSTANCE_NAME").help("Instance name").index(2))
    .get_matches()
}
