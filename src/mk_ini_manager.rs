use anyhow::bail;
use regex::Regex;
#[allow(dead_code)]
#[warn(unused_must_use)]
use std::process::Command;
use std::str::FromStr;
use std::{
    path::{Path, PathBuf},
    usize,
};

pub fn add_cluster_to_key_ring(
    cluster_name: &str, // "xyz123"
    ini_section: &str,  // "server_list_"
    master_keys_ini_file: &str,
    cluster_count: usize,
    pfgoldpath: &str,
) -> Result<(String, String), anyhow::Error> {
    let max_clusters_per_line = 5;

    // Change directory to root of PFGold enlistment
    let root = Path::new(&pfgoldpath);
    println!("Changed to working directory: {:?}", root);
    // Set MasterKeys.ini file ready for editing
    let mut child = Command::new("sd")
        .arg("edit")
        .arg(master_keys_ini_file)
        .spawn()
        .expect("unable to add file to with sd add.");
    let _result = child.wait().unwrap();
    println!("MasterKeys.ini is ready for editing: {}", _result);

    // INI Section update: i.e: PilotfishAll
    let contents = std::fs::read_to_string(master_keys_ini_file).unwrap();
    println!("INI SEC: {}", ini_section);
    let mut lines = Vec::new();
    for line in contents.lines() {
        // Add cluster to key set based on ini section: i.e: PilotfishAll
        if line.starts_with(ini_section) {
            if !line
                .split("=")
                .nth(1)
                .unwrap()
                .to_string()
                .contains(cluster_name)
            {
                // cluster name not found on current line. Add it.
                lines.push(format!("{},{}", line, cluster_name));
            } else {
                println!("{} already exists in: {}", cluster_name, ini_section);
                lines.push(line.to_owned());
            }
        } else {
            lines.push(line.to_owned());
        }
    }

    let new_content = lines.join("\r\n");
    let lines_in: Vec<String> = new_content.lines().map(|s| s.to_owned()).collect();

    // let (set_key_ring_content, key_ring) =
    //     add_server(cluster_name, "Autopilot_Internal_Pilotfish_", lines_in).unwrap_or_default();

    // println!("Key Ring: {}", key_ring);

    // let final_content: String = set_key_ring_content.join("\r\n");

    #[derive(Debug, PartialEq)]
    enum State {
        Initial,
        SeenSomeMatch,
        Done,
    }

    let mut state = State::Initial;
    let mut last_matching_group_out_line_index: usize = 0;
    let mut highest_seen_matching_group_number: usize = 0;
    let mut opt_added_to_group: Option<String> = None;
    let mut lines_out: Vec<String> = Vec::new();
    let mut cluster_donor: String = "".to_string();
    let mut found_home: bool = false;

    // Determine group_prefix based on INI Section i.e: PilotfishAll
    let group_prefix = match &ini_section {
        &"PilotfishAll" => "Autopilot_Internal_Pilotfish_",
        &"BlackForestAll" => "Autopilot_Internal_BlackForest_",
        &"MooncakeAll" => "Autopilot_Internal_Mooncake_",
        &"FairfaxAll" => "Autopilot_Internal_Fairfax_",
        &"FairfaxDodAll" => "Autopilot_Internal_FairfaxDod_",
        &"USNatAll" => "Autopilot_Internal_USNat_",
        &"USSecAll" => "Autopilot_Internal_USSec_",
        _ => "None",
    };

    let rx_str = format!(
        r"^\s*({}(\d+))\s*=\s*(.*?)\s*$",
        regex::escape(group_prefix)
    );
    //println!("Using rx_str: {:?}", rx_str);
    let re = Regex::new(&rx_str).unwrap();

    // Iterate over input lines and EOF (None).
    let lines_in_iter = lines_in
        .iter()
        .map(|s| Some(s))
        .chain(std::iter::once(None));

    for opt_line in lines_in_iter {
        let (is_eod, line) = if let Some(line) = opt_line {
            //println!("[{:?}] '{:?}'", state, line);
            (false, line.to_owned())
        } else {
            //println!("[{:?}] EOD", state);
            (true, String::new())
        };

        // See if line is blank or prefix matches. EOD lines don't count as blank.
        let is_blank = !is_eod && line.trim().is_empty();

        // See if the line matches the group prefix we're expecting.
        let opt_prefix_captures = if state != State::Done && !is_blank && !is_eod {
            re.captures(&line)
        } else {
            None
        };

        // If the line matches, read its index number
        if let Some(ref caps) = opt_prefix_captures {
            let group_number = usize::from_str(&caps[2]).unwrap();
            if highest_seen_matching_group_number < group_number {
                highest_seen_matching_group_number = group_number;
            }
        }

        //if let Some(ref caps) = opt_prefix_captures {
        //println!("[matches] {:?}, {:?}, {:?}", &caps[1], &caps[2], &caps[3]);
        //}

        'reprocess_line_in_new_state: loop {
            let mut reprocess_line = false;

            match state {
                State::Initial => {
                    if opt_prefix_captures.is_some() {
                        state = State::SeenSomeMatch;
                        reprocess_line = true;
                    } else {
                        lines_out.push(line.clone());
                    }
                }
                State::SeenSomeMatch => {
                    if let Some(ref caps) = opt_prefix_captures {
                        // Split up the comma-separated list of clusters
                        let clusters: Vec<String> = caps[3]
                            .to_string()
                            .split(",")
                            .map(|s| s.trim().to_owned())
                            .filter(|s| !s.is_empty())
                            .collect();
                        // println!("clusters: {:?}", clusters);
                        found_home = clusters.contains(&cluster_name.to_string());
                        // println!("Found home? {} - Adding to current line.", found_home);
                        // If we have room in current keyring, just add the cluster
                        if clusters.len() < max_clusters_per_line
                            && found_home == false
                            && clusters.len() + cluster_count <= max_clusters_per_line
                        {
                            let new_clusters = clusters.clone();
                            cluster_donor = new_clusters[0].to_string();
                            println!("Cluster Donor - Existing Keyring: {}", &cluster_donor);
                            opt_added_to_group = Some(caps[1].to_owned());
                            let revised_line = line.trim_end().to_owned() + "," + cluster_name;
                            lines_out.push(revised_line);
                            state = State::Done;
                        } else {
                            // We have to skip lines until we find where to append a new group
                            // But we don't know this until we see a non-blank non-matching line
                            // or EOD.
                            last_matching_group_out_line_index = lines_out.len();
                            lines_out.push(line.clone());
                        }
                    } else if is_blank {
                        lines_out.push(line.clone());
                    } else {
                        // We've hit a non-blank line (or EOD) that doesn't match
                        if !is_eod {
                            lines_out.push(line.clone());
                        }

                        // ADD NEW KEY_RING SECTION
                        if found_home == false {
                             cluster_donor = match &group_prefix {
                                &"Autopilot_Internal_Pilotfish_" => "DM2P".to_string(),
                                &"Autopilot_Internal_BlackForest_" => "FR1N".to_string(),
                                &"Autopilot_Internal_Mooncake_" => "BJ1N".to_string(),
                                &"Autopilot_Internal_Fairfax_" => "BN1N".to_string(),
                                &"Autopilot_Internal_FairfaxDod_" => "DM3N".to_string(),
                                &"Autopilot_Internal_USNat_" => "GRN01N".to_string(),
                                &"Autopilot_Internal_USSec_" => "SAT01N".to_string(),
                                _ => "None".to_string(),
                            };

                            let new_group_number = highest_seen_matching_group_number + 1;
                            let new_group = format!("{}{}", group_prefix, new_group_number);
                            opt_added_to_group = Some(new_group.clone());
                            let new_line =
                                format!("{}={},{}", new_group, cluster_donor, cluster_name);
                            lines_out
                                .insert(last_matching_group_out_line_index + 1, new_line.clone());

                            println!("Added New Key Ring - Cluster Donor: {}", cluster_donor);
                            state = State::Done;
                        }
                    }
                }
                State::Done => {
                    // Just copy remaining lines
                    if !is_eod {
                        lines_out.push(line.clone());
                    }
                }
            } // match state

            if !reprocess_line {
                break 'reprocess_line_in_new_state;
            }
        } // 'reprocess_line_in_new_state
    }

    let final_content: String = lines_out.join("\r\n");
    // println!("Lines_Out Size: {}", final_content.len());
    std::fs::write(master_keys_ini_file, final_content)?;
    // Update INI File

    Ok((match opt_added_to_group {
        Some(group_name) => group_name,
        None => {
            if found_home == true {
                println!("Cluster found. Exiting.");
            } else {
                // We couldn't add a new group line because we couldn't infer where in the
                // file it should be inserted.
                println!("No lines matching prefix found.");
            }
            // return "nothing to see here".to_string();
            bail!("No lines matching prefix found");
        }
    }, cluster_donor.to_string()))
}
