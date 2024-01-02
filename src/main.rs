use std::{
    error::Error,
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc},
};

use httparse::{Request, Response};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpSocket, TcpStream},
    signal,
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

async fn handle_https_connection() -> Result<(), Box<dyn Error>> {
    todo!()
}

async fn handle_connection(mut client_stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut server_stream: Option<TcpStream> = None;
    let mut buff = [0; 1028];
    let bytes_read = client_stream.read(&mut buff).await?;
    if bytes_read == 0 {
        println!("Connection closed by client");
        return Ok(());
    }
    let data = &buff[..bytes_read];

    if data.starts_with(b"CONNECT") {
        let elems = String::from_utf8_lossy(data)
            .lines()
            .next()
            .map(|l| {
                l.split_whitespace()
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap();

        let conn = elems.get(1).expect("Missing connection");
        let (host, port) = conn.split_once(':').unwrap();
        let ipaddr = dnsoverhttps::resolve_host(host)
            .unwrap()
            .get(0)
            .cloned()
            .unwrap();
        let server_socket = if ipaddr.is_ipv4() {
            TcpSocket::new_v4()
        } else {
            TcpSocket::new_v6()
        }?;
        let socket_addr = SocketAddr::new(ipaddr, port.parse().unwrap_or(443));
        println!("Resolved {:?} to socket {:?}", (host, port), socket_addr);
        server_stream = Some(server_socket.connect(socket_addr).await?);
        let data = b"HTTP/1.1 200 OK\r\n\r\n";
        client_stream.write_all(data).await?;

        // Capture the TLS handshake
        //
        let bytes_read = client_stream.read(&mut buff).await?;
        if bytes_read == 0 {
            println!("Connection closed by client");
            return Ok(());
        }
        let data = &buff[..bytes_read];
        for chunk in data.chunks(6) {
            server_stream.as_mut().unwrap().write_all(chunk).await?;
        }
    }
    if let Some(ssteam) = &mut server_stream {
        tokio::io::copy_bidirectional(ssteam, &mut client_stream).await?;
    // } else {
    //     println!("Got chunk: \n{}\n============", String::from_utf8_lossy(&buff));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket = TcpSocket::new_v4()?;
    socket.set_reuseaddr(true)?;
    socket.bind("0.0.0.0:8000".parse().unwrap())?;

    let listener = socket.listen(128)?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error handling connection {e}");
            }
        });
    }
}
