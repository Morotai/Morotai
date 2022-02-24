use std::env;
mod create_artifacts;
use std::fs::File;
use std::io::{Write, Read};
use std::path::PathBuf;
mod get_ini_section;
mod get_island;
mod mk_ini_manager;
mod sign_config_ini;
use std::process::Command;
extern crate r#time;

fn main() {
    let mut args_count = 0;
    let mut cluster_name = "";
    let mut working_dir = "";
    let args: Vec<String> = env::args().skip(1).collect();
    let mut cluster_status_file = "".to_string();
    let mut clusters = Vec::new();
    let mut status_files_tracker = Vec::new();
    let mut cluster_count = 0;

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
        cluster_status_file = create_artifacts::run(cluster_name, None, 1).unwrap();
        status_files_tracker.push(cluster_status_file);

    } else {

        if cluster_name.contains(",") {
            clusters = cluster_name.trim().split(",").collect();
            cluster_count = clusters.len();
        } else {
            clusters.push(cluster_name.trim());
            cluster_count = clusters.len();
            // println!("Single Cluster Vector Length: {:?}", clusters.len());
        }

        for cluster_name in clusters {
            println!("Total Clusters: {:?}", &cluster_count);
            // create required folders and files
         let status_file =  create_artifacts::run(cluster_name, Some(&PathBuf::from(working_dir.to_string())), cluster_count).unwrap();
        //   println!("STATUS FILE FROM MAIN.RS: {cluster_status_file}");
          status_files_tracker.push(status_file);
         // Reduce cluster total by 1
         cluster_count -= 1;
        }

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
    let ap_sign_tool_cmd = format!("apsigntool -a --2p -d ame -c {}", change_list_no);

    for cluster_file in status_files_tracker {

                    // Add apsigntool cmd to status file
                    let contents = match read_file(&cluster_file) {
                        Ok(contents) => contents,
                        Err(e) => panic!("{}", e),
                    };
                    // println!("ORGINAL CONTENT: {:#?}", contents);
        
                    let final_content = contents.replace("AP_SIGN_CMD", &ap_sign_tool_cmd);
                    // println!("FINAL CONTENT: {:#?}", final_content);
        
                    match write_file(&cluster_file, final_content) {
                        Err(e) => panic!("{}", e),
                        _ => (),
                    };

            // Open new cluster.txt file using Notepad
            Command::new("notepad")
            .arg(cluster_file)
            .spawn()
            .expect("failed to open cluster file.");
    }


    // println!("Changelist Number: {:?}", sd_cmd_result[1]);
    println!(
        "Run this sign command: apsigntool -a --2p -d ame -c {}",
        change_list_no
    );
}

fn read_file(file: &str) -> std::io::Result<String> {
    let mut file = File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn write_file(file: &str, contents: String) -> std::io::Result<()> {
    let mut file = File::create(file)?;
    file.write(contents.as_bytes())?;
    Ok(())
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
