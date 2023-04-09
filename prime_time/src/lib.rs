pub mod prime_time_server {

    use serde::{Deserialize, Serialize};
    use serde_json::Number;
    use std::io::{self, Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    const BUF_SIZE: usize = 1024 * 16;
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Request<'a> {
        method: &'a str,
        number: Number,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct Response<'a> {
        method: &'a str,
        prime: bool,
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
                                reponse(stream);
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
    // chuck read using buf and back_buf and back_buf_len
    fn reponse(mut stream: TcpStream) {
        println!(
            "==> new connection : {}",
            stream.peer_addr().unwrap().port()
        );
        let mut buf = [0; BUF_SIZE];
        let mut back_buf = [0; BUF_SIZE];
        let mut back_buf_end: usize = 0;
        loop {
            println!("waiting for request...");
            match chunk_read(&mut stream, &mut buf, &mut back_buf, &mut back_buf_end) {
                Ok(chunk_len) => {
                    println!("chunk len : {}", chunk_len);
                    println!("back buf end : {}", back_buf_end);
                    if chunk_len == 0 {
                        if back_buf_end > 0 {
                            continue;
                        }
                        println!("chunk len is 0");
                        break;
                    }

                    // print buf text
                    let v: Vec<Request> = parse_request(&buf, chunk_len);
                    println!("v : {:?}", v);
                    if v.is_empty() {
                        println!("1. invalid request !");
                        let s = std::str::from_utf8(&buf[..chunk_len]).unwrap();
                        println!("request : {:?}", s);
                        stream.write_all(&[0; 1]).unwrap();
                        break;
                    }

                    let wirte_buf = &mut [0; BUF_SIZE];
                    let mut wirte_buf_len = 0;
                    for request in v {
                        match is_prime_request(&request) {
                            Some(is_prime) => {
                                let response = Response {
                                    method: request.method,
                                    prime: is_prime,
                                };
                                wirte_buf_len = append_line(
                                    serde_json::to_string(&response).unwrap(),
                                    wirte_buf,
                                    wirte_buf_len,
                                );
                                if wirte_buf_len >= BUF_SIZE {
                                    stream.write_all(&wirte_buf[..wirte_buf_len]).unwrap();
                                    wirte_buf_len = 0;
                                }
                            }
                            None => {
                                println!("2. invalid request !");
                                println!("request : {:?}", request);
                                // malformed request, close connection
                                stream.write_all(&[0; 1]).unwrap();
                                return;
                            }
                        }
                    }
                    if wirte_buf_len > 0 {
                        stream.write_all(&wirte_buf[..wirte_buf_len]).unwrap();
                    }

                    println!("write finished.");
                }
                Err(err) => {
                    println!("read error : {:?}", err);
                    panic!("{}", err);
                }
            }
        }
        println!("connection closed");
    }
    pub fn chunk_read(
        mut stream: impl Read,
        buf: &mut [u8; BUF_SIZE],
        back_buf: &mut [u8; BUF_SIZE],
        back_buf_end: &mut usize,
    ) -> io::Result<usize> {
        buf[..*back_buf_end].copy_from_slice(&back_buf[..*back_buf_end]);
        let read_len = stream.read(&mut buf[*back_buf_end..])?;
        let read_len = *back_buf_end + read_len;
        if read_len == 0 {
            println!("read len is 0");
            return Ok(0);
        }

        let last_newline_idx = buf[..read_len]
            .iter()
            .rposition(|&c| c == b'\n')
            .unwrap_or(0);

        let mut chunk_len: usize = 0;
        if last_newline_idx == 0 {
            *back_buf_end = read_len;
        } else {
            chunk_len = last_newline_idx + 1;
            *back_buf_end = read_len - last_newline_idx - 1;
        }
        back_buf[..*back_buf_end].copy_from_slice(&buf[chunk_len..read_len]);
        Ok(chunk_len)
    }
    pub fn parse_request(buf: &[u8; BUF_SIZE], chunk_len: usize) -> Vec<Request> {
        let mut v: Vec<Request> = Vec::new();
        let mut start_i: usize = 0;
        for i in 0..chunk_len {
            if buf[i] == b'\n' {
                match serde_json::from_slice(&buf[start_i..i]) {
                    Ok(request) => v.push(request),
                    Err(err) => {
                        let slice_as_string = std::str::from_utf8(&buf[start_i..i]).unwrap();
                        println!("- not valid json format : {:?}", slice_as_string);
                        println!("- error : {:?}", err);
                    }
                }
                start_i = i + 1;
            }
            if buf[i] == b'\0' {
                break;
            }
        }
        v
    }
    fn is_prime_request(request: &Request) -> Option<bool> {
        let result: bool = match request.method {
            "isPrime" => match request.number {
                _ => is_prime(&request.number),
            },
            _ => {
                return None;
            }
        };
        Some(result)
    }
    fn is_prime(number: &serde_json::Number) -> bool {
        if let Some(n) = number.as_i64() {
            if n <= 0 || n == 1 {
                return false;
            }
            let max = (n as f64).sqrt() as i64 + 1;
            for i in 2..max {
                if n % i == 0 {
                    return false;
                }
            }
            true
        } else if let Some(_) = number.as_f64() {
            // Handle floating point numbers separately
            // ...
            false
        } else {
            false
        }
    }
    fn append_line(
        response_text: String,
        wirte_buf: &mut [u8; BUF_SIZE],
        wirte_buf_len: usize,
    ) -> usize {
        wirte_buf[wirte_buf_len..wirte_buf_len + response_text.len()]
            .copy_from_slice(response_text.as_bytes());
        wirte_buf[wirte_buf_len + response_text.len()] = b'\n';
        wirte_buf_len + response_text.len() + 1
    }
    #[cfg(test)]
    pub mod tests {
        use super::*;
        #[test]
        fn test_is_prime() {
            assert_eq!(is_prime(&Number::from(-1)), false, "-1 is not prime");
            assert_eq!(is_prime(&Number::from(0)), false, "0 is not prime");
            assert_eq!(is_prime(&Number::from(1)), false, "1 is not prime");
            assert_eq!(is_prime(&Number::from(2)), true, "2 is prime");
            assert_eq!(is_prime(&Number::from(3)), true, "3 is prime");
            assert_eq!(is_prime(&Number::from(4)), false, "4 is not prime");
            assert_eq!(is_prime(&Number::from(63454687)), true, "63454687 is prime");
        }
        #[test]
        fn test_is_prime_request() {
            assert_eq!(
                is_prime_request(&Request {
                    method: "IsPrime!!!",
                    number: Number::from(1),
                }),
                None,
                "IsPrime!!! is not a valid method"
            );
            assert_eq!(
                is_prime_request(&Request {
                    method: "isPrime",
                    number: Number::from(1),
                }),
                Some(false),
                "isPrime is valid method"
            );
        }
        #[test]
        fn test_parse_single_valid_request() {
            let param = "{\"method\":\"isPrime\",\"number\":1}\n";
            let mut buf = [0; BUF_SIZE];
            buf[..param.len()].copy_from_slice(param.as_bytes());
            let v = parse_request(&buf, param.len());
            assert_eq!(v.len(), 1, "1 valid request");
        }
        #[test]
        fn test_parse_single_valid_request_ignore_property() {
            let param =
                "{\"method\":\"isPrime\",\"number\":1,\"ignore_prop\": \"should ignore\"}\n";
            let mut buf = [0; BUF_SIZE];
            buf[..param.len()].copy_from_slice(param.as_bytes());
            let v = parse_request(&buf, param.len());
            assert_eq!(v.len(), 1, "1 valid request");
        }
        #[test]
        fn test_parse_single_valid_bignumber() {
            let param = "{\"method\":\"isPrime\",\"number\":64280796743577863187926287835338741906975448905104644648,\"bignumber\":true}\n";
            let mut buf = [0; BUF_SIZE];
            buf[..param.len()].copy_from_slice(param.as_bytes());
            let v = parse_request(&buf, param.len());
            assert_eq!(v.len(), 1, "1 valid request");
        }
        #[test]
        fn test_parse_multiline_valid_request() {
            let param = r#"{"method":"isPrime","number":1}
{"method":"isPrime","number":2}
"#;
            let mut buf = [0; BUF_SIZE];
            buf[..param.len()].copy_from_slice(param.as_bytes());
            let v = parse_request(&buf, param.len());
            assert_eq!(v.len(), 2, "2 valid request");
        }
        #[test]
        fn test_parse_single_invalid_request() {
            let param = "{\"method\":\"isPrime\",\"number\":1";
            let mut buf = [0; BUF_SIZE];
            buf[..param.len()].copy_from_slice(param.as_bytes());
            let v = parse_request(&buf, param.len());
            assert_eq!(v.len(), 0, "invalid request : last '' is missing");
        }
        #[test]
        fn test_copy_from_slice() {
            let buf = "123456789\nabcd".as_bytes();
            let mut back_buf = [0; BUF_SIZE];
            back_buf[0..9].copy_from_slice(&buf[0..9]);
            assert_eq!(&back_buf[0..9], "123456789".as_bytes());
        }
        #[test]
        fn test_chunk_read1() {
            let mut buf = [0; BUF_SIZE];
            let mut back_buf = [0; BUF_SIZE];
            let mut back_buf_end: usize = 0;
            let param = r#"{"method":"isPrime","number":1}
{"method":"isPrime","number":2}
"#;
            let chunk_len: usize = chunk_read(
                &mut param.as_bytes().to_vec().as_slice(),
                &mut buf,
                &mut back_buf,
                &mut back_buf_end,
            )
            .unwrap();
            assert_eq!(&buf[..chunk_len], param.as_bytes());
            assert_eq!(back_buf_end, 0);
        }
        #[test]
        fn test_chunk_smallhead() {
            let mut buf = [0; BUF_SIZE];
            let mut back_buf = [0; BUF_SIZE];
            let mut back_buf_end: usize = 0;
            let param = "{\"met";
            let chunk_len: usize = chunk_read(
                &mut param.as_bytes().to_vec().as_slice(),
                &mut buf,
                &mut back_buf,
                &mut back_buf_end,
            )
            .unwrap();
            assert_eq!(chunk_len, 0);
            assert_eq!(back_buf_end, 5);
            assert_eq!(&back_buf[0..back_buf_end], param.as_bytes());

            let param = "hod\":\"isPrime\",\"number\":1}\n";
            let chunk_len: usize = chunk_read(
                &mut param.as_bytes().to_vec().as_slice(),
                &mut buf,
                &mut back_buf,
                &mut back_buf_end,
            )
            .unwrap();
            assert_eq!(
                &buf[..chunk_len],
                "{\"method\":\"isPrime\",\"number\":1}\n".as_bytes()
            );
            assert_eq!(back_buf_end, 0);
        }
        #[test]
        fn test_chunk_read_longdata() {
            let mut buf = [0; BUF_SIZE];
            let mut back_buf = [0; BUF_SIZE];
            let mut back_buf_end: usize = 0;
            let param = r#"{\"method\":\"isPrime\",\"number\":1133498,\"nummar\":\"for love love royale all now calculator quartz time giant integral the of party giant peach quartz the giant the time come to favicon time for calculator is bluebell giant about the jackdaws peach the now hypnotic casino love of quartz something casino time giant now sphinx quartz hypnotic casino bluebell for prisoners integral the giant for come come all casino integral the good jackdaws come sphinx the the hypnotic for intrusion prisoners sphinx my the favicon royale of all men giant of the all the intrusion about to quartz peach men of about integral the to the calculator about the party royale all bluebell of nasa to party to something the party integral intrusion for men nasa the nasa giant hypnotic for PROTOHACKERS giant the the favicon for bluebell hypnotic jackdaws of the royale quartz the my of the love to prisoners royale quartz jackdaws to peach for hypnotic calculator for to giant of of sphinx time bluebell calculator PROTOHACKERS jackdaws all good love the jackdaws peach love for royale for bluebell all quartz good of giant quartz come to integral peach love quartz for all good integral intrusion of men favicon aid for favicon all to giant giant party royale hypnotic aid time intrusion integral love for of aid now prisoners men party nasa good hypnotic of my favicon hypnotic calculator giant to royale jackdaws intrusion of the nasa quartz my peach bluebell good now PROTOHACKERS aid for bluebell giant casino hypnotic time come calculator integral something giant of to about intrusion time aid is PROTOHACKERS giant for about now calculator nasa hypnotic now of aid royale my party integral the to giant prisoners the to hypnotic aid men the time intrusion peach peach the giant come love to sphinx of peach time the sphinx to PROTOHACKERS for is for PROTOHACKERS PROTOHACKERS calculator is jackdaws prisoners come men hypnotic to love the now come for prisoners peach nasa giant the integral of about jackdaws calculator love of of of now all casino integral the the all integral good PROTOHACKERS good casino all for for intrusion party for to time of giant love now now PROTOHACKERS giant hypnotic quartz quartz party hypnotic quartz to to men for giant of bluebell hypnotic of bluebell sphinx favicon nasa royale time prisoners time all hypnotic is come good giant my jackdaws peach the good the men calculator calculator good the the is sphinx time peach royale royale the the of love royale time integral quartz aid come nasa all for integral giant the something quartz men to is the nasa of hypnotic something giant to my calculator hypnotic favicon aid good to favicon PROTOHACKERS for royale come love of to something bluebell royale come integral royale prisoners integral the love something men bluebell giant men of all about giant time integral for quartz giant the now the all intrusion nasa the sphinx aid of now integral of peach love party party integral party for of nasa intrusion intrusion integral quartz men come come to about time prisoners for the hypnotic the hypnotic come all about calculator nasa of prisoners something intrusion of giant all all prisoners nasa giant men all nasa PROTOHACKERS for of now giant about royale calculator my peach jackdaws prisoners to all intrusion peach the calculator nasa party bluebell the nasa of the casino the to favicon prisoners love good love men men to calculator giant something my aid men all to all now party now for come time peach PROTOHACKERS giant men bluebell nasa of to jackdaws is peach all PROTOHACKERS time to time love the is about love hypnotic casino integral my my good giant to come is bluebell men royale for time party nasa come to party party sphinx casino to party to good for the now time all prisoners of all quartz prisoners the aid hypnotic love jackdaws is intrusion intrusion nasa is the calculator now favicon all calculator the quartz time something all the quartz aid prisoners nasa bluebell giant casino now something time bluebell quartz aid quartz giant integral intrusion is bluebell aid integral all favicon for the of about my men now the royale come now hypnotic to the integral love men for PROTOHACKERS of come peach for the sphinx something aid jackdaws the good is for party for bluebell good party peach to good good for time time the the for the to men intrusion good quartz to for bluebell love all come something my peach hypnotic aid good PROTOHACKERS come jackdaws sphinx intrusion to of favicon favicon favicon something jackdaws party royale favicon hypnotic nasa of intrusion good to the love royale nasa giant men jackdaws the hypnotic giant for nasa nasa party men about bluebell peach bluebell prisoners jackdaws of calculator to of integral now of the hypnotic PROTOHACKERS favicon of favicon to peach time intrusion of is favicon now something calculator intrusion for calculator jackdaws peach bluebell aid about PROTOHACKERS integral for of aid the for hypnotic royale integral jackdaws my bluebell about integral something all is bluebell men for good intrusion now something aid calculator jackdaws peach favicon intrusion giant favicon good to for come all hypnotic all love hypnotic peach favicon integral giant peach favicon to casino hypnotic of come about time royale royale quartz something something jackdaws peach PROTOHACKERS intrusion integral party PROTOHACKERS good something to peach hypnotic prisoners bluebell is is now PROTOHACKERS of bluebell casino good something prisoners men prisoners of for intrusion giant for good to giant time for of jackdaws bluebell PROTOHACKERS sphinx jackdaws my good men hypnotic sphinx aid love the the is sphinx nasa the party intrusion for for to the come now aid giant of giant love casino the the now the to giant calculator the the of calculator to time come giant party all men party integral favicon prisoners to the intrusion the hypnotic now come sphinx for jackdaws for the giant quartz royale royale PROTOHACKERS calculator is for now sphinx of favicon the to something to jackdaws intrusion quartz calculator all jackdaws nasa something is men calculator casino hypnotic the the intrusion nasa sphinx for favicon aid time my PROTOHACKERS royale the is integral men the for bluebell casino nasa party love men jackdaws jackdaws the prisoners is of jackdaws about for jackdaws royale is the quartz something integral good intrusion quartz to the prisoners is casino giant the giant quartz all party jackdaws sphinx the about integral peach men giant intrusion hypnotic come party peach quartz good good the prisoners giant is of the royale intrusion calculator is jackdaws jackdaws the to giant aid bluebell jackdaws nasa the love my nasa integral come my good men all something of of about party giant for the about the the time the about of my integral intrusion about of casino all bluebell now for all giant good calculator my quartz come royale giant hypnotic the the PROTOHACKERS now men of good good calculator casino love casino party my nasa all favicon now to come giant aid aid party royale to the to men come of party giant jackdaws the my love now jackdaws integral bluebell hypnotic good giant to men quartz for party of to love is my prisoners to PROTOHACKERS all all the for jackdaws my now PROTOHACKERS all of the prisoners nasa jackdaws hypnotic PROTOHACKERS something favicon time for love calculator about is of love favicon time the men calculator peach good giant love peach hypnotic for bluebell the favicon jackdaws of now hypnotic nasa quartz about aid love all casino giant for nasa something to my intrusion come time royale is peach for to quartz to bluebell hypnotic aid the giant royale now is to my party prisoners party casino bluebell party giant all bluebell is of favicon royale giant nasa the aid favicon bluebell peach giant men come for time to something to come to bluebell PROTOHACKERS about sphinx all now about to favicon to peach men party for aid the sphinx for giant about bluebell intrusion giant the aid favicon aid love aid of sphinx peach bluebell jackdaws is royale the of for men sphinx favicon favicon intrusion men men casino integral time peach giant giant something to PROTOHACKERS casino the time about the is bluebell jackdaws the to the the for prisoners integral the time party prisoners the to aid jackdaws good PROTOHACKERS the bluebell royale PROTOHACKERS men the time PROTOHACKERS PROTOHACKERS something good come royale casino PROTOHACKERS hypnotic the good giant about men favicon all PROTOHACKERS good the time of PROTOHACKERS men favicon all to something jackdaws something of of favicon to my time jackdaws hypnotic good is nasa casino for good now bluebell the party to casino of integral casino men royale time men to the the casino PROTOHACKERS men of quartz jackdaws my men aid something calculator peach casino all peach to to love favicon favicon love my good something time time for intrusion casino men intrusion to quartz to calculator now of now about hypnotic come royale jackdaws for something for something all to giant giant favicon favicon prisoners intrusion peach favicon favicon the of the now peach bluebell the hypnotic royale for is of to integral time of favicon is party prisoners quartz for giant aid come bluebell intrusion the love come aid hypnotic to of favicon my to my aid the come all nasa intrusion my sphinx love of about the for hypnotic the is sphinx of now giant about favicon aid jackdaws my time to all now of bluebell good is prisoners bluebell love sphinx about integral of is the is for party for the about bluebell something something sphinx come for men aid r"#;
            let chunk_len: usize = chunk_read(
                &mut param.as_bytes().to_vec().as_slice(),
                &mut buf,
                &mut back_buf,
                &mut back_buf_end,
            )
            .unwrap();
            assert_eq!(chunk_len, 0);
            assert_eq!(back_buf_end, 9649);
        }
        #[test]
        fn test_chunk_split() {
            let mut buf = [0; BUF_SIZE];
            let mut back_buf = [0; BUF_SIZE];
            let mut back_buf_end: usize = 0;
            let param = r#"{"method":"isPrime","number":1}
{"method":"isPrime","number":2}
{"method":"isPrime","#;
            let chunk_len: usize = chunk_read(
                &mut param.as_bytes().to_vec().as_slice(),
                &mut buf,
                &mut back_buf,
                &mut back_buf_end,
            )
            .unwrap();

            assert_eq!(
                &buf[..chunk_len],
                r#"{"method":"isPrime","number":1}
{"method":"isPrime","number":2}
"#
                .as_bytes()
            );
            let param = r#""number":3}
"#;
            let chunk_len: usize = chunk_read(
                &mut param.as_bytes().to_vec().as_slice(),
                &mut buf,
                &mut back_buf,
                &mut back_buf_end,
            )
            .unwrap();

            assert_eq!(
                &buf[..chunk_len],
                r#"{"method":"isPrime","number":3}
"#
                .as_bytes()
            );
        }
        #[test]
        fn test_parse_multiline_800_requst() {
            let mut buf = [0; BUF_SIZE];
            let mut back_buf = [0; BUF_SIZE];
            let mut back_buf_end: usize = 0;
            let mut param = String::new();
            for i in 0..800 {
                param.push_str(&format!(
                    r#"{{"method":"isPrime","number":{}}}
"#,
                    78590107 + i
                ));
            }

            let binding = param.as_bytes().to_vec();
            let mut stream = binding.as_slice();

            loop {
                let chunk_len: usize =
                    chunk_read(&mut stream, &mut buf, &mut back_buf, &mut back_buf_end).unwrap();
                if chunk_len == 0 {
                    break;
                }

                let s = std::str::from_utf8(&buf[..chunk_len]).unwrap();
                println!("Chunk: {}", s);
                assert_eq!(&buf[chunk_len - 1], &b'\n', "800 valid request");
            }
            #[test]
            fn test_append_line() {
                let mut wirte_buf = [0; BUF_SIZE];
                let mut wirte_buf_len: usize = 0;
                let response_text = "hello world".to_string();
                wirte_buf_len = append_line(response_text, &mut wirte_buf, wirte_buf_len);
                assert_eq!(&wirte_buf[..wirte_buf_len], "hello world\n".as_bytes());
            }            
        }
    }
}
