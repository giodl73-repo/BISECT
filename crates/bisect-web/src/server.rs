use crate::{data, jobs::Laboratory, model::Experiment};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    io::Read,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

const HTML: &str = include_str!("../../../web/lab/index.html");
const CSS: &str = include_str!("../../../web/lab/lab.css");
const JS: &str = include_str!("../../../web/lab/lab.js");
const STATIC_JS: &str = include_str!("../../../web/lab/static.js");

fn header(request: &Request, name: &'static str) -> Option<String> {
    request
        .headers()
        .iter()
        .find(|h| h.field.equiv(name))
        .map(|h| h.value.to_string())
}
pub fn local_request(host: &str, origin: Option<&str>, port: u16) -> bool {
    let hosts = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
    hosts.contains(&host.to_owned())
        && origin.is_none_or(|origin| hosts.iter().any(|host| origin == format!("http://{host}")))
}
fn respond(request: Request, status: u16, mime: &str, body: Vec<u8>) {
    let response=Response::from_data(body).with_status_code(StatusCode(status))
        .with_header(Header::from_bytes("Content-Type",mime).unwrap())
        .with_header(Header::from_bytes("Cache-Control","no-store").unwrap())
        .with_header(Header::from_bytes("X-Content-Type-Options","nosniff").unwrap())
        .with_header(Header::from_bytes("Referrer-Policy","same-origin").unwrap())
        .with_header(Header::from_bytes("Content-Security-Policy","default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'").unwrap());
    let _ = request.respond(response);
}
fn query(url: &str, name: &str) -> Option<String> {
    url.split_once('?')?.1.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        if key == name {
            Some(value.to_owned())
        } else {
            None
        }
    })
}

fn route(request: &mut Request, lab: &Arc<Laboratory>) -> Result<(&'static str, Vec<u8>)> {
    let url = request.url().to_owned();
    let path = url.split('?').next().unwrap_or(&url);
    if request.method() == &Method::Get {
        let asset = match path {
            "/" => Some(("text/html; charset=utf-8", HTML)),
            "/lab.css" => Some(("text/css; charset=utf-8", CSS)),
            "/lab.js" => Some(("text/javascript; charset=utf-8", JS)),
            "/static.js" => Some(("text/javascript; charset=utf-8", STATIC_JS)),
            _ => crate::input_assets::INPUT_ASSETS.iter().find(|(name,_)|path==format!("/{name}")).map(|(_,source)|("text/javascript; charset=utf-8",*source)),
        };
        if let Some((mime, body)) = asset {
            return Ok((mime, body.as_bytes().to_vec()));
        }
        let value = match path {
            "/api/catalog" => {
                let mut value = lab.paths.catalog();
                value["token"] = json!(lab.token);
                value
            }
            "/api/runs" => lab.list(),
            "/api/geometry" => {
                let code = query(&url, "state").context("Choose a state.")?;
                let year = query(&url, "year").context("Choose a year.")?;
                ensure!(
                    lab.paths.codes().contains(&code)
                        && ["2000", "2010", "2020"].contains(&year.as_str()),
                    "Unknown state or year."
                );
                let _guard = lab.geometry_lock.lock().unwrap();
                data::geometry(&lab.paths, &code, &year)?
            }
            _ => {
                let parts: Vec<_> = path.trim_matches('/').split('/').collect();
                ensure!(
                    parts.len() >= 3 && parts[0] == "api" && parts[1] == "runs",
                    "Route not found."
                );
                let job = lab.get(parts[2])?;
                if parts.len() == 3 {
                    json!(job)
                } else {
                    ensure!(parts.len() == 5, "Route not found.");
                    let code = parts[3];
                    ensure!(
                        job.config.states.iter().any(|s| s == code),
                        "State is not in this experiment."
                    );
                    let name = match parts[4] {
                        "map" => format!("{code}_map.json"),
                        "assignments" => format!("{code}_assignments.json"),
                        _ => anyhow::bail!("Route not found."),
                    };
                    let bytes =
                        std::fs::read(lab.paths.store.join("jobs").join(&job.id).join(name))
                            .context("State result is not ready.")?;
                    return Ok(("application/json", bytes));
                }
            }
        };
        return Ok(("application/json", serde_json::to_vec(&value)?));
    }
    ensure!(request.method() == &Method::Post, "Method not allowed.");
    ensure!(
        header(request, "X-Lab-Token").as_deref() == Some(&lab.token),
        "Missing laboratory session token."
    );
    ensure!(
        header(request, "Content-Type").is_some_and(|v| v.starts_with("application/json")),
        "Expected application/json."
    );
    if let Some(length) = header(request, "Content-Length") {
        ensure!(length.parse::<usize>()? <= 65536, "Request is too large.");
    }
    let mut body = Vec::new();
    request.as_reader().take(65537).read_to_end(&mut body)?;
    ensure!(body.len() <= 65536, "Request is too large.");
    let value: Value = serde_json::from_slice(&body)?;
    let result = if path == "/api/runs" {
        let config: Experiment = serde_json::from_value(value)?;
        json!(lab.submit(config)?)
    } else {
        let parts: Vec<_> = path.trim_matches('/').split('/').collect();
        ensure!(
            parts.len() == 4 && parts[0] == "api" && parts[1] == "runs" && parts[3] == "cancel",
            "Route not found."
        );
        lab.cancel(parts[2])?;
        json!({"cancelled":true})
    };
    Ok(("application/json", serde_json::to_vec(&result)?))
}

pub fn serve(lab: Arc<Laboratory>, port: u16) -> Result<()> {
    let server = Server::http(("127.0.0.1", port))
        .map_err(|e| anyhow::anyhow!("Cannot start local server: {e}"))?;
    println!("BISECT Laboratory: http://127.0.0.1:{port}");
    println!("Experiment store: {}", lab.paths.store.display());
    let active = Arc::new(AtomicUsize::new(0));
    for mut request in server.incoming_requests() {
        if !local_request(
            &header(&request, "Host").unwrap_or_default(),
            header(&request, "Origin").as_deref(),
            port,
        ) {
            respond(
                request,
                403,
                "application/json",
                br#"{"error":"Local requests only."}"#.to_vec(),
            );
            continue;
        }
        if active.load(Ordering::Relaxed) >= 16 {
            respond(
                request,
                503,
                "application/json",
                br#"{"error":"Laboratory is busy."}"#.to_vec(),
            );
            continue;
        }
        active.fetch_add(1, Ordering::Relaxed);
        let active = active.clone();
        let lab = lab.clone();
        std::thread::spawn(move || {
            match route(&mut request, &lab) {
                Ok((mime, body)) => respond(request, 200, mime, body),
                Err(error) => respond(
                    request,
                    400,
                    "application/json",
                    serde_json::to_vec(&json!({"error":format!("{error:#}")})).unwrap(),
                ),
            }
            active.fetch_sub(1, Ordering::Relaxed);
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blocks_remote_origins_and_rebinding_hosts() {
        assert!(local_request(
            "127.0.0.1:4317",
            Some("http://127.0.0.1:4317"),
            4317
        ));
        assert!(!local_request("evil.example:4317", None, 4317));
        assert!(!local_request(
            "localhost:4317",
            Some("https://evil.example"),
            4317
        ));
        assert!(!local_request(
            "localhost:4317",
            Some("http://localhost:9999"),
            4317
        ));
    }
}
