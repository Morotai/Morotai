use std::vec;

pub fn find_island(cluster_name: String) -> String {
    let island = run(cluster_name.to_string());

    match island {
        Some(c) => c.to_string(),
        None => panic!("Unable to find cluster island"),
    }
}

fn run(cluster_name: String) -> Option<Archipelago> {
    let last_char = cluster_name.to_uppercase().chars().last().unwrap();

    // If cluster_name ends in P (most of them do), return PublicPilotfish
    // If cluster_name ends in N, iterate through the different categories to see where it belongs
    match last_char {
        'P' | 'p' => Some(Archipelago::PublicPilotfish),
        _ if publicpf_fn(&cluster_name) => Some(Archipelago::PublicPilotfish),
        'N' => find_nat_cloud_island(&cluster_name),
        _ if xbox_pf(&cluster_name) => Some(Archipelago::Xbox),
        _ if ap_classic(&cluster_name) => Some(Archipelago::ApClassic),
        _ => None,
    }
}

// PublicPilotfish Island
fn publicpf_fn(cluster_name: &str) -> bool {
    let publicpf_clusters = vec![
        "STG01S", "STG02S", "STG03S", "BNZ02C", "CBN03C", "ZZZ00Z", "BN4Q",
    ];
    if publicpf_clusters.iter().any(|&i| i == cluster_name) {
        return true;
    } else {
        return false;
    }
}

// Xbox Island
fn xbox_pf(cluster_name: &str) -> bool {
    let xbox_clusters = vec![
        "BN1Lab", "BN1Prod", "CO1Prod", "CO1bProd", "CY1Prod", "CY1PPE",
    ];
    if xbox_clusters.iter().any(|&i| i == cluster_name) {
        return true;
    } else {
        return false;
    }
}

// ApClassic Island - 2022
// All ApClassic clusters as of 6/23/2020. No new clusters will be created in ApClassic.
fn ap_classic(cluster_name: &str) -> bool {
    let ap_classic_clusters = vec![
        "BJ1",
        "Bn1",
        "Bn2",
        "BN2B",
        "CH01",
        "Ch1",
        "Ch1b",
        "Ch1d",
        "Co3",
        "Co3b",
        "co3c",
        "CO4",
        "CO4C",
        "cy2",
        "CY2B",
        "CY4",
        "Db3",
        "Db4",
        "db5",
        "DM3",
        "HK2",
        "MW1",
        "Sg1",
        "Sn2",
        "MWH01",
        "HKG01",
        "DUB01",
        "BN01",
        "CHI02",
        "CO01",
        "DUB02",
        "BJS01",
        "CY2Test01",
        "CY2Test02",
        "CY2Test03",
        "BLDEV01",
        "BNZE01",
        "CHIE01",
        "CYSE01",
        "DSME01",
        "DUBE01",
        "HKGE01",
        "MWHE01",
        "PUSE01",
        "SATE01",
        "BN4C",
        "CO1C",
        "DM2C",
        "ApSingleBox",
    ];
    if ap_classic_clusters.iter().any(|&i| i == cluster_name) {
        return true;
    } else {
        return false;
    }
}

fn find_nat_cloud_island(val: &str) -> Option<Archipelago> {
    match val {
        v if v.ends_with("FGN") => Some(Archipelago::FairFax),
        "BN1N" | "SN5N" | "CY01N" | "PHX01N" | "DM2N" | "BGZ01N" | "BNZ02N" => {
            Some(Archipelago::FairFax)
        }
        v if v.ends_with("FDN") => Some(Archipelago::FairFaxDod),
        "BN3N" | "DM3N" => Some(Archipelago::FairFaxDod),
        v if v.ends_with("EXN") => Some(Archipelago::USNat),
        "GRN01N" | "RED01N" => Some(Archipelago::USNat),
        v if v.ends_with("RXN") => Some(Archipelago::USSec),
        "MNZ01N" | "SAT01N" | "CYX01N" => Some(Archipelago::USSec),
        v if v.ends_with("BFN") => Some(Archipelago::BlackForest),
        "FR1N" | "LG1N" =>
        //Deprecated by end of 2020.
        {
            Some(Archipelago::BlackForest)
        }
        v if v.ends_with("MCN") => Some(Archipelago::Mooncake),
        "BJ1N" | "SH1N" | "BJS01N" | "SHA01N" => Some(Archipelago::Mooncake),
        _ => return None,
    }
}

// National Cloud new naming convention follows the pattern aaannXYZ.
// The old naming convention won't change anymore and is hardcoded.
// List of Islands
#[derive(Debug, Eq, PartialEq)]
pub enum Archipelago {
    PublicPilotfish,
    Xbox,
    ApClassic,
    FairFax,
    FairFaxDod,
    USNat,
    USSec,
    BlackForest,
    Mooncake,
    None,
}

impl ToString for Archipelago {
    fn to_string(&self) -> String {
        match self {
            Archipelago::PublicPilotfish => "PublicPilotfish".to_string(),
            Archipelago::Xbox => "Xbox".to_string(),
            Archipelago::ApClassic => "ApClassic".to_string(),
            Archipelago::FairFax => "FairFax".to_string(),
            Archipelago::FairFaxDod => "FairFaxDod".to_string(),
            Archipelago::USNat => "USNat".to_string(),
            Archipelago::USSec => "USSec".to_string(),
            Archipelago::BlackForest => "BlackForest".to_string(),
            Archipelago::Mooncake => "Mooncake".to_string(),
            Archipelago::None => "No Island Found.".to_string(),
        }
    }
}
