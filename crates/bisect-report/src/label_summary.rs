//! Native label/year summary rendering, shared by desktop and browser adapters.
//! Preserves the minimal native index-based report; this is not a commission report.

fn html_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

/// Render a minimal but well-formed HTML report for a label/year.
pub fn render_label_html_report(
    label: &str,
    year: &str,
    analysis_summary: &serde_json::Value,
) -> String {
    let label = html_text(label);
    let year = html_text(year);
    let states_section = analysis_summary["states"]
        .as_object()
        .map(|states| {
            let rows: String = states
                .iter()
                .map(|(state, info)| {
                    let status = info["status"].as_str().unwrap_or("unknown");
                    let row_class = if status == "ok" { "ok" } else { "failed" };
                    let state = html_text(state);
                    let status = html_text(status);
                    format!("<tr class=\"{row_class}\"><td>{state}</td><td>{status}</td></tr>\n")
                })
                .collect();
            format!(
                "<table>\n<thead><tr><th>State</th><th>Status</th></tr></thead>\n\
                 <tbody>\n{rows}</tbody>\n</table>"
            )
        })
        .unwrap_or_else(|| "<p>No state analysis data found.</p>".to_string());

    format!(
        "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         <head>\n\
           <meta charset=\"UTF-8\">\n\
           <title>Redistricting Report: {label} ({year})</title>\n\
           <style>\n\
             body {{ font-family: sans-serif; margin: 2em; }}\n\
             table {{ border-collapse: collapse; width: 100%; }}\n\
             th, td {{ border: 1px solid #ccc; padding: 0.4em 0.8em; text-align: left; }}\n\
             tr.ok   td {{ color: green; }}\n\
             tr.failed td {{ color: red; }}\n\
           </style>\n\
         </head>\n\
         <body>\n\
           <h1>Redistricting Report</h1>\n\
           <p><strong>Label:</strong> {label}</p>\n\
           <p><strong>Year:</strong> {year}</p>\n\
           <h2>State Analysis Summary</h2>\n\
           {states_section}\n\
         </body>\n\
         </html>\n"
    )
}

/// Build a JSON report structure for a label/year.
pub fn build_label_json_report(
    label: &str,
    year: &str,
    analysis_summary: &serde_json::Value,
    created: &str,
) -> serde_json::Value {
    serde_json::json!({
        "report_type": "label_analysis",
        "label": label,
        "year": year,
        "created": created,
        "states": analysis_summary["states"],
    })
}

/// Render exactly one native summary file from the optional raw analysis index.
/// Missing index uses the native empty-state fallback; invalid JSON is an error.
/// Dynamic HTML text is escaped for both targets. Browser callers must also use
/// restricted script-free previews and never execute imported HTML content.
pub fn render_label_summary(label: &str, year: &str, analysis_index_json: Option<&str>, format: &str, created: &str) -> Result<(String, String), String> {
    let summary = match analysis_index_json {
        Some(raw) => serde_json::from_str(raw).map_err(|error| format!("[INTERNAL] report: cannot parse analysis index: {error}"))?,
        None => serde_json::json!({"label":label,"year":year,"states":{}}),
    };
    let text = match format {
        "html" => render_label_html_report(label, year, &summary),
        "json" => serde_json::to_string_pretty(&build_label_json_report(label, year, &summary, created)).map_err(|error| format!("[INTERNAL] report: serialise failed: {error}"))?,
        other => return Err(format!("[CONFIG] report: unsupported format '{other}'. Use 'html' or 'json'.")),
    };
    Ok((format!("{label}_{year}_report.{format}"), text))
}

#[cfg(test)]
mod tests {
    use super::*;
    const INDEX: &str = include_str!("../tests/fixtures/label-summary/analysis-index.json");

    #[test]
    fn html_matches_original_desktop_bytes() {
        let (name, text) = render_label_summary("summary_fixture", "2020", Some(INDEX), "html", "unused").unwrap();
        assert_eq!(name, "summary_fixture_2020_report.html");
        assert_eq!(text, include_str!("../tests/fixtures/label-summary/html.html"));
    }

    #[test]
    fn json_matches_original_desktop_bytes_with_explicit_time() {
        let expected = include_str!("../tests/fixtures/label-summary/json.json");
        let original: serde_json::Value = serde_json::from_str(expected).unwrap();
        let (name, text) = render_label_summary("summary_fixture", "2020", Some(INDEX), "json", original["created"].as_str().unwrap()).unwrap();
        assert_eq!(name, "summary_fixture_2020_report.json");
        assert_eq!(text, expected);
    }

    #[test]
    fn missing_index_retains_original_empty_state_html() {
        let (_, text) = render_label_summary("summary_fixture", "2020", None, "html", "unused").unwrap();
        assert_eq!(text, include_str!("../tests/fixtures/label-summary/missing-index.html"));
    }

    #[test]
    fn malformed_index_precedes_unsupported_format() {
        let error = render_label_summary("fixture", "2020", Some("{"), "pdf", "unused").unwrap_err();
        assert!(error.starts_with("[INTERNAL] report: cannot parse analysis index:"));
        let error = render_label_summary("fixture", "2020", None, "pdf", "unused").unwrap_err();
        assert_eq!(error, "[CONFIG] report: unsupported format 'pdf'. Use 'html' or 'json'.");
    }

    #[test]
    fn arbitrary_native_json_context_is_not_replaced_by_typed_state_projection() {
        let raw = r#"{"states":[{"votes":18446744073709551615,"note":"original"}]}"#;
        let (_, text) = render_label_summary("fixture", "2020", Some(raw), "json", "recorded").unwrap();
        assert!(text.contains("18446744073709551615"));
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["states"][0]["votes"].as_u64(), Some(u64::MAX));
        let (_, html) = render_label_summary("fixture", "2020", Some(raw), "html", "unused").unwrap();
        assert!(html.contains("No state analysis data found."));
    }

    #[test]
    fn supplied_html_characters_remain_text_in_every_interpolation() {
        let source = serde_json::json!({"states":{"</td><script>evil()</script>":{"status":"<img src=x onerror=evil()>&\"'"}}});
        let html = render_label_html_report("<label>&\"'", "<year>", &source);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img "));
        assert!(html.contains("&lt;/td&gt;&lt;script&gt;evil()&lt;/script&gt;"));
        assert!(html.contains("&lt;img src=x onerror=evil()&gt;&amp;&quot;&#39;"));
        assert!(html.contains("&lt;label&gt;&amp;&quot;&#39;"));
        assert!(html.contains("&lt;year&gt;"));
    }
}
