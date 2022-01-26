#![allow(unused)]
use crate::{get_ini_section, get_island, mk_ini_manager, sign_config_ini};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::ops::RangeBounds;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, usize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
struct Cluster {
    date: String,
    stage: String,
    cluster_name: String,
    cluster_island: String,
    branch_name: String,
    working_dir: PathBuf,
    pfgold_path: String,
    virtual_env_path: String,
    cluster_path: String,
    master_key_file: String,
    sconfig_file: String,
    secstore_file: String,
    pfgold_sign_config: String,
    donor: String,
    ve_root_value: String,
    ap_internal_value: String,
    new_cluster_file: String,
    pki_cmd: String,
}

pub fn run(cluster_name: &str, maybe_cert_path: Option<&Path>) -> Result<(), Error> {
    let mut clusters = Vec::new();
    let mut cluster_count = 0;
    use time::OffsetDateTime;

    if cluster_name.contains(",") {
        clusters = cluster_name.trim().split(",").collect();
        cluster_count = clusters.len();
    } else {
        clusters.push(cluster_name.trim());
        cluster_count = clusters.len();
        // println!("Single Cluster Vector Length: {:?}", clusters.len());
    }

    println!("Multi Cluster Vector Length: {:?}", clusters.len());

    // Get PFGold Root folder and branch to ensure the program is running under pfgold enlistment
    let branch = "_BUILDBRANCH";
    let enlistment_path = "inetroot";


    // set date folder name as Month-day-year (i.e: October-10-2020)
    let date_folder: String = format!("{}-{}-{}", OffsetDateTime::now_utc().month(), OffsetDateTime::now_utc().day(), OffsetDateTime::now_utc().year());

    let mut local_cert_path: PathBuf = match maybe_cert_path {
        Some(path) => path.to_owned(),
        None => {
            // default to path in temp dir
            env::temp_dir()
        }
    };

    // Check if local cert Full path exists and create it if not
    match fs::create_dir(&local_cert_path.clone()) {
        Ok(_) => {
            println!("Created working directory folder: {:?}", local_cert_path);
        }
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                println!("Working directory: {:?}", local_cert_path);
            } else {
                return Err(Error::Io(err));
            }
        }
    }

    // Check if ApCertUtil folder exists - create it if not found.
    let mut ap_cert_util_dir = local_cert_path.clone();

    ap_cert_util_dir.push("ApCertUtil");
    match fs::create_dir(&ap_cert_util_dir.clone()) {
        Ok(_) => {
            println!("Created ApCertUtil folder: {:?}", ap_cert_util_dir);
        }
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                println!("ApCertUtil Folder Already Exists: {:?}", ap_cert_util_dir);
            } else {
                return Err(Error::Io(err));
            }
        }
    }

    // Update local_cert_path with custom folder name: i.e: %working_dir%\ApCertUtil\October-28-2020
    ap_cert_util_dir.push(&date_folder.trim());
    let mut working_folder = ap_cert_util_dir.clone();

    // Check if local cert Full path exists and create it if not
    match fs::create_dir(&working_folder.clone()) {
        Ok(_) => {
            println!("Created working directory folder: {:?}", working_folder);
        }
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                println!("Working directory: {:?}", working_folder);
            } else {
                return Err(Error::Io(err));
            }
        }
    }

    // Get PFGold enlistment folder
    let pfgoldpath = match env::var(enlistment_path) {
        Ok(val1) => val1,
        Err(e) => {
            return Err(Error::Other(format!(
                "Enlistment is not defined in the environment.{}: {}",
                enlistment_path, e,
            )))
        }
    };
    println!("PFGold Path: {}", pfgoldpath);
    // Get enlistment branch
    let pfbranch = match env::var(branch) {
        Ok(val) => val,
        Err(e) => {
            return Err(Error::Other(format!(
                "{} is not defined in the environment. Error: {}",
                branch, e
            )))
        }
    };

    println!("Current Branch: {}", pfbranch);

    let master_keys_file = format!(
        "{}\\data\\Autopilot\\SecretStoreMasterKeys\\masterkeys.ini",
        pfgoldpath
    );
    let pf_sign_config_file = format!(
        "{}\\autopilotservice\\Global\\VirtualEnvironments\\Autopilot\\SigningConfig.ini",
        pfgoldpath
    );

    // Iterate trough all clusters
    for cluster_item in &clusters {
        // set cluster name to upper case
        let cluster = cluster_item.to_uppercase();
        println!("Processing Cluster: {:?}", &cluster);
        let mut ve_path = format!(
            "{}\\autopilotservice\\Global\\VirtualEnvironments\\Autopilot\\ApPki-{}",
            pfgoldpath, cluster
        );
        let sign_config_file = format!("{}\\SigningConfig.ini", ve_path);
        let secret_config_file = format!("{}\\SecretStoreConfig.ini", ve_path);
        let mut ve_root_text = format!("VE-ROOT/Autopilot/ApPki-{}=1", cluster);
        let ap_svc_path = format!("{}\\autopilotservice\\{}", pfgoldpath, cluster);
        println!("CREATE ARTIFACTS: VE ROOT VALUE: {}", ve_root_text);
        // Save cluster settings to cluster_name.json file
        let cluster_status_file = format!("{}\\{}.txt", working_folder.display(), cluster);

        // Create cluster folder in autopilotservice path
        // Create Cluster directories. Skip if already exist
        if Path::new(&ve_path).exists() | Path::new(&ap_svc_path).exists() {
            // if cluster folder exists, exit the application
            panic!("Cluster {} already exists. Skipping...\nCheck for these these folders: \n{}\n Or \n{}\n", cluster, ve_path, ap_svc_path);
        } else {
            new_cluster_foler(ve_path.to_string());
            new_cluster_foler(ap_svc_path.to_string());

            // Create new cluster empty file
            let new_empty_file = format!("{}\\NewCluster.txt", ap_svc_path);
            println!("New cluster empty file: {}", new_empty_file);

            File::create(new_empty_file);

            // get cluster Island
            let cluster_island = get_island::find_island(cluster.to_string());
            println!("Cluster Island: {}", cluster_island);

            let my_ini_section = get_ini_section::find_ini_section(&cluster_island);
            println!("INI Section: {}", my_ini_section);

            // Update MasterKeys.ini file with current cluster
            let (key_set, donor) = mk_ini_manager::add_cluster_to_key_ring(
                &cluster,
                &my_ini_section,
                &master_keys_file,
                cluster_count,
                &pfgoldpath,
            )
            .unwrap_or_default();

            println!("CLUSTER COUNT!!: {}", cluster_count);
            // Reduce cluster total by 1
            cluster_count -= 1;

            println!("Selected Key Ring: {:?}", key_set);
            println!("Donor: {:?}", &donor);

            println!("Cluster status file: {:?}", &cluster_status_file);
            println!("Autopilot Service Path: {}", ap_svc_path);
            println!("VE ROOT Value: {}", ve_root_text);

            let sign_config_result =
                sign_config_ini::update_sign_config(&cluster, &ve_root_text, &pf_sign_config_file);

            match sign_config_result {
                Ok(val) => println!("{}", val),
                Err(e) => println!("{}", e),
            };
            println!("2nd Updated SigningConfig.ini file.");

            // Create SigningConfig.ini file
            new_file(
                String::from("[Metadata]\r\nServiceTreeIds=1bd5fd01-0565-4c8e-affe-ec1b21c96529"),
                &sign_config_file,
            );

            let sec_store_config_value = format!(
                "[KeyAccess]\r\nPrivateKeyApAcl=APMF\\SEC.AutopilotSecurity.{}\r\nMasterKeySet={}\r\n",
                cluster, &key_set
            );

            // Create SecretStoreConfig.ini file
            // let sec_store_config = format!("{}\\SecretStoreConfig.ini", ve_path);
            new_file(String::from(&sec_store_config_value), &secret_config_file);

            // Build PkiCmd string
            let pki_cmd_1 = format!("PkiCmd -c -b -t {} ", cluster);
            let pki_cmd_2 = format!(
                "-p apca.autopilot.{}.ap.gbl -d {} -g ",
                donor, cluster_island
            );
            let pki_cmd_3 = r#"autopilot\ApPki -r "#;
            let pki_cmd_4 = format!(
                "{}\\{}.csr -l {}\\{}.xml -k {}\\APCA-{}_0.key.encr --ame",
                working_folder.to_string_lossy(),
                cluster,
                working_folder.to_string_lossy(),
                cluster,
                working_folder.to_string_lossy(),
                cluster
            );

            let pki_cmd = format!("{}{}{}{}", pki_cmd_1, pki_cmd_2, pki_cmd_3, pki_cmd_4);

            // Struct to keep track of each cluster stage
            let new_cluster = Cluster {
                date: date_folder.to_string(),
                stage: "Stage1".to_string(),
                cluster_name: cluster.to_string(),
                cluster_island: cluster_island.to_string(),
                branch_name: pfbranch.to_string(),
                working_dir: working_folder.to_owned(),
                pfgold_path: pfgoldpath.to_string(),
                virtual_env_path: ve_path.to_string(),
                cluster_path: ap_svc_path.to_string(),
                master_key_file: master_keys_file.to_string(),
                sconfig_file: sign_config_file.to_string(),
                secstore_file: secret_config_file.to_string(),
                pfgold_sign_config: pf_sign_config_file.to_string(),
                donor: donor.to_string(),
                ve_root_value: ve_root_text.to_string(),
                ap_internal_value: key_set.to_string(),
                new_cluster_file: cluster_status_file.to_string(),
                pki_cmd: pki_cmd,
            };
            let cluster_file = cluster_status_file.clone();

            impl fmt::Display for Cluster {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, 
                        "Date: {}\nStage: {}\nCluster_Name: {}\nCluster_Island: {}\nBranch_Name: {}\nWorking_Dir: {}\nPFGold_Path: {}\nVirtual_Env_Path: {}\nCluster_Path: {}\nMaskter_Key_File: {}\nSConfig_File: {}\nSecret_Store: {}\nPFGold_Sign_Config: {}\nDonor: {}\nVE_Root_Value: {}\nAP_Internal_Value: {}\nNew_Cluster_File: {}\nPKI_CMD: {}", 
                    self.date, self.stage, self.cluster_name, self.cluster_island, self.branch_name, self.working_dir.to_string_lossy(), self.pfgold_path, self.virtual_env_path,
                self.cluster_path, self.master_key_file, self.sconfig_file, self.secstore_file, self.pfgold_sign_config, self.donor, self.ve_root_value, self.ap_internal_value, self.new_cluster_file, self.pki_cmd)
                }
            };
            // let serialized_cluster = serde_json::to_string(&new_cluster).unwrap();
            // println!("Cluster Struct content: {:?}", serialized_cluster);
            // create json file to keep track of different stages for a new cert creation process
            // new_file(serialized_cluster.to_string(), &cluster_status_file);
            new_file(new_cluster.to_string(), &cluster_status_file);

            fn new_file(ser_file_content: String, working_dir: &str) -> std::io::Result<()> {
                let mut file = File::create(working_dir)?;
                let ser_file_content: Vec<u8> = ser_file_content.bytes().collect();
                file.write_all(&ser_file_content)?;
                Ok(())
            }
