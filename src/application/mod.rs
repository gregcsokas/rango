use tokio::net::TcpListener;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use bytes::BytesMut;

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>
}

#[derive(Default)]
pub struct Application {
    port: u16
}

impl Application {
    pub fn new() -> Self {
        Self {
            port: 8000,
        }
    }

    async fn handle_connection(mut socket: tokio::net::TcpStream) {
        let mut buffer = BytesMut::with_capacity(1024);

                match socket.read_buf(&mut buffer).await {
                    Ok(_) => {

                        match Self::parse_request(&buffer) {
                            Ok(request) => {
                                println!("Received request: {:?}", request);

                                println!("Method: {:?}", request.method);
                                println!("Path: {:?}", request.path);
                                println!("Headers: {:?}", request.headers);

                                let response = format!(
                                    "HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/plain\r\n\
                                    Content-Length: 59\r\n\
                                    \r\n\
                                    Received {} request to {} with {} headers",
                                    request.method,
                                    request.path,
                                    request.headers.len()
                                );

                                if let Err(e) = socket.write_all(response.as_bytes()).await {
                                    eprintln!("Failed to write response: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse request: {}", e);
                                let response = "HTTP/1.1 400 Bad Request\r\n\r\nInvalid request";
                                let _ = socket.write_all(response.as_bytes()).await;
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to read from socket: {}", e),
                }
    }

    fn parse_request(buffer: &[u8]) -> Result<Request, Box<dyn std::error::Error + Send + Sync>> {

        // Maximum 16 headers supported
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);

        let result = req.parse(buffer)?;

        if result.is_partial() {
            return Err("Incomplete request".into());
        }

        let method = req.method.ok_or("No method")?.to_string();
        let path = req.path.ok_or("No path")?.to_string();

        let headers = headers
            .iter()
            .take_while(|h| h.name.len() > 0)
            .map(|h| {
                (
                    h.name.to_string(),
                    String::from_utf8_lossy(h.value).into_owned()
                    )
            })
            .collect();

        let headers_len = result.unwrap();
        let body = buffer[headers_len..].to_vec();

        Ok(Request {
            method,
            path,
            headers,
            body,
        })
    }

    pub async fn run(self) {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await.unwrap();

        println!("Server running at http://{}", addr);

        loop {
            let (socket, addr) = listener.accept().await.unwrap();
            println!("New connection from: {}", addr);

            tokio::spawn(async move {
                Self::handle_connection(socket).await;
            });
        }
    }
}