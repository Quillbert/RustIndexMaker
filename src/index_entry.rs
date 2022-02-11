use std::cmp;
use std::collections::BTreeSet;
use std::fmt;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

#[derive(Debug)]
pub struct IndexEntry {
    word: Mutex<String>,
    nums_list: BTreeSet<ANum>,
    num_occurrences: AtomicU32,
    total_words: AtomicU32,
}

impl IndexEntry {
    pub fn new(word: &str) -> IndexEntry {
        IndexEntry {
            word: Mutex::new(String::from(word)),
            nums_list: BTreeSet::new(),
            num_occurrences: AtomicU32::new(0),
            total_words: AtomicU32::new(0),
        }
    }

    pub fn add(&mut self, num: u32) {
        self.nums_list.insert(ANum::new(num));
        self.num_occurrences.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_word(&self) -> String {
        String::clone(&self.word.lock().unwrap())
    }

    pub fn get_frequency(&self) -> f64 {
        self.num_occurrences.fetch_add(0, Ordering::Relaxed) as f64
            / self.total_words.fetch_add(0, Ordering::Relaxed) as f64
    }

    pub fn get_occurrences(&self) -> u32 {
        self.num_occurrences.fetch_add(0, Ordering::Relaxed)
    }

    pub fn set_total_words(&mut self, total_words: u32) {
        self.total_words = AtomicU32::new(total_words);
    }
}

impl fmt::Display for IndexEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = format!(
            "{}    Frequency: {}    Occurrences: ",
            self.get_word(),
            self.get_frequency()
        );
        let list = format!("{:?}", self.nums_list);
        out.push_str(&list[1..list.len() - 1]);
        write!(f, "{}", out)
    }
}

struct ANum {
    value: AtomicU32,
}

impl ANum {
    fn new(num: u32) -> ANum {
        ANum {
            value: AtomicU32::new(num),
        }
    }
}

impl std::fmt::Debug for ANum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value.fetch_add(0, Ordering::Relaxed))
    }
}

impl cmp::Ord for ANum {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let num1 = self.value.fetch_add(0, Ordering::Relaxed);
        let num2 = other.value.fetch_add(0, Ordering::Relaxed);
        if num1 > num2 {
            return cmp::Ordering::Greater;
        }
        if num1 < num2 {
            return cmp::Ordering::Less;
        }
        cmp::Ordering::Equal
    }
}

impl cmp::PartialOrd for ANum {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::PartialEq for ANum {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl cmp::Eq for ANum {}
