extern crate udt;
#[macro_use]
extern crate log;

use udt::*;

#[cfg(target_os="linux")]
#[allow(unused_variables)]
fn do_platform_specific_init(sock: &mut UdtSocket) {}

#[cfg(target_os="macos")]
fn do_platform_specific_init(sock: &mut UdtSocket) {
    sock.setsockopt(UdtOpts::UDP_RCVBUF, 8192).unwrap();
    sock.setsockopt(UdtOpts::UDP_SNDBUF, 8192).unwrap();
}


#[test]
fn test_getsockopt() {
    init();
    let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
    do_platform_specific_init(&mut sock);

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
    do_platform_specific_init(&mut sock);

    assert_eq!(sock.getsockopt(UdtOpts::UDT_MSS).unwrap(), 1500);
    sock.setsockopt(UdtOpts::UDT_MSS, 1400).unwrap();
    assert_eq!(sock.getsockopt(UdtOpts::UDT_MSS).unwrap(), 1400);
}




#[test]
fn test_sendmsg() {
    use std::thread::spawn;
    use std::net::{SocketAddr, SocketAddrV4};
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use std::sync::mpsc::channel;

    init();

    let localhost = Ipv4Addr::from_str("127.0.0.1").unwrap();

    // the server will bind to a random port and pass it back for the client to connect to
    let (tx, rx) = channel();

    // spawn the server
    let server = spawn(move || {
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
        do_platform_specific_init(&mut sock);
        sock.bind(SocketAddr::V4(SocketAddrV4::new(localhost, 0))).unwrap();
        let my_addr = sock.getsockname().unwrap();
        debug!("Server bound to {:?}", my_addr);

        sock.listen(5).unwrap();

        tx.send(my_addr.port()).unwrap();

        let (new, peer) = sock.accept().unwrap();
        debug!("Server recieved connection from {:?}", peer);

        let peer2 = new.getpeername().unwrap();
        assert_eq!(peer2, peer);


        let msg = new.recvmsg(100).unwrap();
        assert_eq!(msg.len(), 5);
        assert_eq!(msg, "hello".as_bytes());
        new.sendmsg("world".as_bytes()).unwrap();

        new.close().unwrap();
        sock.close().unwrap();


    });

    let client = spawn(move || {
        let port = rx.recv().unwrap();
        debug!("Client connecting to port {:?}", port);
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
        do_platform_specific_init(&mut sock);
        sock.connect(SocketAddr::V4(SocketAddrV4::new(localhost, port))).unwrap();

        sock.sendmsg("hello".as_bytes()).unwrap();
        let msg = sock.recvmsg(1024).unwrap();
        assert_eq!(msg.len(), 5);
        assert_eq!(msg, "world".as_bytes());

        sock.close().unwrap();


    });

    server.join().unwrap();
    client.join().unwrap();



}


#[test]
fn test_send() {
    use std::thread::spawn;
    use std::net::{SocketAddr, SocketAddrV4};
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use std::sync::mpsc::channel;

    init();

    let localhost = Ipv4Addr::from_str("127.0.0.1").unwrap();

    // the server will bind to a random port and pass it back for the client to connect to
    let (tx, rx) = channel();

    // spawn the server
    let server = spawn(move || {
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Stream).unwrap();
        do_platform_specific_init(&mut sock);
        sock.bind(SocketAddr::V4(SocketAddrV4::new(localhost, 0))).unwrap();
        let my_addr = sock.getsockname().unwrap();
        debug!("Server bound to {:?}", my_addr);

        sock.listen(5).unwrap();

        tx.send(my_addr.port()).unwrap();

        let (new, new_peer) = sock.accept().unwrap();
        debug!("Server recieved connection from {:?}", new_peer);

        let mut buf: [u8; 10] = [0; 10];
        assert_eq!(new.recv(&mut buf, 5).unwrap(), 5);
        assert_eq!(&buf[0..5], "hello".as_bytes());
        assert_eq!(&buf[5..], [0; 5]);
        assert_eq!(new.recv(&mut buf[5..], 5).unwrap(), 5);
        assert_eq!(&buf[5..], "world".as_bytes());
        assert_eq!(&buf, "helloworld".as_bytes());

        new.close().unwrap();
        sock.close().unwrap();


    });

    let client = spawn(move || {
        let port = rx.recv().unwrap();
        debug!("Client connecting to port {:?}", port);
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Stream).unwrap();
        do_platform_specific_init(&mut sock);
        sock.connect(SocketAddr::V4(SocketAddrV4::new(localhost, port))).unwrap();

        assert_eq!(sock.send("hello".as_bytes()).unwrap(), 5);
        assert_eq!(sock.send("world".as_bytes()).unwrap(), 5);


        sock.close().unwrap();


    });

    server.join().unwrap();
    client.join().unwrap();


}


