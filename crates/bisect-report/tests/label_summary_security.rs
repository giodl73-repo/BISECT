use bisect_report::label_summary::render_label_html_report;

#[test]
fn metadata_cannot_close_title_or_body_text_contexts() {
    let label = "</title><script>evilLabel()</script>&\"'";
    let year = "</p><img src=x onerror=evilYear()>";
    let html = render_label_html_report(label, year, &serde_json::json!({"states":{}}));
    assert!(!html.contains("<script>"));
    assert!(!html.contains("<img "));
    assert_eq!(html.matches("&lt;/title&gt;&lt;script&gt;evilLabel()&lt;/script&gt;&amp;&quot;&#39;").count(), 2);
    assert_eq!(html.matches("&lt;/p&gt;&lt;img src=x onerror=evilYear()&gt;").count(), 2);
}
