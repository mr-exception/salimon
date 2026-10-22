//! Rebuild/audit the immutable U11 publication. stdout is canonical manifest JSON.
use salimon_universe::starting_system;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let publication = starting_system::candidate_publication()?;
    let start = starting_system::freeze(&publication)?;
    match std::env::args().nth(1).as_deref() {
        None => println!("{}", String::from_utf8(start.encode()?)?),
        Some("--audit") => println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "candidate_publication_sha256": publication.sha256()?,
                "world_definition": start.definition(),
                "assessments": starting_system::assess(&publication)?,
                "manifest": start.manifest(),
            }))?
        ),
        Some(_) => return Err("usage: starting_system [--audit]".into()),
    }
    Ok(())
}
