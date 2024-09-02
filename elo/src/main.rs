extern crate rand;
extern crate lazy_static;

use std::{
    collections::VecDeque, 
    error::Error, 
    fmt::Display, 
    mem::swap, 
    sync::{Arc, Condvar, Mutex}, 
    thread::{sleep, spawn}, 
    time::Duration, 
    ptr::{addr_of, addr_of_mut}, 
};
use rand::prelude::*;

#[derive(Debug)]
struct SelfError {
    message: String,
}

impl SelfError {
    fn new<T>(msg: T) -> Self 
    where T: Into<String> {
        SelfError {
            message: msg.into(),
        }
    }
}

impl Display for SelfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SelfError: {}", self.message)
    }
}

impl Error for SelfError {}

struct TestResult {
    pub player: (usize, usize),
    pub result: MatchResult,
}

enum MatchResult {
    Win,
    Draw,
    Loss,
}

struct ResPool {
    resources: VecDeque<TestResult>,
    mutex: Option<Arc<Mutex<u32>>>,
    condvar: Option<Arc<Condvar<>>>,
    empty_mutex: Option<Arc<Mutex<bool>>>,
    empty_condvar: Option<Arc<Condvar<>>>,
}

impl ResPool {
    const fn new() -> Self {
        ResPool {
            resources: VecDeque::new(),
            mutex: None,
            condvar: None,
            empty_mutex: None,
            empty_condvar: None,
        }
    }

    fn initialize(&mut self) {
        self.mutex = Some(Arc::new(Mutex::new(0)));
        self.condvar = Some(Arc::new(Condvar::new()));
        self.empty_mutex = Some(Arc::new(Mutex::new(false)));
        self.empty_condvar = Some(Arc::new(Condvar::new()));
    }

    fn read(&mut self) -> Option<TestResult> {
        let rt = self.mutex.clone().unwrap();
        let mut t = rt.lock().unwrap();
        while *t == 0 {
            t = self.condvar.clone().unwrap().wait(t).unwrap();
        }
        let res = self.resources.pop_front();
        *t -= 1;
        if *t == 0 {
            drop(t);
            let rt2 = self.empty_mutex.clone().unwrap();
            let mut t2 = rt2.lock().unwrap();
            *t2 = true;
            self.empty_condvar.clone().unwrap().notify_one();
        }
        res
    }

    fn write(&mut self, tar: TestResult) {
        let rt = self.mutex.clone().unwrap();
        let mut t = rt.lock().unwrap();
        self.resources.push_back(tar);
        *t += 1;
        drop(t);
        let rt2 = self.empty_mutex.clone().unwrap();
        let mut t2 = rt2.lock().unwrap();
        *t2 = false;
        self.condvar.clone().unwrap().notify_all();
    }

    fn wait_empty(&self) {
        let rt2 = self.empty_mutex.clone().unwrap();
        let t2 = rt2.lock().unwrap();
        if *t2 == false {
            let _unused = self.empty_condvar.clone().unwrap().wait(t2);
        }
    }
}

const INITSCORE: i32 = 1000;

struct ScoreRecorder {
    score: Vec<i32>,
    mutex: Vec<Arc<Mutex<bool>>>,
}

impl ScoreRecorder {
    const fn new() -> Self {
        ScoreRecorder {
            score: vec![],
            mutex: vec![],
        }
    }

    fn initialize(&mut self, player_num: usize) {
        self.score = vec![INITSCORE; player_num];
        self.mutex = vec![];
        for _i in 0..player_num {
            self.mutex.push(Arc::new(Mutex::new(false)));
        }
    }