#[test]
fn test_epoll() {
    use std::thread::spawn;
    use std::net::{SocketAddr, SocketAddrV4};
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use std::sync::mpsc::channel;
    use std::thread::sleep;
    use std::time::Duration;

    init();

    let localhost = Ipv4Addr::from_str("127.0.0.1").unwrap();

    // the server will bind to a random port and pass it back for the client to connect to
    let (tx, rx) = channel();

    // spawn the server
    let server = spawn(move || {
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
        do_platform_specific_init(&mut sock);
        sock.bind(SocketAddr::V4(SocketAddrV4::new(localhost, 0))).unwrap();
        let my_addr = sock.getsockname().unwrap();
        debug!("Server bound to {:?}", my_addr);

        sock.listen(5).unwrap();

        tx.send(my_addr.port()).unwrap();

        let mut epoll = Epoll::create().unwrap();

        epoll.add_usock(&sock, None).unwrap();

        let mut counter = 0;
        loop { 
            let (pending_rd, pending_wr) = epoll.wait(1000, true).unwrap();
            debug!("Pending sockets: {:?} {:?}", pending_rd, pending_wr);
            
            let rd_len = pending_rd.len();
            for s in pending_rd {
                if s == sock {
                    debug!("trying to accept new sock");
                    let (new, peer) = sock.accept().unwrap();
                    debug!("Server recieved connection from {:?}", peer);
                    epoll.add_usock(&new, None).unwrap();
                } else {
                    let msg = s.recvmsg(100).unwrap();
                    let msg_string = String::from_utf8(msg).unwrap();
                    debug!("Received message: {:?}", msg_string);
                }

            }

            for s in pending_wr {
                let state = s.getstate();
                if rd_len == 0 && (state == UdtStatus::BROKEN || state == UdtStatus::CLOSED || state == UdtStatus::NONEXIST) {
                    epoll.remove_usock(&s).unwrap();
                    return;
                }
                debug!("Sock {:?} is in state {:?}", s, state);
            }
            sleep(Duration::from_millis(100));
            counter += 1;
            assert!(counter < 500);
        }


    });

    let client = spawn(move || {
        let port = rx.recv().unwrap();
        debug!("Client connecting to port {:?}", port);
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
        do_platform_specific_init(&mut sock);
        sock.connect(SocketAddr::V4(SocketAddrV4::new(localhost, port))).unwrap();

        sock.sendmsg("hello".as_bytes()).unwrap();

        sleep(Duration::from_millis(3000));
        sock.sendmsg("world".as_bytes()).unwrap();
        sock.sendmsg("done.".as_bytes()).unwrap();

        sock.close().unwrap();


    });

    server.join().unwrap();
    client.join().unwrap();



}

#[test]
fn test_epoll2() {
    use std::thread::spawn;
    use std::net::{SocketAddr, SocketAddrV4};
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use std::sync::mpsc::channel;
    use std::thread::sleep;
    use std::time::Duration;

    init();

    let localhost = Ipv4Addr::from_str("127.0.0.1").unwrap();

    // the server will bind to a random port and pass it back for the client to connect to
    let (tx, rx) = channel();

    // spawn the server
    let server = spawn(move || {
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
        do_platform_specific_init(&mut sock);
        sock.bind(SocketAddr::V4(SocketAddrV4::new(localhost, 0))).unwrap();
        let my_addr = sock.getsockname().unwrap();
        println!("Server bound to {:?}", my_addr);

        sock.listen(5).unwrap();

        tx.send(my_addr.port()).unwrap();

        let mut epoll = Epoll::create().unwrap();

        epoll.add_usock(&sock, Some(UDT_EPOLL_ERR | UDT_EPOLL_IN)).unwrap();

        let mut counter = 0;
        let mut outer = true;
        while outer { 
            let (pending_rd, pending_wr) = epoll.wait(1000, true).unwrap();
            println!("Pending sockets: {:?} {:?}", pending_rd, pending_wr);
            
            let rd_len = pending_rd.len();
            for s in pending_rd {
                if s == sock {
                    println!("trying to accept new sock");
                    let (new, peer) = sock.accept().unwrap();
                    println!("Server recieved connection from {:?}", peer);
                    epoll.add_usock(&new, Some(UDT_EPOLL_ERR | UDT_EPOLL_IN)).unwrap();
                } else {
                    if let Ok(msg) = s.recvmsg(100) {
                        let msg_string = String::from_utf8(msg).unwrap();
                        println!("Received message: {:?}", msg_string);
                    } else {
                        println!("Error on recieve, removing usock");
                        epoll.remove_usock(&s).unwrap();
                        outer = false;
                    }
                }

            }

            for s in pending_wr {
                let state = s.getstate();
                println!("state: {:?}", state);
                if rd_len == 0 && (state == UdtStatus::BROKEN || state == UdtStatus::CLOSED || state == UdtStatus::NONEXIST) {
                    epoll.remove_usock(&s).unwrap();
                    return;
                }
                println!("Sock {:?} is in state {:?}", s, state);
            }
            sleep(Duration::from_millis(100));
            counter += 1;
            assert!(counter < 500);
        }


    });

    let client = spawn(move || {
        let port = rx.recv().unwrap();
        debug!("Client connecting to port {:?}", port);
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
        do_platform_specific_init(&mut sock);
        sock.connect(SocketAddr::V4(SocketAddrV4::new(localhost, port))).unwrap();

        sock.sendmsg("hello".as_bytes()).unwrap();

        sleep(Duration::from_millis(3000));
        sock.sendmsg("world".as_bytes()).unwrap();
        sock.sendmsg("done.".as_bytes()).unwrap();

        sock.close().unwrap();
        println!("Client is done");


    });

    server.join().unwrap();
    client.join().unwrap();



}

#[test]
fn test_epoll3() {
    use std::thread::spawn;
    use std::net::{SocketAddr, SocketAddrV4};
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    init();

    let localhost = Ipv4Addr::from_str("127.0.0.1").unwrap();

    // spawn the server
    let server = spawn(move || {
        let mut sock = UdtSocket::new(SocketFamily::AFInet, SocketType::Datagram).unwrap();
        do_platform_specific_init(&mut sock);
        sock.bind(SocketAddr::V4(SocketAddrV4::new(localhost, 0))).unwrap();
        let my_addr = sock.getsockname().unwrap();
        println!("Server bound to {:?}", my_addr);

        sock.listen(5).unwrap();

        let mut epoll = Epoll::create().unwrap();
        println!("Epoll {:?} created", epoll);

        epoll.add_usock(&sock, None).unwrap();
        epoll.remove_usock(&sock).unwrap();
        epoll.add_usock(&sock, None).unwrap();
    });
    server.join().unwrap();



}
