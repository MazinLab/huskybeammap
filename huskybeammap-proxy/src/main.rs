use huskybeammap_types::{Object, StatusMessage};

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::mpsc::{Sender, TryRecvError, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::spawn;

use facet_json::{from_str, to_string};
use tungstenite::{Message, accept};
use uuid::Uuid;

/// Absolutely disgusting but it should work and not peg the CPU
fn main() {
    let phone_server = TcpListener::bind("0.0.0.0:9001").unwrap();
    let proxy_server = TcpListener::bind("0.0.0.0:9002").unwrap();

    let notification = Arc::new((Mutex::new(0usize), Condvar::new()));

    let phones = Arc::new(Mutex::new(HashMap::new()));
    let clients = Arc::new(Mutex::new(HashMap::new()));

    let phones_c = phones.clone();
    spawn(move || {
        let lp = phones_c.clone();
        for stream in phone_server.incoming() {
            let llp = lp.clone();
            spawn(move || {
                let lllp = llp.clone();
                let s = stream.unwrap();
                println!("Phone Connected From {:?}", s.peer_addr());
                let mut websocket = accept(s).unwrap();

                let id = Uuid::new_v4();
                let (sstatus, rstatus): (Sender<StatusMessage>, _) = channel();
                let (sobj, robj): (Sender<Vec<Object>>, _) = channel();
                {
                    let mut ps = lllp.lock().unwrap();
                    ps.insert(id, (rstatus, sobj));
                }
                loop {
                    let obj = robj.recv().unwrap();
                    websocket
                        .send(Message::Text(to_string(&obj).into()))
                        .unwrap();
                    match websocket.read().unwrap() {
                        Message::Text(t) => {
                            let resp = from_str(t.as_str()).unwrap();
                            sstatus.send(resp).unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
            });
        }
    });

    let clients_c = clients.clone();
    let notification_c = notification.clone();
    spawn(move || {
        let lc = clients_c.clone();
        let ln = notification_c.clone();
        for stream in proxy_server.incoming() {
            let llc = lc.clone();
            let lln = ln.clone();
            spawn(move || {
                let lllc = llc.clone();
                let llln = lln.clone();
                let s = stream.unwrap();
                println!("Client Connected From {:?}", s.peer_addr());
                let mut websocket = accept(s).unwrap();

                let id = Uuid::new_v4();
                let (sstatus, rstatus): (Sender<Vec<StatusMessage>>, _) = channel();
                let (sobj, robj): (Sender<Vec<Object>>, _) = channel();
                {
                    let mut cs = lllc.lock().unwrap();
                    cs.insert(id, (sstatus, robj));
                }
                loop {
                    match websocket.read().unwrap() {
                        Message::Text(t) => {
                            let obj = from_str(t.as_str()).unwrap();
                            sobj.send(obj).unwrap();
                        }
                        _ => unreachable!(),
                    }
                    {
                        let mut p = llln.0.lock().unwrap();
                        *p += 1;
                        llln.1.notify_one();
                    }
                    let resp = rstatus.recv().unwrap();
                    websocket
                        .send(Message::Text(to_string(&resp).into()))
                        .unwrap();
                }
            });
        }
    });

    let lc = clients.clone();
    let ln = notification.clone();
    loop {
        let llc = lc.clone();
        let lln = ln.clone();
        {
            let mut c = lln.0.lock().unwrap();
            while *c == 0 {
                c = lln.1.wait(c).unwrap();
            }
            *c -= 1;
        }
        let status_sender = {
            let mut cs = llc.lock().unwrap();
            let mut hangups = vec![];
            let mut res = None;
            for (k, v) in cs.iter() {
                match v.1.try_recv() {
                    Ok(p) => {
                        res = Some((v.0.clone(), p));
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        continue;
                    }
                    Err(TryRecvError::Disconnected) => {
                        hangups.push(*k);
                    }
                }
            }
            for hangup in hangups {
                cs.remove(&hangup);
            }
            res
        };

        if let Some((s, objs)) = status_sender {
            let _ = s.send({
                let mut ps = phones.lock().unwrap();
                let mut hangups = vec![];
                for (k, v) in ps.iter() {
                    if v.1.send(objs.clone()).is_err() {
                        hangups.push(*k);
                    }
                }
                for hangup in hangups.iter() {
                    ps.remove(hangup);
                }
                hangups.clear();
                let mut resps = vec![];
                for (k, v) in ps.iter() {
                    if let Ok(r) = v.0.recv() {
                        resps.push(r);
                    } else {
                        hangups.push(*k);
                    }
                }
                for hangup in hangups.iter() {
                    ps.remove(hangup);
                }
                resps
            });
        }
    }
}