    fn modify(&mut self, res: TestResult) {
        let (mut p1, mut p2) = res.player;
        if p1 > p2 {
            swap(&mut p1, &mut p2);
        } else if p1 == p2 {
            println!("Error Occured in Score modifying. Someone beats himself!");
            return
        }
        let t1 = self.mutex[p1].lock().unwrap();
        let t2 = self.mutex[p2].lock().unwrap();
        let delta = match res.result {
            MatchResult::Win => {
                if self.score[p1] >= self.score[p2] {20} else {30}
            }
            MatchResult::Draw => {
                match self.score[p1].cmp(&self.score[p2]) {
                    std::cmp::Ordering::Less => 10,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => -10,
                }
            }
            MatchResult::Loss => {
                if self.score[p2] >= self.score[p1] {-20} else {-30}
            }
        };
        self.score[p1] += delta;
        self.score[p2] -= delta;
        drop(t2);
        drop(t1);
        sleep(Duration::from_millis(1));
    }

    fn display(&self) {
        let mut total = 0;
        for (i, score) in self.score.iter().enumerate() {
            total += score;
            println!("Player {i} gets {score} score!");
        }
        println!();
        println!("Total score is {total}");
    }
}

static mut SR: ScoreRecorder = ScoreRecorder::new();
static mut RP: ResPool = ResPool::new();

/// producer use this bound to produce MatchResult
/// we have the probability (PRO. / 2) in PRO. to be Win or Fail
/// and only if PRO. is odd, 1 in PRO to be Draw
const PRODUCER_RANGE_UPPER_BOUND: usize = 9;
const WIN_UPPER_BOUND: usize = PRODUCER_RANGE_UPPER_BOUND / 2;
const LOSS_LOWER_BOUND: usize = (PRODUCER_RANGE_UPPER_BOUND + 1) / 2 + 1;

fn producer(p: usize, k: usize) {
    let rpm = unsafe { addr_of_mut!(RP).as_mut().unwrap() };
    let mut rng = thread_rng();
    for _i in 0..k {
        let p1 = rng.gen_range(0..p);
        let mut p2 = rng.gen_range(0..(p-1));
        if p2 >= p1 { p2 += 1; }
        let match_number = rng.gen_range(1..=PRODUCER_RANGE_UPPER_BOUND);
        let match_result = 
        if match_number <= WIN_UPPER_BOUND { MatchResult::Win }
        else if match_number >= LOSS_LOWER_BOUND { MatchResult::Loss }
        else { MatchResult::Draw };
        rpm.write(
        TestResult { 
                player: (p1, p2), 
                result: match_result, 
            }
        );
    }
}

fn consumer() {
    let rpm = unsafe { addr_of_mut!(RP).as_mut().unwrap() };
    loop {
        let res = rpm.read().unwrap();
        let sr = unsafe { addr_of_mut!(SR).as_mut().unwrap() };
        sr.modify(res);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (p, m, n, k) = (
        if let Some(t) = option_env!("P") {
            t.parse::<usize>()?
        } else {
            return Err(Box::new(SelfError::new("Needed Command arg P parse error")));
        }, 
        if let Some(t) = option_env!("M") {
            t.parse::<usize>()?
        } else {
            return Err(Box::new(SelfError::new("Needed Command arg M parse error")));
        },
        if let Some(t) = option_env!("N") {
            t.parse::<usize>()?
        } else {
            return Err(Box::new(SelfError::new("Needed Command arg N parse error")));
        },
        if let Some(t) = option_env!("K") {
            t.parse::<usize>()?
        } else {
            return Err(Box::new(SelfError::new("Needed Command arg K parse error")));
        },
    );

    let sr = unsafe { addr_of_mut!(SR).as_mut().unwrap() };
    sr.initialize(p); 
    let rp = unsafe {addr_of_mut!(RP).as_mut().unwrap() };
    rp.initialize();

    for _i in 0..n {
        spawn(move || consumer());
    }
    let mut thread_record = Vec::new();
    for _i in 0..m {
        thread_record.push(spawn(move || producer(p, k)));
    }
    
    for handle in thread_record {
        handle.join().expect("Producer exited failed");
    }

    let monitor = spawn(move || {
        rp.wait_empty();
    });
    monitor.join().expect("Error occured in waiting for all consumed");
    
    let mutex_guard = unsafe { addr_of!(SR).as_ref().unwrap() };
    mutex_guard.display();

    Ok(())
}
