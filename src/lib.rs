use std::os::unix::prelude::{AsRawFd, RawFd};
use nix::sys::epoll::{EpollFlags,EpollEvent,EpollCreateFlags,EpollOp,epoll_create1,epoll_ctl,epoll_wait};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("cannot epoll_create1")]
    CantCreateEpollFd(#[source] nix::errno::Errno),
    #[error("cannot add fd")]
    CantEpollCtlAdd(#[source] nix::errno::Errno),
}

pub struct TcpHangupMonitor {
   epoll_fd: RawFd,
}

type EventSender = tokio::sync::oneshot::Sender<()>;

impl TcpHangupMonitor {
    /// Create new TcpHangupMonitor and start it's backtround thread.
    /// The thread won't exit ever (unless error happens), even if TcpHangupMonitor instance is dropped.
    pub fn new() -> Result<TcpHangupMonitor, Error> {
        let ef = EpollCreateFlags::EPOLL_CLOEXEC;
        let epoll_fd = epoll_create1(ef).map_err(|e|Error::CantCreateEpollFd(e))?;

        std::thread::spawn(move ||{
            loop {
                let mut evts = [nix::sys::epoll::EpollEvent::empty()];
                match epoll_wait(epoll_fd, &mut evts, -1) {
                    Ok(0) => (),
                    Ok(_) => {
                        let ev = evts[0];
                        let data = ev.data();
                        let ptr : *mut EventSender = data as *mut EventSender;
                        // SAFETY:
                        // relying on assumption that in EPOLLONESHOT mode we only receive the notification once for this FD,
                        // regardless what happens to the fd afterwards.
                        let snd = unsafe { Box::from_raw(ptr) };
                        if snd.send(()).is_err() {
                            log::debug!("Failed to deliver TCP hangup notification");
                        }
                        // not doing EPOLL_CTL_DEL because of file descriptor may be already closed
                        // and therefore auto-removed (if I understand epoll details correctly) from set.
                        // Trying to explicitly remove it again may actually remove some unrelated newly added fd.
                    }
                    Err(e) => {
                        log::error!("Error from TcpHangupMonitor's epoll_wait: {}", e);
                        break;
                    }
                }
            }
        });
        Ok(TcpHangupMonitor {
            epoll_fd,
        })
    }

    /// Start monitoring this TCP stream for hangup events (also optionally for read hangup events).
    /// 
    pub fn register(&self, socket: &tokio::net::TcpStream, also_subscribe_to_rdhup: bool) -> Result<tokio::sync::oneshot::Receiver<()>, Error> {
        let fd = socket.as_raw_fd();

        let (tx,rx) = tokio::sync::oneshot::channel();

        let snd = Box::new(tx);
        let ptr = Box::into_raw(snd);
        let data = ptr as u64;

        let mut flags = EpollFlags::EPOLLONESHOT;
        if also_subscribe_to_rdhup {
            flags |= EpollFlags::EPOLLRDHUP;
        }
        let mut ev = EpollEvent::new(flags, data);
        match epoll_ctl(self.epoll_fd,EpollOp::EpollCtlAdd, fd, Some(&mut ev)) {
            Ok(()) => (),
            Err(e) => {
                // SAFETY: I assume that `data` of a failed EPOLL_CTL_ADD is not going anywhere, so
                // we are the sole owner of `ptr` at this point.
                drop(unsafe { Box::from_raw(ptr) } );
                return Err(Error::CantEpollCtlAdd(e));
            }
        };

        Ok(rx)
    }
}
