extern crate getopts;
use std::io;
use std::os::args;
use std::os;
use std::option::Option;
use getopts::{optflag,getopts};
use std::io::{Acceptor, Listener};

fn main() {
  let prog_opts = match parse_args() {
    Some(t) => { t }
    None => { return; }
  };

  if prog_opts.help {
    print_help();
    return;
  }

  if prog_opts.listen {
    match tcp_listen(prog_opts.ip.as_slice(), prog_opts.port) {
      Some(m) => {
        let m_read = io::BufferedReader::new(m.clone());
        let m_write = io::BufferedWriter::new(m.clone());
        spawn(proc() in_to_out(m_read, io::stdio::stdout_raw()));
        spawn(proc() in_to_out(io::stdio::stdin_raw(), m_write));
      }
      None => { return; }
    }

  } else {
    match tcp_connect(prog_opts.ip.as_slice(), prog_opts.port) {
      Some(m) => {
        let m_read = io::BufferedReader::new(m.clone());
        let m_write = io::BufferedWriter::new(m.clone());
        spawn(proc() in_to_out(io::stdio::stdin_raw(), m_write));
        spawn(proc() in_to_out(m_read, io::stdio::stdout_raw()));
      }
      None => { return; }
    }
  }
}

struct ProgOpts {
  ip: String,
  port: u16,
  listen: bool,
  verbose: bool,
  help: bool
}

fn parse_args() -> Option<ProgOpts> {
  let mut prog_opts = ProgOpts {
    ip: "127.0.0.1".into_string(),
    port: 0,
    listen: false,
    verbose: false,
    help: false
  };
  let args = os::args();
  let opts = [
    optflag("l","listen","Listen for a connection on ip and port instead of connecting"),
    optflag("h","help","Print help"),
    optflag("v","verbose","Print debug output to stderr")
  ];
  let matches = match getopts(args.tail(), opts) {
    Ok(m) => { m }
    Err(f) => {
      print_error(f, 1);
      return None;
    }
  };

  if matches.opt_present("h") {
    prog_opts.help = true;
  }
  if matches.opt_present("l") {
    prog_opts.listen = true;
  }
  if matches.opt_present("v") {
    prog_opts.verbose = true;
  }
  
  match matches.free.len() {
    1 => {
      if !prog_opts.listen {
        print_error("Not enough arguments", 1);
        return None;
      }
      prog_opts.port = match from_str::<u16>(matches.free[0].as_slice()) {
        Some(a) => { a }
        None => {
          print_error("Port isn't a number.", 1);
          return None;
        }
      };
    }
    2 => {
      prog_opts.ip = matches.free[0].clone();
      prog_opts.port = match from_str::<u16>(matches.free[1].as_slice()) {
        Some(a) => { a }
        None => {
          print_error("Port isn't a number.", 1);
          return None;
        }
      };
    }
    _ => {
      print_error("Too many arguments.", 1);
      return None;
    }
  }
  Some(prog_opts)
}

fn print_error<A: std::fmt::Show>( error: A, errno: int ) {
  writeln!(io::stdio::stderr(), "{}", error);
  os::set_exit_status(errno);
}


fn print_help() {
  println!("HELP!");
  println!("SOMEBODY HELP ME, HELP!");
}

fn tcp_listen( ip: &str, port: u16 ) -> Option<io::net::tcp::TcpStream> {
  let listener = match io::net::tcp::TcpListener::bind(ip, port) {
    Ok(m) => { m }
    Err(e) => {
      print_error(e, 2);
      return None;
    }
  };
  let mut acceptor = match listener.listen() {
    Ok(m) => { m }
    Err(e) => {
      print_error(e, 2);
      return None;
    }
  };
  match acceptor.accept() {
    Ok(m) => { Some(m) }
    Err(e) => {
      print_error(e, 2);
      return None;
    }
  }
}

fn tcp_connect( ip: &str, port: u16 ) -> Option<io::net::tcp::TcpStream> {
  match io::net::tcp::TcpStream::connect(ip, port) {
    Ok(m) => { Some(m) }
    Err(e) => {
      print_error(e.desc, 3);
      return None;
    }
  }
}

fn in_to_out<A: io::Reader, B: io::Writer>( input: A, output: B ) {
  // let mut input_buffer = io::BufferedReader::new(input);
  // let mut output_buffer = io::BufferedWriter::new(output);
  let mut input_buffer = input;
  let mut output_buffer = output;
  let mut buf = [0, ..1];
  let mut count: uint;

  loop{
    count = match input_buffer.read(buf) {
      Ok(a) => {a}
      Err(e) => {
        match e.kind {
          io::EndOfFile => { return; }
          _ => {
            print_error(e, 5);
            return;
          }
        }
      }
    };

    match output_buffer.write(buf.slice(0, count)) {
      Ok(_) => {}
      Err(e) => {
        print_error(e, 6);
        return;
      }
    }
  }
}
