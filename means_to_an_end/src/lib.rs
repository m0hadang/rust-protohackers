pub mod means_to_an_end_server {
    use std::io::Read;
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    pub struct Request {
        char: char,
        data1: i32,
        data2: i32,
    }

    pub fn run_server(ip: &str, port: u32) {
        match TcpListener::bind(format!("{ip}:{port}")) {
            Ok(listener) => {
                println!("wait client...");
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            println!("accept client...");
                            thread::spawn(|| {
                                handle(stream);
                            });
                            println!("get next connect");
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
    
    fn handle(mut stream: TcpStream) {
        println!(
            "==> new connection : {}",
            stream.peer_addr().unwrap().port()
        );
        let mut buf = [0; 9];
        match stream.read(&mut buf) {
            Ok(read_len) => {
                let request_cnt = read_len / 9;

                let mut i = 0;
                while i < read_len {
                    i += 1
                }
                
                let req = Request {
                    char: buf[0] as char,
                    data1: ((buf[1] as i32) << 24)
                        + ((buf[2] as i32) << 16)
                        + ((buf[3] as i32) << 8)
                        + (buf[4] as i32),
                    data2: ((buf[5] as i32) << 24)
                        + ((buf[6] as i32) << 16)
                        + ((buf[7] as i32) << 8)
                        + (buf[8] as i32),
                };                
            }
            Err(err) => {
                println!("read error : {}", err);
            }
        }
    }
    #[test]
    fn sample_test() {

    }
}