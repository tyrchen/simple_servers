use lazy_static::lazy_static;

use clap::App;
use snow::params::NoiseParams;
use snow::Builder;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

static SECRET: &'static [u8] = b"XTSPPFrCk7sZmBFm8Hm6cXjjS7Ddd3PV";
lazy_static! {
    static ref PARAMS: NoiseParams = "Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s".parse().unwrap();
}

fn main() {
    let matches = App::new("simple")
        .args_from_usage("-s --server 'server mode'")
        .get_matches();

    if matches.is_present("server") {
        run_server();
    } else {
        run_client();
    }
    println!("all done.");
}

fn run_server() {
    let mut buf = vec![0u8; 65535];

    // initialize responder
    let builder: Builder<'_> = Builder::new(PARAMS.clone());
    let static_key = builder.generate_keypair().unwrap().private;
    let mut noise = builder
        .local_private_key(&static_key)
        .psk(3, SECRET)
        .build_responder()
        .unwrap();

    // wait on client's arrival
    println!("Listening on 0.0.0.0:9999");
    let (mut stream, _) = TcpListener::bind("0.0.0.0:9999").unwrap().accept().unwrap();

    // <- e
    noise
        .read_message(&recv(&mut stream).unwrap(), &mut buf)
        .unwrap();

    // -> e, ee, s, es
    let len = noise.write_message(&[0u8; 0], &mut buf).unwrap();
    send(&mut stream, &buf[..len]);

    // <- s, se
    noise
        .read_message(&recv(&mut stream).unwrap(), &mut buf)
        .unwrap();

    // transition the state machine to transport mode sinc handshake is complete.
    let mut noise = noise.into_transport_mode().unwrap();

    while let Ok(msg) = recv(&mut stream) {
        let len = noise.read_message(&msg, &mut buf).unwrap();
        println!("client said: {}", String::from_utf8_lossy(&buf[..len]));
    }

    println!("connection closed");
}

fn run_client() {
    let mut buf = vec![0u8; 65535];

    // initialize initiator
    let builder: Builder<'_> = Builder::new(PARAMS.clone());
    let static_key = builder.generate_keypair().unwrap().private;
    let mut noise = builder
        .local_private_key(&static_key)
        .psk(3, SECRET)
        .build_initiator()
        .unwrap();

    // connect to server
    let mut stream = TcpStream::connect("127.0.0.1:9999").unwrap();
    println!("connected!");

    // -> e
    let len = noise.write_message(&[], &mut buf).unwrap();
    send(&mut stream, &buf[..len]);

    // <- e, ee, s, es
    noise
        .read_message(&recv(&mut stream).unwrap(), &mut buf)
        .unwrap();

    // -> s, se
    let len = noise.write_message(&[], &mut buf).unwrap();
    send(&mut stream, &buf[..len]);

    let mut noise = noise.into_transport_mode().unwrap();
    println!("Session established...");

    // send secure data
    for _ in 0..10 {
        let len = noise.write_message(b"HACK THE PLANET", &mut buf).unwrap();
        send(&mut stream, &buf[..len]);
    }

    println!("done!");
}

fn recv(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut msg_len_buf = [0u8; 2];
    stream.read_exact(&mut msg_len_buf)?;
    let msg_len = ((msg_len_buf[0] as usize) << 8) + (msg_len_buf[1] as usize);
    let mut msg = vec![0u8; msg_len];
    stream.read_exact(&mut msg[..])?;
    Ok(msg)
}

fn send(stream: &mut TcpStream, buf: &[u8]) {
    let msg_len_buf = [(buf.len() >> 8) as u8, (buf.len() & 0xff) as u8];
    stream.write_all(&msg_len_buf).unwrap();
    stream.write_all(buf).unwrap();
}
