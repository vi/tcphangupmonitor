#[tokio::main(flavor="current_thread")]
async fn main() {
    use tokio::net::{TcpListener, TcpStream};
    let args : Vec<_> = std::env::args().collect();
    let bind : std::net::SocketAddr = args[1].parse().unwrap();
    let connect : std::net::SocketAddr = args[2].parse().unwrap();
    let ss = TcpListener::bind(bind).await.unwrap();
    let hangup_monitor = tcphangupmonitor::TcpHangupMonitor::new().unwrap();
    while let Ok((cs,_)) = ss.accept().await {
        let g = TcpStream::connect(connect).await.unwrap();

        let g_hup = hangup_monitor.register(&g, false).unwrap();
        let cs_hup = hangup_monitor.register(&cs, false).unwrap();

        let (mut gr, mut gw) = g.into_split();
        let (mut csr, mut csw) = cs.into_split();
        use tokio::io::AsyncWriteExt;
        let g2cs = tokio::spawn(async move {
            let _ = tokio::io::copy(&mut gr, &mut csw).await;
            let _ = csw.shutdown();
        });
        let cs2g = tokio::spawn( async move {
            let _ = tokio::io::copy(&mut csr, &mut gw).await;
            let _ = gw.shutdown();
        });

        tokio::spawn(async move {
            tokio::select! {
                _ = g_hup => {}
                _ = cs_hup => {}
            }
            cs2g.abort();
            g2cs.abort();
        });
    }
}

/*
tcptunnelchecker result:
[ OK ] Trivial test 1
[ OK ] Trivial test 2
[ OK ] Clogged close test passed: outDrainCheck_inClose
[ OK ] Clogged close test passed: outDrainCheck_inDrainClose
[ OK ] Clogged close test passed: outClogCheck_inClose
[ OK ] Clogged close test passed: outClogDrainCheck_inClose
[ OK ] Clogged close test passed: outShutDrainCheck_inClose
[ OK ] Clogged close test passed: outShutDrainCheck_inDrainClose
[ OK ] Clogged close test passed: outClogCheck_inClogClose
[ OK ] Clogged close test passed: outDrainCheck_inShutClose
[ OK ] Clogged close test passed: outDrainCheck_inShutDrainClose
[ OK ] Clogged close test passed: outClogCheck_inShutClose
[ OK ] Clogged close test passed: outClogDrainCheck_inShutClose
[ OK ] Clogged close test passed: outShutDrainCheck_inShutClose
[ OK ] Clogged close test passed: outShutDrainCheck_inShutDrainClose
[ OK ] Clogged close test passed: outClose_inDrainCheck
[ OK ] Clogged close test passed: outDrainClose_inDrainCheck
[ OK ] Clogged close test passed: outShutClose_inDrainCheck
[ OK ] Clogged close test passed: outShutDrainClose_inDrainCheck
[ OK ] Clogged close test passed: outClose_inClogCheck
[ OK ] Clogged close test passed: outClose_inClogDrainCheck
[ OK ] Clogged close test passed: outClogClose_inClogCheck
[ OK ] Clogged close test passed: outShutClose_inClogCheck
[ OK ] Clogged close test passed: outShutClose_inClogDrainCheck
[ OK ] Clogged close test passed: outClose_inShutDrainCheck
[ OK ] Clogged close test passed: outDrainClose_inShutDrainCheck
[ OK ] Clogged close test passed: outShutClose_inShutDrainCheck
[ OK ] Clogged close test passed: outShutDrainClose_inShutDrainCheck
*/
