
#[cfg(test)]
mod tests {
    use bbqueue::BBQueue;
    use std::thread::spawn;
    use std::time::{Instant, Duration};

    #[test]
    fn sanity_check() {
        // Hmm, this is probably an interface smell
        let bb = Box::new(BBQueue::new());
        let bbl = Box::leak(bb);
        let panny = format!("{:p}", &bbl.buf[0]);
        let (mut tx, mut rx) = bbl.split();

        const ITERS: usize = 10_000_000;

        let timeout_tx = Duration::from_millis(10000);
        let timeout_rx = Duration::from_millis(10100);
        let start_tx = Instant::now();
        let start_rx = start_tx.clone();

        let tx_thr = spawn(move || {
            for i in 0..ITERS {
                'inner: loop {
                    if start_tx.elapsed() > timeout_tx {
                        panic!("tx timeout, iter {}", i);
                    }
                    match tx.grant(1) {
                        Ok(gr) => {
                            gr.buf[0] = (i & 0xFF) as u8;
                            tx.commit(1, gr);
                            break 'inner;
                        }
                        Err(_) => {}
                    }
                }
            }
        });

        let rx_thr = spawn(move || {
            for i in 0..ITERS {
                'inner: loop {
                    ::std::sync::atomic::fence(::std::sync::atomic::Ordering::SeqCst);
                    if start_rx.elapsed() > timeout_rx {
                        panic!("rx timeout, iter {}", i);
                    }
                    let gr = rx.read();
                    if gr.buf.is_empty() {
                        continue 'inner;
                    }
                    let act = gr.buf[0] as u8;
                    let exp = (i & 0xFF) as u8;
                    if act != exp {
                        println!("baseptr: {}", panny);
                        println!("offendr: {:p}", &gr.buf[0]);
                        println!("act: {:?}, exp: {:?}", act, exp);
                        println!("len: {:?}", gr.buf.len());
                        println!("{:?}", gr.buf);
                        panic!("RX Iter: {}, mod: {}", i, i % 6);
                    }
                    rx.release(1, gr);
                    break 'inner;
                }
            }
        });

        tx_thr.join().unwrap();
        rx_thr.join().unwrap();
    }

}
