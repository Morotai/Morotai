use std::env;
mod create_artifacts;
use std::path::PathBuf;
mod get_ini_section;
mod get_island;
mod mk_ini_manager;
mod sign_config_ini;
use std::process::Command;
extern crate r#time;
// 4
fn main() {
    let mut args_count = 0;
    let mut cluster_name = "";
    let mut working_dir = "";
    let args: Vec<String> = env::args().skip(1).collect();
    match args.len() {
        // no arguments passed
        0 => {
            help();
        }
        // one argument passed
        1 => {
            cluster_name = &args[0];
            // print!("Provided cluster name: {} and size: {}", &args[0], args.len());

            if cluster_name.len() < 3 {
                help();
            } else {
                args_count = 1;
            }
        }
        // two arguments passed
        2 => {
            cluster_name = &args[0];
            working_dir = &args[1];
            args_count = 2;
            // parse the command
        }
        // all the other cases
        _ => {
            // show a help message
            help();
        }
    }

    if args_count == 1 {
        create_artifacts::run(cluster_name, None).unwrap();
    } else {
        // create required folders and files
        create_artifacts::run(cluster_name, Some(&PathBuf::from(working_dir.to_string()))).unwrap();
    }
    // run sd change after all files have been created, updated and added
    let sd_cmd = Command::new("sd")
        .arg("change")
        .output()
        .expect("failed to execute sd change command.");

    let my_result = String::from_utf8_lossy(&sd_cmd.stdout);
    // let mut sd_cmd_result = Vec::new();
    let sd_cmd_result: Vec<&str> = my_result.split_whitespace().collect();
    let change_list_no = sd_cmd_result[1];
    println!("Changelist Number: {:?}", sd_cmd_result[1]);
    println!(
        "Run this sign command: apsigntool.exe -a --2p -d ame -c {}",
        change_list_no
    );
}

fn help() {
    println!(
        "usage:
[Cluster_Name] is a required parameter.\n[Working_Dir] folder is optional.
    At least ONE cluster name is required.
A comma separated list of cluster names is also supported.
    i.e: PFGold_CertUtil cluster1,cluster2,cluster3 or \nPFGold_CertUtil cluster1,cluster2,cluster3 c:\\working_dir "
    );
    panic!("Please run again from your PFGold enlistment.");
}
