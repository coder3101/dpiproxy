use std::error::Error;

use httparse::{Request, Response};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{unix::SocketAddr, TcpSocket, TcpStream},
};

fn response_to_bytes(response: &Response) -> Result<Vec<u8>, Box<dyn Error>> {
    let status_code = response.code.unwrap_or(200); // Default to 200 OK if status code is not specified
    let reason = response.reason.unwrap_or("OK"); // Default to "OK" if reason is not specified

    let mut http_response = format!("HTTP/1.1 {} {}\r\n", status_code, reason);

    for header in response.headers.iter().filter(|h| !h.value.is_empty()) {
        http_response.push_str(&format!(
            "{}: {}\r\n",
            header.name,
            String::from_utf8(header.value.to_vec())?
        ));
    }

    http_response.push_str("\r\n");

    Ok(http_response.into_bytes())
}

fn request_to_bytes(request: &Request) -> Result<Vec<u8>, Box<dyn Error>> {
    let method = request.method.unwrap_or("GET"); // Default to GET if method is not specified
    let path = request.path.unwrap_or("/"); // Default to '/' if path is not specified

    let mut http_request = format!("{} {} HTTP/1.1\r\n", method, path);

    for header in request.headers.iter().filter(|h| !h.value.is_empty()) {
        http_request.push_str(&format!(
            "{}: {}\r\n",
            header.name,
            String::from_utf8(header.value.to_vec())?
        ));
    }

    http_request.push_str("\r\n");

    Ok(http_request.into_bytes())
}

async fn handle_connection(
    mut stream: TcpStream,
    _: std::net::SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut buf = String::new();
    let mut server_stream = None;
    let (mut reader, mut writer) = stream.split();
    while (reader.read_to_string(&mut buf).await).is_ok() {
        let mut h = [httparse::EMPTY_HEADER; 16];
        let mut req = Request::new(&mut h);
        req.parse(buf.as_bytes())?;

        if let Some("CONNECT") = req.method {
            let res = Response {
                version: req.version,
                code: Some(200),
                reason: Some("Connection Established"),
                headers: &mut [],
            };
            let server_socket = TcpSocket::new_v4()?;
            server_stream = Some(server_socket.connect(req.path.unwrap().parse()?).await?);
            let data = response_to_bytes(&res)?;
            writer.write_all(&data).await?;
        } else if let Some(stream) = &mut server_stream {
            stream.write_all(buf.as_bytes()).await?;
        }
    }
    println!("Done inner");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket = TcpSocket::new_v4()?;
    socket.bind("0.0.0.0:8001".parse().unwrap())?;

    let listener = socket.listen(128)?;
    loop {
        let con = listener.accept().await?;
        tokio::spawn(async move {
            let _ = handle_connection(con.0, con.1).await;
        });

        println!("New request processing...");
    }
    // Ok(())
}
