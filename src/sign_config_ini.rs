use regex::Regex;
#[warn(unused_must_use)]
use std::error::*;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn update_sign_config(
    cluster_name: &str,
    ve_root_val: &str,
    sign_config_ini: &str,
) -> Result<String, Box<dyn Error>> {
    // Get content from INI file
    let contents: String = std::fs::read_to_string(sign_config_ini).unwrap();
// 6
    // Get previous cluster name
    let get_groups = Regex::new(r"(^\D+)(\d+)(\D)").unwrap();
    let mut old_cluster: String = "".to_string();
    let mut cluster_prefix: &str = "";
    let mut new_prefix: String = "".to_string();
    let mut new_contents = "".to_string();
    let mut lines = Vec::new();
    let mut found_home: bool = false;

    // Get Singingconfig.ini ready for update
    let mut child = Command::new("sd")
        .arg("edit")
        .arg(sign_config_ini)
        .spawn()
        .expect("unable to add file to with sd add.");

    // wait for the sd edit command to finish
    let _result = child.wait().unwrap();

    println!("MasterKeys.ini Sd Edit CMD Result: {}", _result);

    let contents = std::fs::read_to_string(sign_config_ini).unwrap();
    match get_groups.captures(cluster_name) {
        Some(cap) => {
            let g1 = cap.get(1).unwrap();
            let g2 = cap.get(2).unwrap();
            let g3 = cap.get(3).unwrap();
            cluster_prefix = &g1.as_str();

            let my_int: i32 = g2.as_str().parse().unwrap();
            let n = my_int - 1;
            if my_int == 01 {
                old_cluster = format!("{}00{}", g1.as_str(), g3.as_str());
            } else if my_int == 0 {
                new_prefix = format!("{}00", g1.as_str());
                println!("Updated prefix: {}", &new_prefix);
                cluster_prefix = &new_prefix;
                old_cluster = format!("{}00{}", g1.as_str(), g3.as_str());
            } else if g2.as_str().contains("0") {
                new_prefix = format!("{}0", g1.as_str());
                println!("Updated prefix: {}", &new_prefix);
                cluster_prefix = &new_prefix;
                old_cluster = format!("{}0{}{}", g1.as_str(), n, g3.as_str());
            } else if g2.as_str().len() == 1 {
                new_prefix = format!("{}{}", g1.as_str(), n);
                println!("Updated prefix: {}", &new_prefix);
                cluster_prefix = &new_prefix;
                old_cluster = format!("{}{}{}", g1.as_str(), n, g3.as_str());
            } else if my_int > 01 {
                old_cluster = format!("{}{}{}", g1.as_str(), n, g3.as_str());
            } else {
                old_cluster = "None".to_string();
            }
        }
        None => println!("Unable to determine previous cluster name."),
    };

    println!("Previous Cluster Name: {}", old_cluster);
    println!("Cluster Prefix: {}", cluster_prefix);

    println!("VE ROOT VALUE: {}", ve_root_val);

    let found = contents.contains(ve_root_val);
    println!("Was VE Root Found?. {}", found);

    if found == false {
        for line in contents.lines() {
            if line.starts_with("VE-ROOT/Autopilot") && line.contains(&old_cluster) {
                if found_home == false {
                    println!("CURRENT VE LINE: {}", line);

                    lines.push(format!("{}\n{}", line, &ve_root_val));
                    found_home = true;
                } else {
                    lines.push(line.to_owned());
                }
            } else {
                lines.push(line.to_owned());
            }
        }

        if found_home == false {
            let new_str = format!("\r\n{}", ve_root_val);

            println!("No match found. Appending to the bottom.");
            new_contents = lines.join("\r\n");
            new_contents.push_str(&new_str);
        } else {
            new_contents = lines.join("\r\n");
        }

        // TODO
        // 1. Need to handle the scenario when the cluster prefix or cluster name is NOT found
        // In this scenario, just add to the current line
        // 2. Need to return line_key_no and cluster_donor values from this function

        std::fs::write(sign_config_ini, new_contents)?;
    }

    return Ok("all done".to_string());
}