// 1
            // let mut contents = match read_file(&cluster_status_file) {
            //     Ok(contents) => contents,
            //     Err(e) => panic!("{}", e),
            // };
            // println!("ORGINAL CONTENT: {:#?}", contents);

            // Update json file to remove double backslashes from the autopilot group name
            // let final_content = contents.replace("autopilot\\MANOLO", r#"autopilot\ApPki"#);
            // println!("FINAL CONTENT: {:#?}", final_content);

            // match write_file(&cluster_status_file, final_content) {
            //     Err(e) => panic!("{}", e),
            //     _ => (),
            // };
            // change directory to PFGoldRoot
            let root = Path::new(&pfgoldpath);
            // Add artifacts to SD - This requires more MasterKeys.ini parsing
            let mut add_cmd = Command::new("sd")
                .arg("add")
                .arg(sign_config_file)
                .spawn()
                .expect("unable to add file to with sd add.");
            let _result = add_cmd.wait().unwrap();
            println!("Sd add Sign_Config result: {}", _result);

            let mut add_cmd2 = Command::new("sd")
                .arg("add")
                .arg(secret_config_file)
                .spawn()
                .expect("unable to add file to with sd add.");
            let _result2 = add_cmd2.wait().unwrap();
            println!("Sd add secret_config result: {}", _result2);

            // Open new cluster.json file using Notepad
            Command::new("notepad")
                .arg(cluster_file)
                .spawn()
                .expect("failed to open cluster file.");
        }
    }
    Ok(())
}

fn new_cluster_foler(new_cluster: String) {
    match fs::create_dir(&new_cluster) {
        Ok(_) => {
            println!("Created cluster folder: {:?}", new_cluster);
        }
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                println!(
                    "Cluster folder already exists. Skipping.: {:?}",
                    new_cluster
                );
            } else {
                println!(
                    "Unable to create cluster folder. Parent folder may not exist. {}",
                    new_cluster
                );
            }
        }
    }
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

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Other(String),
}
