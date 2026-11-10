use bisect_tui::session::Session;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (session, status) = match args[0].as_str() {
        "read" => match args.get(1) {
            None => (Session::default(), "defaulted-absent"),
            Some(path) => match toml::from_str::<Session>(&std::fs::read_to_string(path)?) {
                Ok(s) => (s, "parsed"),
                Err(_) => (Session::default(), "defaulted-invalid"),
            },
        },
        "export" => {
            let mut value: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&args[1])?)?;
            value["seats_per_district"] = serde_json::json!(value["seats_per_district"]
                .as_str()
                .ok_or("seats string")?
                .parse::<u64>()?);
            (serde_json::from_value(value)?, "exported")
        }
        _ => return Err("read [source] or export settings-json".into()),
    };
    let text = toml::to_string_pretty(&session)?;
    let mut settings = serde_json::to_value(&session)?;
    settings["seats_per_district"] = serde_json::json!(session.seats_per_district.to_string());
    println!(
        "{}",
        serde_json::json!({"status":status,"settings":settings,"native_toml":text})
    );
    Ok(())
}
