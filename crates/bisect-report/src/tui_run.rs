use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Form {
    pub location: String,
    pub chamber: String,
    pub year: String,
    pub resolution: String,
    pub seed: String,
    pub label: String,
    pub version: String,
    pub balance_tol: String,
}
pub fn state_arguments(form: &Form) -> Vec<String> {
    let mut args = vec![
        "state".into(),
        "--state".into(),
        form.location.clone(),
        "--chamber".into(),
        form.chamber.clone(),
        "--year".into(),
        form.year.clone(),
        "--resolution".into(),
        form.resolution.clone(),
        "--version".into(),
        form.version.clone(),
        "--force".into(),
    ];
    for (flag, value) in [
        ("--seed", &form.seed),
        ("--label", &form.label),
        ("--balance-tolerance", &form.balance_tol),
    ] {
        if !value.is_empty() {
            args.push(flag.into());
            args.push(value.clone());
        }
    }
    args
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_flags_and_optional_values() {
        let f = Form {
            location: "RI".into(),
            chamber: "house".into(),
            year: "2020".into(),
            resolution: "tract".into(),
            version: "v1".into(),
            seed: "18446744073709551615".into(),
            label: "".into(),
            balance_tol: "5.0".into(),
        };
        let a = state_arguments(&f);
        assert!(a
            .windows(2)
            .any(|p| p == ["--seed", "18446744073709551615"]));
        assert!(!a.contains(&"--label".into()));
        assert!(a.windows(2).any(|p| p == ["--balance-tolerance", "5.0"]));
    }
}
