#![feature(ip_addr)]

extern crate udt_rs;

use udt_rs::*;


#[test]
fn test_getsockopt() {
    init();
    let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();

    // test some defaults
    assert_eq!(sock.getsockopt(UdtOpts::UDT_MSS).unwrap(), 1500 as i32);
    assert_eq!(sock.getsockopt(UdtOpts::UDT_SNDSYN).unwrap(), true);
    assert_eq!(sock.getsockopt(UdtOpts::UDT_RCVSYN).unwrap(), true);
    assert_eq!(sock.getsockopt(UdtOpts::UDT_FC).unwrap(), 25600 as i32);
    assert_eq!(sock.getsockopt(UdtOpts::UDT_RENDEZVOUS).unwrap(), false);
    assert_eq!(sock.getsockopt(UdtOpts::UDT_SNDTIMEO).unwrap(), -1);
    assert_eq!(sock.getsockopt(UdtOpts::UDT_RCVTIMEO).unwrap(), -1);

}

#[test]
fn test_setsockopt() {
    init();
    let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();

    assert_eq!(sock.getsockopt(UdtOpts::UDT_MSS).unwrap(), 1500);
    sock.setsockopt(UdtOpts::UDT_MSS, 1400).unwrap();
    assert_eq!(sock.getsockopt(UdtOpts::UDT_MSS).unwrap(), 1400);
}




#[test]
fn test_sendmsg() {
    use std::thread::spawn;
    use std::net::SocketAddr;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::mpsc::channel;

    init();

    let localhost = IpAddr::from_str("127.0.0.1").unwrap();

    // the server will bind to a random port and pass it back for the client to connect to
    let (tx, rx) = channel();

    // spawn the server
    let server = spawn(move || {
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
        sock.bind(SocketAddr::new(localhost, 0)).unwrap();
        let my_addr = sock.getsockname().unwrap();
        println!("Server bound to {:?}", my_addr);

        sock.listen().unwrap();

        tx.send(my_addr.port()); 

        let mut new = sock.accept().unwrap();

        let msg = new.recvmsg(100).unwrap();
        assert_eq!(msg.len(), 5);
        assert_eq!(msg, "hello".as_bytes());
        new.sendmsg("world".as_bytes()).unwrap();

        new.close();
        sock.close();


    });

    let client = spawn(move || {
        let port = rx.recv().unwrap();
        println!("Client connecting to port {:?}", port);
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
        sock.connect(SocketAddr::new(localhost, port)).unwrap();

        sock.sendmsg("hello".as_bytes()).unwrap();
        let msg = sock.recvmsg(1024).unwrap();
        assert_eq!(msg.len(), 5);
        assert_eq!(msg, "world".as_bytes());

        sock.close();


    });

    server.join();
    client.join();



}


#[test]
fn test_send() {
    use std::thread::spawn;
    use std::net::SocketAddr;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::mpsc::channel;

    init();

    let localhost = IpAddr::from_str("127.0.0.1").unwrap();

    // the server will bind to a random port and pass it back for the client to connect to
    let (tx, rx) = channel();

    // spawn the server
    let server = spawn(move || {
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Stream).unwrap();
        sock.bind(SocketAddr::new(localhost, 0)).unwrap();
        let my_addr = sock.getsockname().unwrap();
        println!("Server bound to {:?}", my_addr);

        sock.listen().unwrap();

        tx.send(my_addr.port()); 

        let mut new = sock.accept().unwrap();

        let mut buf: [u8; 10] = [0; 10];
        assert_eq!(new.recv(&mut buf, 5).unwrap(), 5);
        assert_eq!(&buf[0..5], "hello".as_bytes());
        assert_eq!(&buf[5..], [0; 5]);
        assert_eq!(new.recv(&mut buf[5..], 5).unwrap(), 5);
        assert_eq!(&buf[5..], "world".as_bytes());
        assert_eq!(&buf, "helloworld".as_bytes());

        new.close();
        sock.close();


    });

    let client = spawn(move || {
        let port = rx.recv().unwrap();
        println!("Client connecting to port {:?}", port);
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Stream).unwrap();
        sock.connect(SocketAddr::new(localhost, port)).unwrap();

        assert_eq!(sock.send("hello".as_bytes()).unwrap(), 5);
        assert_eq!(sock.send("world".as_bytes()).unwrap(), 5);


        sock.close();


    });

    server.join();
    client.join();


}
