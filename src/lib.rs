#![feature(cstr_to_str)]
#![feature(into_raw_os)]

extern crate libudt4_sys as raw;

use std::sync::{Once, ONCE_INIT};
extern crate libc;

use libc::{AF_INET, AF_INET6};
use libc::{SOCK_STREAM, SOCK_DGRAM};
use libc::{c_int};
use std::mem::size_of;
use libc::{sockaddr, sockaddr_in, in_addr};
use std::ffi::{CStr};
use std::net::SocketAddr;
use std::net::SocketAddrV4;


// makes defining the UdtOpts mod a little less messy
macro_rules! impl_udt_opt {
    ($(#[$doc:meta])*
     impl $name:ident: $ty:ty) => {
         $(#[$doc])*
        pub struct $name;
        impl ::UdtOption<$ty> for $name {
            fn get_type(&self) -> ::raw::UDTOpt { ::raw::UDTOpt::$name }
        }
    };
}

pub fn init() {
    static INIT: Once = ONCE_INIT;
        INIT.call_once(|| unsafe {
            println!("did INIT");
            raw::udt_startup();
            assert_eq!(libc::atexit(shutdown), 0);
        });
    extern fn shutdown() {
        unsafe { raw::udt_cleanup(); } ;
    }
}

/// A UDT Socket
#[derive(Debug)]
pub struct UdtSocket {
    _sock: raw::UDTSOCKET, 
}

#[derive(Debug)]
pub struct UdtError {
    pub err_code: i32,
    pub err_msg: String
}

pub trait UdtOption<T> {
    fn get_type(&self) -> raw::UDTOpt;
}

#[repr(C)]
/// Linger option
pub struct Linger {
    /// Nonzero to linger on close
    onoff: i32,
    /// Time to longer
    linger: i32
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod UdtOpts {
    //! Various options that can be passed to `getsockopt` or `setsockopt`

    impl_udt_opt!{
        /// Maximum Packet size (bytes)
        ///
        /// Including all UDT, UDP, and IP headers.  Default 1500 bytes
        impl UDT_MSS: i32
    }
    
    impl_udt_opt!{
        /// Synchronization mode of data sending
        ///
        /// True for blocking sending; false for non-blocking sending.  Default true
        impl UDT_SNDSYN: bool
    }

    impl_udt_opt! {
        /// Synchronization mode for receiving.
        /// 
        /// true for blocking receiving; false for non-blocking
        /// receiving. Default true.
        impl UDT_RCVSYN: bool
    }


    // MISSING: UDT_CC for custom congestion control

    impl_udt_opt! {
        ///Maximum window size (packets)
        ///
        ///Default 25600. Do NOT change this unless you know what you are doing. Must change this
        ///before modifying the buffer sizes.
        impl UDT_FC: i32
    }

    impl_udt_opt!(
        /// UDT sender buffer size limit (bytes)
        ///
        /// Default 10MB (10240000).
        impl UDT_SNDBUF: i32);

    impl_udt_opt!(
        /// UDT receiver buffer size limit (bytes)
        ///
        /// Default 10MB (10240000).
        impl UDT_RCVBUF: i32);

    impl_udt_opt!(///UDP socket sender buffer size (bytes)
        ///
        /// Default 1MB (1024000).
        impl UDP_SNDBUF: i32);
    impl_udt_opt!(/// UDP socket receiver buffer size (bytes)
        ///
        /// Default 1MB (1024000).
        impl UDP_RCVBUF: i32);
    impl_udt_opt!(/// Linger time on close().
        ///
        /// Default 180 seconds.
        impl UDT_LINGER: ::Linger);
    impl_udt_opt!(/// Rendezvous connection setup.
        ///
        /// Default false (no rendezvous mode).
        impl UDT_RENDEZVOUS: bool);
    impl_udt_opt!(/// Sending call timeout (milliseconds).
        ///
        /// Default -1 (infinite).
        impl UDT_SNDTIMEO: i32);
    impl_udt_opt!(/// Receiving call timeout (milliseconds).
        ///
        /// Default -1 (infinite).
        impl UDT_RCVTIMEO: i32);
    impl_udt_opt!(/// Reuse an existing address or create a new one.
        /// 
        /// Default true (reuse).
        impl UDT_REUSEADDR: bool);
    impl_udt_opt!(/// Maximum bandwidth that one single UDT connection can use (bytes per second).
        ///
        /// Default -1 (no upper limit).
        impl UDT_MAXBW: i64);
    impl_udt_opt!(/// Current status of the UDT socket. Read only.
        impl UDT_STATE: i32);
    impl_udt_opt!(/// The EPOLL events available to this socket.    Read only.
        impl UDT_EVENT: i32);
    impl_udt_opt!(/// Size of pending data in the sending buffer.   Read only.
        impl UDT_SNDDATA: i32);
    impl_udt_opt!(/// Size of data available to read, in the receiving buffer.  Read only.
        impl UDT_RCVDATA: i32);
    

}


fn get_last_err() -> UdtError {
    let msg = unsafe{ CStr::from_ptr(raw::udt_getlasterror_desc()) };
    UdtError{err_code: unsafe{ raw::udt_getlasterror_code() as i32},
    err_msg: msg.to_string_lossy().into_owned()}
}

#[repr(C)]
pub enum SocketFamily {
    /// IPv4
    AFInet,
    /// IPV6
    AFInet6
}

impl SocketFamily {
    fn get_val(&self) -> c_int { match *self {
        SocketFamily::AFInet => AF_INET,
        SocketFamily::AFInet6 => AF_INET6
    }}
}

/// Socket type
///
/// When a UDT socket is created as a Datagram type, UDT will send and receive data as messages.
/// The boundary of the message is preserved and the message is delivered as a whole unit. Sending
/// or receving messages do not need a loop; a message will be either completely delivered or not
/// delivered at all. However, at the receiver side, if the user buffer is shorter than the message
/// length, only part of the message will be copied into the user buffer while the message will
/// still be discarded.
///
///
#[repr(C)]
pub enum SocketType {
    /// A socket type that supports data streaming
    Stream = SOCK_STREAM as isize,
    /// A socket type for messaging
    ///
    /// Note that UDT Datagram sockets are also connection oriented.  A UDT connection can only be
    /// set up between the same socket types
    Datagram = SOCK_DGRAM as isize
}

impl SocketType {
    fn get_val(&self) -> c_int { match *self {
        SocketType::Stream => SOCK_STREAM,
        SocketType::Datagram => SOCK_DGRAM
    }}
}


// SocketAddr to sockaddr_in
fn get_sockaddr(name: SocketAddr) -> sockaddr_in {
    if let SocketAddr::V4(v4) = name {
        println!("binding to {:?}", v4);
        let addr_bytes = v4.ip().octets();
        let addr_b: u32 = ((addr_bytes[3] as u32) << 24)  + 
            ((addr_bytes[2] as u32) << 16)  + 
            ((addr_bytes[1] as u32) << 8 )  + 
            ( addr_bytes[0] as u32);
        // construct a sockaddr_in
         sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: v4.port().to_be(),
            sin_addr: in_addr{s_addr: addr_b},
            sin_zero: [0; 8]
      }
    } else {
        panic!("ipv6 not implemented (yet) in this binding");
    }
}
   
// sockaddr_to_SocketAddr
fn sockaddr_to_socketaddr(s: sockaddr) -> SocketAddr {
    let fam: i32 = s.sa_family as i32;

    match fam {
        AF_INET => {
            let name1: sockaddr_in = unsafe{ std::mem::transmute(s) };
            let ip: u32 = name1.sin_addr.s_addr;
            let d: u8 = ((ip & 0xff000000) >> 24) as u8;
            let c: u8 = ((ip & 0xff0000) >> 16) as u8;
            let b: u8 = ((ip & 0xff00) >> 8) as u8;
            let a: u8 = ((ip & 0xff)) as u8;
            SocketAddr::V4(SocketAddrV4::new(
                        std::net::Ipv4Addr::new(a, b, c, d),
                        u16::from_be(name1.sin_port)
                        ))
        },
        AF_INET6 => {
            panic!("ipv6 not yet implemented")
        },
        _ => panic!("unknown family type")
    }
}



impl UdtSocket {
    fn wrap_raw(u: raw::UDTSOCKET) -> UdtSocket {
        UdtSocket{_sock: u}
    }
    
    /// Creates a new UDT Socket.
    ///
    /// Creates a new socket.  There is no limits for the number of UDT sockets in one system, as
    /// long as there is enough system resources.  UDT supports both IPv4 and IPv6, which can be
    /// selected by the `address_family` parameter. 
    ///
    /// Two socket types are supported in UDT:  Stream for data streaming and Datagram for
    /// messaging.  Note that UDT sockets are connection oriented in all cases.
    ///
    pub fn new(address_family: SocketFamily, ty: SocketType) -> Result<UdtSocket, UdtError> {

        let fd = unsafe {
            raw::udt_socket(address_family.get_val(),
                       ty.get_val(),
                       0)
        };
        if fd == raw::INVALID_SOCK {
            Err(get_last_err())
        } else {
            Ok(UdtSocket{_sock: fd})
        }
    }

    /// The bind method binds a UDT socket to a known or an available local address.
    ///
    /// The bind method is usually to assign a UDT socket a local address, including IP address and
    /// port number. If INADDR_ANY is used, a proper IP address will be used once the UDT
    /// connection is set up. If 0 is used for the port, a randomly available port number will be
    /// used. The method getsockname can be used to retrieve this port number.
    ///
    /// The second form of bind allows UDT to bind directly on an existing UDP socket. This is
    /// usefule for firewall traversing in certain situations: 1) a UDP socket is created and its
    /// address is learned from a name server, there is no need to close the UDP socket and open a
    /// UDT socket on the same address again; 2) for certain firewall, especially some on local
    /// system, the port mapping maybe changed or the "hole" may be closed when a UDP socket is
    /// closed and reopened, thus it is necessary to use the UDP socket directly in UDT.
    ///
    /// Use the second form of bind with caution, as it violates certain programming rules
    /// regarding code robustness. Once the UDP socket descriptor is passed to UDT, it MUST NOT be
    /// touched again. DO NOT use this unless you clearly understand how the related systems work.
    ///
    /// The bind call is necessary in all cases except for a socket to listen. If bind is not
    /// called, UDT will automatically bind a socket to a randomly available address when a
    /// connection is set up.
    ///
    /// By default, UDT allows to reuse existing UDP port for new UDT sockets, unless UDT_REUSEADDR
    /// is set to false. When UDT_REUSEADDR is false, UDT will create an exclusive UDP port for
    /// this UDT socket. UDT_REUSEADDR must be called before bind. To reuse an existing UDT/UDP
    /// port, the new UDT socket must explicitly bind to the port. If the port is already used by a
    /// UDT socket with UDT_REUSEADDR as false, the new bind will return error. If 0 is passed as
    /// the port number, bind always creates a new port, no matter what value the UDT_REUSEADDR
    /// sets.
    ///
    /// # Returns
    ///
    /// If the binding is successful, bind returns 0, otherwise it returns UDT::ERROR and the
    /// specific error information can be retrieved using getlasterror.
    pub fn bind(&mut self, name: std::net::SocketAddr) -> Result<(), UdtError> {

        let addr: sockaddr_in = get_sockaddr(name); 
        let ret = unsafe {
            raw::udt_bind(self._sock, 
                          std::mem::transmute(&addr),
                          size_of::<sockaddr_in>() as i32
                         )
        };
        if ret == raw::SUCCESS {
            Ok(())
        } else {
            Err(get_last_err())
        }
    }

    pub fn bind_from(&mut self, other: std::net::UdpSocket) -> Result<(), UdtError> {
        use std::os::unix::io::IntoRawFd;
        let ret = unsafe {
            raw::udt_bind2(self._sock,
                          other.into_raw_fd())
        };
        if ret == raw::SUCCESS {
            Ok(())
        } else {
            Err(get_last_err())
        }
    }

    pub fn connect(&mut self, name: std::net::SocketAddr) -> Result<(), UdtError> {
        let addr = get_sockaddr(name);
        let ret = unsafe {
            raw::udt_connect(self._sock,
                             std::mem::transmute(&addr),
                             size_of::<sockaddr_in>() as i32)
        };
        println!("connect returned  {:?}", ret);
        if ret == raw::SUCCESS {
            Ok(())
        } else {
            Err(get_last_err())
        }

    }

    pub fn listen(&mut self) -> Result<(), UdtError> {
        let ret = unsafe { raw::udt_listen(self._sock, 5) };

        if ret == raw::SUCCESS {
            Ok(())
        } else {
            Err(get_last_err())
        }
    }

    pub fn accept(&mut self) -> Result<UdtSocket, UdtError> {
        use std::ptr;
        let ret = unsafe { raw::udt_accept(self._sock, ptr::null_mut(), ptr::null_mut()) };
        if ret == raw::INVALID_SOCK {
            Err(get_last_err())
        } else {
            Ok(UdtSocket::wrap_raw(ret))
        }
    }

    pub fn close(self) -> Result<(), UdtError> {
        let ret = unsafe { raw::udt_close(self._sock) };
        if ret == raw::SUCCESS {
            Ok(())
        } else {
            Err(get_last_err())
        }
    }


    pub fn getpeername(&mut self) -> Result<std::net::SocketAddr, UdtError> {
        let mut name = sockaddr { sa_family: 0, sa_data: [0; 14]};
        let mut size: i32 = size_of::<sockaddr>() as i32;
        let ret = unsafe { raw::udt_getpeername(self._sock,&mut name, &mut size) };
        assert_eq!(size as usize, size_of::<sockaddr>());
        if ret != raw::SUCCESS {
            Err(get_last_err())
        } else {
            Ok(sockaddr_to_socketaddr(name))
        }
    }
   
    /// Retrieves the local address associated with a UDT socket.
    ///
    /// The getsockname retrieves the local address associated with the socket. The UDT socket must
    /// be bound explicitly (via bind) or implicitly (via connect), otherwise this method will fail
    /// because there is no meaningful address bound to the socket.
    ///
    /// If getsockname is called after an explicit bind, but before connect, the IP address
    /// returned will be exactly the IP address that is used for bind and it may be 0.0.0.0 if
    /// ADDR_ANY is used. If getsockname is called after connect, the IP address returned will be
    /// the address that the peer socket sees. In the case when there is a proxy (e.g., NAT), the
    /// IP address returned will be the translated address by the proxy, but not a local address.
    /// If there is no proxy, the IP address returned will be a local address. In either case, the
    /// port number is local (i.e, not the translated proxy port).
    ///
    /// Because UDP is connection-less, using getsockname on a UDP port will almost always return
    /// 0.0.0.0 as IP address (unless it is bound to an explicit IP) . As a connection oriented
    /// protocol, UDT will return a meaningful IP address by getsockname if there is no proxy
    /// translation exist.
    ///
    /// UDT has no multihoming support yet. When there are multiple local addresses and more than
    /// one of them can be routed to the destination address, UDT may not behave properly due to
    /// the multi-path effect. In this case, the UDT socket must be explicitly bound to one of
    /// the local addresses.
    pub fn getsockname(&mut self) -> Result<std::net::SocketAddr, UdtError> {
        let mut name = sockaddr { sa_family: 0, sa_data: [0; 14]};
        let mut size: i32 = size_of::<sockaddr>() as i32;
        let ret = unsafe { raw::udt_getsockname(self._sock,&mut name, &mut size) };

        assert_eq!(size as usize, size_of::<sockaddr>());
        if ret != raw::SUCCESS {
            Err(get_last_err())
        } else {
            Ok(sockaddr_to_socketaddr(name))
        }

    }

    /// Sends a message to the peer side.
    ///
    /// The sendmsg method sends a message to the peer side. The UDT socket must be in SOCK_DGRAM
    /// mode in order to send or receive messages. Message is the minimum data unit in this
    /// situation. In particular, sendmsg always tries to send the message out as a whole, that is,
    /// the message will either to completely sent or it is not sent at all.
    ///
    /// In blocking mode (default), sendmsg waits until there is enough space to hold the whole
    /// message. In non-blocking mode, sendmsg returns immediately and returns error if no buffer
    /// space available.
    ///
    /// If UDT_SNDTIMEO is set and the socket is in blocking mode, sendmsg only waits a limited
    /// time specified by UDT_SNDTIMEO option. If there is still no buffer space available when the
    /// timer expires, error will be returned. UDT_SNDTIMEO has no effect for non-blocking socket.
    ///
    /// The ttl parameter gives the message a limited life time, which starts counting once the
    /// first packet of the message is sent out. If the message has not been delivered to the
    /// receiver after the TTL timer expires and each packet in the message has been sent out at
    /// least once, the message will be discarded. Lost packets in the message will be
    /// retransmitted before TTL expires.
    ///
    /// On the other hand, the inorder option decides if this message should be delivered in order.
    /// That is, the message should not be delivered to the receiver side application unless all
    /// messages prior to it are either delivered or discarded.
    ///
    /// Finally, if the message size is greater than the size of the receiver buffer, the message
    /// will never be received in whole by the receiver side. Only the beginning part that can be
    /// hold in the receiver buffer may be read and the rest will be discarded.
    ///
    /// # Returns
    ///
    /// On success, sendmsg returns the actual size of message that has just been sent. The size
    /// should be equal to len. Otherwise UDT::ERROR is returned and specific error information can
    /// be retrieved by getlasterror. If UDT_SNDTIMEO is set to a positive value, zero will be
    /// returned if the message cannot be sent before the timer expires.
    pub fn sendmsg(&mut self, buf: &[u8]) -> Result<i32, UdtError> {

        let ret = unsafe {
            raw::udt_sendmsg(self._sock, 
                             buf.as_ptr(),
                             buf.len() as i32,
                             -1 as i32,
                             1 as i32)
        };
        if ret == raw::UDT_ERROR {
            Err(get_last_err())
        } else {
            Ok(ret)
        }
    }


    /// Sends out a certain amount of data from an application buffer.
    ///
    /// The send method sends certain amount of data from the application buffer. If the the size
    /// limit of sending buffer queue is reached, send only sends a portion of the application
    /// buffer and returns the actual size of data that has been sent.
    ///
    /// In blocking mode (default), send waits until there is some sending buffer space available.
    /// In non-blocking mode, send returns immediately and returns error if the sending queue limit
    /// is already limited.
    ///
    /// If UDT_SNDTIMEO is set and the socket is in blocking mode, send only waits a limited time
    /// specified by UDT_SNDTIMEO option. If there is still no buffer space available when the
    /// timer expires, error will be returned. UDT_SNDTIMEO has no effect for non-blocking socket.
    ///
    /// # Returns
    ///
    /// On success, returns the actual size of the data that as been sent.  Otherwise, a UdtError
    /// is returned with specific error information.
    ///
    /// If UDT_SNDTIMEO is set to a positive value, zero will be returned if no data is sent before
    /// the time expires.
    pub fn send(&mut self, buf: &[u8]) -> Result<i32, UdtError> {

        let ret = unsafe { raw::udt_send(self._sock, buf.as_ptr(), buf.len() as i32, 0) };
        if ret == raw::UDT_ERROR {
            Err(get_last_err())
        } else {
            Ok(ret)
        }


    }

    /// The recvmsg method receives a valid message.
    ///
    /// The recvmsg method reads a message from the protocol buffer. The UDT socket must be in
    /// SOCK_DGRAM mode in order to send or receive messages. Message is the minimum data unit in
    /// this situation. Each recvmsg will read no more than one message, even if the message is
    /// smaller than the size of buf and there are more messages available. On the other hand, if
    /// the buf is not enough to hold the first message, only part of the message will be copied
    /// into the buffer, but the message will still be discarded after this recvmsg call.
    ///
    /// In blocking mode (default), recvmsg waits until there is a valid message received into the
    /// receiver buffer. In non-blocking mode, recvmsg returns immediately and returns error if no
    /// message available.
    ///
    /// If UDT_RCVTIMEO is set and the socket is in blocking mode, recvmsg only waits a limited
    /// time specified by UDT_RCVTIMEO option. If there is still no message available when the
    /// timer expires, error will be returned. UDT_RCVTIMEO has no effect for non-blocking socket.
    ///
    /// # Returns
    ///
    /// On success, recvmsg returns the actual size of received message. Otherwise UDT::ERROR is
    /// returned and specific error information can be retrieved by getlasterror. If UDT_RCVTIMEO
    /// is set to a positive value, zero will be returned if no message is received before the
    /// timer expires.
    pub fn recvmsg(&mut self, len: usize) -> Result<Vec<u8>, UdtError> {
        let mut v: Vec<u8> = Vec::with_capacity(len);
        v.extend(std::iter::repeat(0 as u8).take(len).collect::<Vec<u8>>());
        let ret = unsafe {
            raw::udt_recvmsg(self._sock, 
                             v.as_mut_ptr(),
                             len as i32)
        };
        if ret > 0 {
            v.truncate(ret as usize);
            Ok(v)
        } else {
            Err(get_last_err())
        }
    }

    pub fn recv(&mut self, buf: &mut [u8], len: usize) -> Result<i32, UdtError> {
        let ret = unsafe {
            raw::udt_recv(self._sock, buf.as_mut_ptr(), len as i32, 0)
        };

        if ret == raw::UDT_ERROR {
            Err(get_last_err())
        } else {
            Ok(ret)
        }

    }

    pub fn getsockopt<B: Default, T: UdtOption<B>>(&mut self, opt: T) -> Result<B, UdtError> {
        let mut val: B = unsafe{ std::mem::zeroed() };
        let val_p: *mut B = &mut val;
        let ty: raw::UDTOpt = opt.get_type();
        let mut size: c_int = size_of::<B>() as i32;
        let ret = unsafe { raw::udt_getsockopt(self._sock, 0, ty,
                                               std::mem::transmute(val_p), &mut size) };
        
        if ret == raw::SUCCESS {
            Ok(val)
        } else {
            Err(get_last_err())
        }

    }

    pub fn setsockopt<B, T: UdtOption<B>>(&mut self, opt: T, value: B) -> Result<(), UdtError> {
        let ty: raw::UDTOpt = opt.get_type();
        let val_p: *const B = &value;
        let size: c_int = size_of::<B>() as i32;

        let ret = unsafe {
            raw::udt_setsockopt(self._sock, 0, ty,
                                std::mem::transmute(val_p), size)
        };
        
        if ret == raw::SUCCESS {
            Ok(())
        } else {
            Err(get_last_err())
        }

    }
}

