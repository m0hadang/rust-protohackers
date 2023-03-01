pub mod echo_server {
    use std::io::Read;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};    

    fn echo(mut stream: TcpStream) {
        loop {
            let mut read = [0; 1028];
            match stream.read(&mut read) {
                Ok(n) => {
                    if n == 0 {
                        // connection was closed
                        break;
                    }
                    stream.write(&read[0..n]).unwrap();
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }
        }
    }

    pub fn run_server(ip: &str, port: u32) {
        match TcpListener::bind(format!("{ip}:{port}")) {
            Ok(listener) => {
                println!("running...");
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            println!("accept client...");
                            echo(stream);
                        }
                        Err(err) => {
                            println!("incoming error : {}", err);
                        }
                    }
                }
            }
            Err(err) => {
                println!("tco bind error : {}", err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
