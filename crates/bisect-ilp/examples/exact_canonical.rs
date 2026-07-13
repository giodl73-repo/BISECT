use bisect_ilp::{
    solve_exact_canonical_artifacts, verify_exact_canonical_artifacts, ExactCanonicalCertificate,
    ExactCanonicalInstance, ExactProofTranscript,
};
use std::error::Error;
use std::path::Path;

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Box<dyn Error>> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    match args.get(1).and_then(|value| value.to_str()) {
        Some("solve") if args.len() == 4 => {
            let instance: ExactCanonicalInstance = read_json(Path::new(&args[2]))?;
            let out_dir = Path::new(&args[3]);
            let artifacts = solve_exact_canonical_artifacts(&instance)?;
            std::fs::create_dir_all(out_dir)?;
            std::fs::write(
                out_dir.join("exact-canonical-certificate.json"),
                serde_json::to_string_pretty(&artifacts.certificate)?,
            )?;
            std::fs::write(
                out_dir.join("exact-canonical-proof.json"),
                serde_json::to_string_pretty(&artifacts.proof)?,
            )?;
            println!("Exact Canonical solve: PASS");
        }
        Some("verify") if args.len() == 5 => {
            let instance: ExactCanonicalInstance = read_json(Path::new(&args[2]))?;
            let certificate: ExactCanonicalCertificate = read_json(Path::new(&args[3]))?;
            let proof: ExactProofTranscript = read_json(Path::new(&args[4]))?;
            verify_exact_canonical_artifacts(&instance, &certificate, &proof)?;
            println!("Exact Canonical verification: PASS");
        }
        _ => {
            return Err(
                "usage: exact_canonical solve <instance.json> <out-dir> | verify <instance.json> <certificate.json> <proof.json>"
                    .into(),
            );
        }
    }
    Ok(())
}
