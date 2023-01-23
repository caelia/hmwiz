use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Instant, Duration};
use std::thread;

use crate::grid::Grid;
use crate::layout::Point;

const TIMEOUT: Duration = Duration::from_secs(20);
const MSG_TIMEOUT: Duration = Duration::from_millis(2);
const SHORT_PAUSE: Duration = Duration::from_micros(2);
const LONG_PAUSE: Duration = Duration::from_millis(10);

#[derive(Debug)]
enum CrawlerRole {
    Edge,
    Inner,
    Middle,
}

#[derive(Debug)]
enum Msg {
    Start,
    Continue,
    Request(String),
    Roger,
    NoCanDo,
    Done,
}

#[derive(Debug)]
enum CSel {
    Inner,
    Outer,
    Outer1,
    Outer2,
    Rear,
}

type Q = Rc<VecDeque<Msg>>;

#[derive(Debug)]
struct Channel {
    a: Q,
    b: Q,
    connections: usize,
}

impl Channel {
    fn new() -> Self {
        Channel {
            a: Rc::new(VecDeque::new()),
            b: Rc::new(VecDeque::new()),
            connections: 0
        }
    }
    fn get_ports(&mut self) -> (Q, Q) {
        match self.connections {
            0 => {
                self.connections = 1;
                (self.a, self.b)
            },
            1 => {
                self.connections = 2;
                (self.b, self.a)
            },
            _ => panic!("No ports available."),
        }
    }

}

#[derive(Debug)]
struct MsgPorts {
    rear_send: Option<Q>,
    rear_recv: Option<Q>,
    lateral1_send: Option<Q>,
    lateral1_recv: Option<Q>,
    lateral2_send: Option<Q>,
    lateral2_recv: Option<Q>,
}

impl MsgPorts {
    fn new() -> Self {
        MsgPorts {
            rear_send: None,
            rear_recv: None,
            lateral1_send: None,
            lateral1_recv: None,
            lateral2_send: None,
            lateral2_recv: None
        }
    }
}

#[derive(Debug)]
struct FakeSlice {}  // just until I can figure out what to do in grid module

#[derive(Debug)]
struct GridCrawler<'a> {
    data: &'a mut Vec<Point<f32>>,
    role: CrawlerRole,
    ports: MsgPorts,    
}

impl GridCrawler<'_> {
    fn new(role: CrawlerRole, data: &'_ mut Vec<Point<f32>>) -> Self {
        GridCrawler {
            data,
            role,
            ports: MsgPorts::new()
        }
    }
    fn connect_rear(&mut self, (send, recv): (Q, Q) {
        self.ports.rear_send = Some(send);
        self.ports.rear_recv = Some(recv);
    }
    fn connect_inner(&mut self, (send, recv): (Q, Q)) {
        self.ports.lateral1_send = Some(send);
        self.ports.lateral1_recv = Some(recv);
    }
    fn connect_outer(&mut self, (send, recv): (Q, Q)) {
        self.ports.lateral2_send = Some(send);
        self.ports.lateral2_recv = Some(recv);
    }
    fn connect_outer1(&mut self, (send, recv): (Q, Q)) {
        self.ports.lateral1_send = Some(send);
        self.ports.lateral1_recv = Some(recv);
    }
    fn connect_outer2(&mut self, (send, recv): (Q, Q)) {
        self.ports.lateral2_send = Some(send);
        self.ports.lateral2_recv = Some(recv);
    }
    fn verify_connections(&self) -> bool {
        match self.role {
            CrawlerRole::Edge => {
                self.ports == MsgPorts {
                    rear_send: Some(_),
                    rear_recv: Some(_),
                    lateral1_send: Some(_),
                    lateral1_recv: Some(_),
                    lateral2_send: None,
                    lateral2_recv: None
                }
            },
            _ => {
                self.ports == MsgPorts {
                    rear_send: None,
                    rear_recv: None,
                    lateral1_send: Some(_),
                    lateral1_recv: Some(_),
                    lateral2_send: Some(_),
                    lateral2_recv: Some(_)
                }
            }
        }
    }
    fn send(&self, sel: CSel, msg: Msg) {
        let mut port = match sel {
            CSel::Inner | CSel::Outer1 => self.ports.lateral1_send,
            CSel::Outer | CSel::Outer2 => self.ports.lateral2_send,
            CSel::Rear => self.ports.rear_send,
        };
        match port {
            Some(p) => p.push_back(msg),
            None => panic!("Attempting to use a nonexistent message port ({:?})", sel),
        }
    }
    fn recv(&self, sel: CSel) -> Option<Msg> {
        let mut port = match sel {
            CSel::Inner | CSel::Outer1 => self.ports.lateral1_recv,
            CSel::Outer | CSel::Outer2 => self.ports.lateral2_recv,
            CSel::Rear => self.ports.rear_recv,
        };
        match port {
            Some(p) => p.pop_front(),
            None => panic!("Attempting to use a nonexistent message port ({:?})", sel),
        }
    }
    fn await_msg(&self, sel: CSel) -> Option<Msg> {
        let timer = Instant::now();
        loop {
            let msg = self.recv(sel);
            match msg {
                Some(_) => return msg,
                None => (),
            }
            if timer.elapsed() > MSG_TIMEOUT {
                return None
            }
            thread::sleep(SHORT_PAUSE);
        }
    }
    fn ready(&self) {
        loop {
            
        }
    }
}

