#![allow(unused)]

use std::os::unix::prelude::AsRawFd;

use tokio::io::AsyncReadExt;

/*
#[tokio::main(flavor="current_thread")]
async fn main() {
    use tokio::net::{TcpListener, TcpStream};
    let args : Vec<_> = std::env::args().collect();
    let bind : std::net::SocketAddr = args[1].parse().unwrap();
    let ss = TcpListener::bind(bind).await.unwrap();
    while let Ok((cs,_)) = ss.accept().await {
        println!("Obtained client socket");
        let fd = cs.as_raw_fd();
        let ef = nix::sys::epoll::EpollCreateFlags::EPOLL_CLOEXEC;
        let e = nix::sys::epoll::epoll_create1(ef).unwrap();
        let mut ev = nix::sys::epoll::EpollEvent::new(nix::sys::epoll::EpollFlags::EPOLLRDHUP, 0);
        nix::sys::epoll::epoll_ctl(e,nix::sys::epoll::EpollOp::EpollCtlAdd, fd, Some(&mut ev)).unwrap();
        //dbg!(ev);
        let mut evts = [nix::sys::epoll::EpollEvent::empty()];
        nix::sys::epoll::epoll_wait(e, &mut evts, -1).unwrap();
        println!("waited");
    }
}
 */



#[tokio::main(flavor="current_thread")]
async fn main() {
    use tokio::net::{TcpListener, TcpStream};
    let args : Vec<_> = std::env::args().collect();
    let bind : std::net::SocketAddr = args[1].parse().unwrap();
    let ss = TcpListener::bind(bind).await.unwrap();
    let hangup_monitor = tcphangupmonitor::TcpHangupMonitor::new().unwrap();
    while let Ok((mut cs,_)) = ss.accept().await {
        println!("Obtained client socket");
        let tx = hangup_monitor.register(&cs, true).unwrap();
        tokio::spawn(async move {
            tx.await;
            println!("waited");
        });
        let mut buf = [0; 4];
        cs.read(&mut buf).await;
    }
}
