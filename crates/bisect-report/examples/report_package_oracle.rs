use bisect_report::{
    assemble_report,
    report_assembly::{OPTIONAL_ANALYSIS_FILES, REQUIRED_ANALYSIS_FILES},
    report_package::assemble_report_package,
    PlanManifest, ReportContext,
};
use std::{collections::BTreeMap, path::PathBuf};
fn main() -> anyhow::Result<()> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("output directory required"))?,
    );
    let archive: serde_json::Value = serde_json::from_slice(&std::fs::read(
        std::env::args()
            .nth(2)
            .ok_or_else(|| anyhow::anyhow!("real build archive required"))?,
    )?)?;
    let context = archive["result"]["files"]
        .as_object()
        .and_then(|f| f.iter().find(|(p, _)| p.ends_with(".rctx")))
        .and_then(|(_, v)| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("real RCTX fixture missing"))?;
    std::fs::create_dir_all(&out)?;
    for case in 0..9 {
        let dir = out.join(format!("case-{case}"));
        std::fs::create_dir_all(&dir)?;
        let mut manifest = PlanManifest {
            label: format!("report_{case}"),
            state_code: "RI".into(),
            year: "2020".into(),
            chamber: "congressional".into(),
            num_districts: 2,
            seed: Some(9_007_199_254_740_993),
            created_at: "2026-10-07T00:00:00Z".into(),
            balance_tolerance_pct: 0.5,
            population_balance_valid: true,
            ..Default::default()
        };
        let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        for name in REQUIRED_ANALYSIS_FILES {
            files.insert(
                format!("analysis/{name}"),
                serde_json::to_vec(
                    &serde_json::json!({"fixture":name,"count":9_007_199_254_740_993_u64}),
                )?,
            );
        }
        if case >= 1 {
            for name in OPTIONAL_ANALYSIS_FILES {
                files.insert(
                    format!("analysis/{name}"),
                    serde_json::to_vec(&serde_json::json!({"fixture":name,"status":"supplied"}))?,
                );
            }
        }
        if case >= 2 {
            files.insert("maps/districts.png".into(), vec![137, 80, 78, 71, 0, 255]);
        }
        if case >= 3 {
            manifest.audit_certificate_path = Some("audit/certificate.json".into());
            files.insert(
                "audit/certificate.json".into(),
                serde_json::to_vec(
                    &serde_json::json!({"algorithm_lineage":{"fixture":"unverified"}}),
                )?,
            );
        }
        if case >= 4 {
            manifest.rctx_path = Some("context.rctx".into());
            files.insert("context.rctx".into(), context.as_bytes().to_vec());
        }
        if case == 5 {
            files.insert("analysis/summary.json".into(), b"[1,2,3]".to_vec());
        }
        if case == 6 {
            files.insert("analysis/splits.json".into(), b"null".to_vec());
        }
        if case == 7 {
            files.insert(
                "analysis/compactness.json".into(),
                b"{\"status\":\"unavailable\",\"reason\":\"fixture\"}".to_vec(),
            );
        }
        if case == 8 {
            manifest.label = "<script>alert('label')</script> & \"quoted\"".into();
            manifest.state_code = "<img src=x onerror=alert('state')>".into();
            files.insert("analysis/partisan.json".into(), serde_json::to_vec(
                &serde_json::json!({"text":"</pre><script>alert('analysis')</script>"}))?);
            files.insert("audit/certificate.json".into(), serde_json::to_vec(
                &serde_json::json!({"algorithm_lineage":{"text":"<script>alert('lineage')</script>"}}))?);
        }
        files.insert(
            "manifest.json".into(),
            serde_json::to_vec_pretty(&manifest)?,
        );
        for (path, raw) in &files {
            let target = dir.join(path);
            std::fs::create_dir_all(target.parent().unwrap())?;
            std::fs::write(target, raw)?;
        }
        let mut native = assemble_report(&ReportContext::new(dir.clone(), manifest))?;
        native.generated_at = "2026-10-07T01:00:00Z".into();
        let portable = assemble_report_package(&files, native.generated_at.clone())?;
        let text = serde_json::to_string_pretty(&native)?;
        assert_eq!(text, serde_json::to_string_pretty(&portable)?);
        std::fs::write(out.join(format!("expected-{case}.json")), text)?;
        let html = bisect_report::render_html_report(&native)?;
        assert_eq!(html, bisect_report::render_html_report(&portable)?);
        if case == 8 {
            assert!(!html.contains("<script>"));
            assert!(!html.contains("<img src=x"));
            assert!(html.contains("&lt;script&gt;"));
        }
        std::fs::write(out.join(format!("expected-{case}.html")), html)?;

        std::fs::write(
            out.join(format!("request-{case}.json")),
            serde_json::to_vec(
                &serde_json::json!({"operation":"assemble-commission-report","files":files,"generated_at":native.generated_at}),
            )?,
        )?;
    }
    println!(
        "Nine actual filesystem-native/portable complete JSON and HTML report fixtures match exactly."
    );
    Ok(())
}