#[derive(Debug)]
pub struct GridCrawlerArray<'a> {
    crawlers: Vec<GridCrawler<'a>>,
    channels: (Channel, Channel),
    recv_port1: Q,
    recv_port2: Q,
    send_port1: Q,
    send_port2: Q,
}

impl GridCrawlerArray<'_> {
    pub fn new(grid: Grid<Point<f32>>) -> Self {
        let mut slices = grid.get_slices();
        let (mid, last) = (slices.len() / 2, slices.len() - 1);
        let mut crawlers = Vec::new();
        for i in 0..slices.len() {
            let crawler = if i == 0 || i == last {
                GridCrawler::new(CrawlerRole::Edge, slices[i])
            } else if i == mid {
                GridCrawler::new(CrawlerRole::Middle, slices[i])
            } else {
                GridCrawler::new(CrawlerRole::Inner, slices[i])
            };
            crawlers.push(crawler);
        }
        let ch1 = Channel::new();
        let ch2 = Channel::new();
        let (rp1, sp1) = ch1.get_ports();
        let (rp2, sp2) = ch2.get_ports();
        GridCrawlerArray {
            crawlers,
            channels: (ch1, ch2),
            recv_port1: rp1,
            recv_port2: rp2,
            send_port1: sp1,
            send_port2: sp2
         }
    }

    pub fn connect(&self) {
        let mut left = self.crawlers[0];
        let mut right = self.crawlers[self.crawlers.len() - 1];
        left.connect_rear(self.channels.0.get_ports());
        right.connect_rear(self.channels.1.get_ports());
        let end = self.crawlers.len() - 1;
        let mut past_middle = false;
        for i in 0..end {
            let current = self.crawlers[i];
            let next = self.crawlers[i + 1];
            let channel = Channel::new();
            match (current.role, past_middle) {
                (CrawlerRole::Middle, false) => {
                    current.connect_outer2(channel.get_ports());
                    past_middle = true;
                },
                (CrawlerRole::Middle, true) => {
                    panic!("There can only be one Middle Crawler!");
                },
                (CrawlerRole::Inner, false) | (CrawlerRole::Edge, _) => {
                    current.connect_inner(channel.get_ports());
                },
                (CrawlerRole::Inner, true) => {
                    current.connect_outer(channel.get_ports());
                },
            }
            match (next.role, past_middle) {
                (CrawlerRole::Middle, _) => {
                    next.connect_outer1(channel.get_ports());
                },
                (CrawlerRole::Inner, false) => {
                    next.connect_outer(channel.get_ports());
                },
                (CrawlerRole::Inner, true) => {
                    next.connect_inner(channel.get_ports());
                },
                (CrawlerRole::Edge, _) => {
                    next.connect_inner(channel.get_ports());
                }
            }
        }

        if !self.crawlers.iter().all(|cr| cr.verify_connections()) {
            panic!("Some message connections are incorrect!");
        }
    }

    pub fn crawl(&self) {
        let left = self.crawlers.first();
        let right = self.crawlers.last();
        let mut left_done = false;
        let mut right_done = false;
        
        self.send_port1.push_back(Msg::Start);
        self.send_port2.push_back(Msg::Start);
        
        let timer = Instant::now();
        loop {
            if !left_done { 
                match self.recv_port1.pop_front() {
                    Some(Msg::Done) => left_done = true,
                    Some(msg) => panic!("Incorrect message received! {:?}", msg), 
                    None => (),
                }
            }
            if !right_done { 
                match self.recv_port2.pop_front() {
                    Some(Msg::Done) => right_done = true,
                    Some(msg) => panic!("Incorrect message received! {:?}", msg), 
                    None => (),
                }
            }
            if left_done && right_done {
                println!("Grid crawlers have completed all tasks!");
                break;
            }
            if timer.elapsed > TIMEOUT {
                panic!("Grid crawler timeout!");
            }
            thread::sleep(LONG_PAUSE);
        }
    }
}
