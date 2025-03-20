use std::net::{TcpListener, TcpStream};
use std::{io, thread};
fn main() -> io::Result<()>{
    println!("Starting your Reverse Proxy");

    let listen_addr = "127.0.0.1:8080";
    let backed_addr = "127.0.0.1:9000";
    run_proxy_server(listen_addr, backed_addr)?;
    Ok(())
}



fn run_proxy_server(listen_addr: &str,backed_addr: &str) -> io::Result<()> {
    let listner = TcpListener::bind(listen_addr)?;
    println!("Proxy running at {}", listen_addr);
    println!("backend running at {}", backed_addr);

    for stream in listner.incoming() {
        match stream {
            Ok(client_stream)=> {
                let backend = backed_addr.to_string();
                thread::spawn(move || {
                    if let Err(e) = handle_connection(client_stream, &backend) {
                        eprintln!("Error accepting connection")
                    }
                });
            }
            Err(e) => {
                println!("we got this error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_connection(mut client_stream: TcpStream, backend_addr: &str)-> io::Result<()>{

    println!("Handled Connection from : {}", client_stream.peer_addr()?);
   Ok(())
}
