use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            // Read request
            if let Ok(bytes_read) = socket.read(&mut buffer).await {
                if bytes_read == 0 {
                    return;
                }

                // Convert bytes to string (HTTP request)
                let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Received request:\n{}", request);

                // Simple HTTP response
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, world!";
                socket.write_all(response.as_bytes()).await.unwrap();
                socket.flush().await.unwrap();
            }
        });
    }
}
