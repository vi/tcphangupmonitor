#[tokio::main(flavor="current_thread")]
async fn main() {
    use tokio::net::{TcpListener, TcpStream};
    let args : Vec<_> = std::env::args().collect();
    let bind : std::net::SocketAddr = args[1].parse().unwrap();
    let connect : std::net::SocketAddr = args[2].parse().unwrap();
    let ss = TcpListener::bind(bind).await.unwrap();
    while let Ok((cs,_)) = ss.accept().await {
        let g = TcpStream::connect(connect).await.unwrap();
        let (mut gr, mut gw) = g.into_split();
        let (mut csr, mut csw) = cs.into_split();
        use tokio::io::AsyncWriteExt;
        tokio::spawn(async move {
            let _ = tokio::io::copy(&mut gr, &mut csw).await;
            let _ = csw.shutdown();
        });
        tokio::spawn( async move {
            let _ = tokio::io::copy(&mut csr, &mut gw).await;
            let _ = gw.shutdown();
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
[FAIL] Clogged close test failed: outClogCheck_inClogClose
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
[FAIL] Clogged close test failed: outClogClose_inClogCheck
[ OK ] Clogged close test passed: outShutClose_inClogCheck
[ OK ] Clogged close test passed: outShutClose_inClogDrainCheck
[ OK ] Clogged close test passed: outClose_inShutDrainCheck
[ OK ] Clogged close test passed: outDrainClose_inShutDrainCheck
[ OK ] Clogged close test passed: outShutClose_inShutDrainCheck
[ OK ] Clogged close test passed: outShutDrainClose_inShutDrainCheck
*/
