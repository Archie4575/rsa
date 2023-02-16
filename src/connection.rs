use std::net::{TcpListener, TcpStream};
use std::io::{Write};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

struct Listener {
    listener: TcpListener,
    query: Mutex<VecDeque<TcpStream>>,
}

impl Listener {
    fn start(&self, quit: &bool) {
        while *quit {
            match self.listener.accept() {
                Ok((stream, _)) => {
                let query = &mut self.query.lock().unwrap(); 
                query.push_back(stream); 
                println!("Added Stream: {:?} in {:?}", query.back(), query)
                },
                Err(e) => println!("couldn't get client: {e:?}")
            }
        }      
    }
}

struct OutStreams<'o> {
    stream: Mutex<TcpStream>,
    query: Mutex<VecDeque<&'o str>>,
}

impl<'o> OutStreams<'o> {
    fn connect() ->  {
        let stream = TcpStream::connect(ref_addr);
        match stream {
            Ok(_) => println!("Connected to the server!"),
            Err(e) => panic!("Couldn't connect to server...\n{e:?}")
        }
        stream
    }

    fn send(&self) -> bool {
        let query = &mut self.query.lock().unwrap();
        let message = query[0];
        match self.stream.lock().unwrap().write(message.as_bytes()) {
            Ok(_) => {println!("Sent {:p}, Query: {:?}", message, query); self.stream.lock().unwrap().flush(); return true},
            Err(e) => println!("WARNING!!  {e:?}")
        };
        false
    }
}

pub struct StreamsController<'s> {
    inc: Arc<Listener>, 
    out: Arc<OutStreams<'s>>,
    quit: Arc<Mutex<bool>>
}

impl<'s> StreamsController<'s> {
    pub fn new() -> Self {

        fn get_full_addr(addr: &str, port: &str) -> &str {
        let mut full_addr: String = addr.to_owned();
        full_addr.push_str(":");
        full_addr.push_str(port);
        let ref_addr: &str = full_addr.as_str();
        ref_addr
        }

        let listen_addr = get_full_addr("127.0.0.1", &9999.to_string());
        let out_addr = get_full_addr("127.0.0.1", &9999.to_string());

        let quit = Arc::new(Mutex::new(false));

        Self {
            inc: Listener {
                listener: TcpListener::bind(&addr).unwrap(), query: Mutex::new(VecDeque::new())
            }, 
            out: OutStreams {

                stream: Mutex::new(stream.unwrap()), query: Mutex::new(VecDeque::new())
            }, 
            quit}
    }

    pub fn start_listener(&mut self) {
        Arc::get_mut(&mut self.inc).unwrap().start(Arc::get_mut(&mut self.quit).unwrap().get_mut().unwrap());
    }


    pub fn sync(&mut self) {
        while !*Arc::get_mut(&mut self.quit).unwrap().get_mut().unwrap() {
            let inc_query = self.inc.query.lock().unwrap();
            let out_query =  self.out.query.lock().unwrap();
            print!("{inc_query:?}");
            if !inc_query.is_empty() {
                println!("{:?}", inc_query[0]);
                self.inc.query.lock().unwrap().pop_front();
            }

            if !out_query.is_empty() {
                println!("{:?}", out_query[0]);

                if self.out.send() {
                    self.out.query.lock().unwrap().pop_front();
                }

            }
            self.out.query.lock().unwrap().push_back("ds");
            print!("test");
        }
    }

    pub fn quit(&mut self) {
        *self.quit.lock().unwrap() = true;
    }
}