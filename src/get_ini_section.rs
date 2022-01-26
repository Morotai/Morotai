pub fn find_ini_section(cluster_island: &str) -> &'static str {
    match run(cluster_island) {
        Some(section) => section,
        None => panic!("Unable to determine INI Section."),
    }
}
// 2
fn run(cluster_island: &str) -> Option<&'static str> {
    match cluster_island {
        "PublicPilotfish" => Some("PilotfishAll"),
        "FairFax" => Some("FairfaxAll"),
        "FairFaxDod" => Some("FairfaxDodAll"),
        "USNat" => Some("USNatAll"),
        "USSec" => Some("USSecAll"),
        "Mooncake" => Some("MooncakeAll"),
        _ => None,
    }
}
