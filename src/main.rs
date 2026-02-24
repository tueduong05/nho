use tokio::{io, net::TcpListener};

use crate::{
    command::Command,
    protocol::{Connection, Response},
};

mod command;
mod protocol;
mod storage;

#[tokio::main]
async fn main() -> io::Result<()> {
    let store = storage::new_store();

    let listener = TcpListener::bind("0.0.0.0:6379").await?;
    println!("Listening on 0.0.0.0:6379");

    loop {
        let (socket, address) = listener.accept().await?;
        println!("New connection from: {}", address);

        let store = store.clone();

        tokio::spawn(async move {
            let mut connection = Connection::new(socket);

            loop {
                match connection.read_frame().await {
                    Ok(Some(command)) => {
                        let response = match command {
                            Command::Ping => Response::Ok,
                            Command::Get(key) => match storage::get(&store, &key) {
                                Some(value) => Response::Data(value),
                                None => Response::Nil,
                            },
                            Command::Set(key, value, ttl) => {
                                storage::set(&store, key, value, ttl);
                                Response::Ok
                            }
                            Command::Unknown => Response::Error("Unknown command".to_string()),
                        };

                        connection.write_response(response);
                    }
                    Ok(None) => {
                        println!("Connection closed by client: {}", address);
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error handling client {}: {}", address, e);
                        break;
                    }
                }
            }

            let _ = connection.flush().await;
        });
    }
}
