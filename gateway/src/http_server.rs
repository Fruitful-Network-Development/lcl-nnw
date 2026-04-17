use crate::router::RouteDecision;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn write_response(mut stream: TcpStream, status_line: &str, body: &str) -> std::io::Result<()> {
    let response = format!(
        "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())
}

fn route_body(route: &RouteDecision) -> String {
    format!(
        "{{\"profile\":\"{}\",\"model_alias\":\"{}\",\"fallback_model_alias\":\"{}\",\"backend\":\"{}\",\"endpoint\":\"{}\"}}",
        route.profile, route.model_alias, route.fallback_model_alias, route.backend, route.endpoint
    )
}

pub fn serve_once(addr: &str, route: &RouteDecision) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    if let Ok((mut stream, _peer)) = listener.accept() {
        let mut buf = [0_u8; 1024];
        let size = stream.read(&mut buf)?;
        let request = String::from_utf8_lossy(&buf[..size]);

        if request.starts_with("GET /health") {
            return write_response(stream, "HTTP/1.1 200 OK", "{\"status\":\"ok\"}");
        }

        if request.starts_with("GET /route") {
            let body = route_body(route);
            return write_response(stream, "HTTP/1.1 200 OK", &body);
        }

        return write_response(
            stream,
            "HTTP/1.1 404 Not Found",
            "{\"error\":\"not_found\"}",
        );
    }

    Ok(())
}
