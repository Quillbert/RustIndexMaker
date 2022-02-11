use crate::index_entry::IndexEntry;
use dashmap::DashMap;
use regex::Regex;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

#[derive(Debug)]
pub struct DocumentIndex {
    longest_word: String,
    most_frequent_word: String,
    least_frequent_word: String,
    shortest_meaningful_word: String,
    total_words: u32,
    index: Arc<DashMap<String, IndexEntry>>,
    workers: Vec<Adder>,
    sender: mpsc::Sender<Message>,
}

impl DocumentIndex {
    pub fn new() -> DocumentIndex {
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let size = num_cpus::get();

        let mut workers = Vec::with_capacity(size);

        let index = DashMap::new();

        let index_arc = Arc::new(index);

        for _ in 0..size {
            workers.push(Adder::new(Arc::clone(&receiver), Arc::clone(&index_arc)));
        }

        DocumentIndex {
            longest_word: String::new(),
            most_frequent_word: String::new(),
            least_frequent_word: String::new(),
            shortest_meaningful_word: String::new(),
            total_words: 0,
            index: index_arc,
            workers,
            sender,
        }
    }

    pub fn add_all_words(&self, line: &str, num: u32) {
        self.sender
            .send(Message::NewJob(String::from(line), num))
            .unwrap();
    }

    pub fn join(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for adder in &mut self.workers {
            if let Some(thread) = adder.thread.take() {
                thread.join().unwrap();
            }
        }
    }

    fn set_total_words(&mut self) {
        for entry in self.index.iter() {
            self.total_words += entry.get_occurrences();
        }

        for mut entry in self.index.iter_mut() {
            entry.set_total_words(self.total_words);
        }
    }

    pub fn update_info(&mut self) {
        self.set_total_words();

        let mut least_frequency: f64 = 3.0;
        let mut most_frequency: f64 = -1.0;

        for entry in self.index.iter() {
            let ent = entry;

            if (ent.get_word().len() < self.shortest_meaningful_word.len()
                || self.shortest_meaningful_word.is_empty())
                && DocumentIndex::is_meaningful(&ent.get_word())
                && ent.get_frequency() >= 0.01
            {
                self.shortest_meaningful_word = ent.get_word();
            }
            if ent.get_word().len() > self.longest_word.len() {
                self.longest_word = ent.get_word();
            }
            let frequency = ent.get_frequency();
            if frequency < least_frequency {
                self.least_frequent_word = ent.get_word();
                least_frequency = frequency;
            }
            if frequency > most_frequency {
                self.most_frequent_word = ent.get_word();
                most_frequency = frequency;
            }
        }
    }

    fn is_meaningful(word: &str) -> bool {
        !word.is_empty()
            && word != "A"
            && word != "AN"
            && word != "THE"
            && !word.chars().next().unwrap().is_numeric()
            && (word.len() > 1 || word != "I")
    }

    pub fn size(&self) -> usize {
        self.index.len()
    }

    pub fn get_longest_word(&self) -> &str {
        &self.longest_word
    }

    pub fn get_most_frequent_word(&self) -> &str {
        &self.most_frequent_word
    }

    pub fn get_least_frequent_word(&self) -> &str {
        &self.least_frequent_word
    }

    pub fn get_shortest_meaningful_word(&self) -> &str {
        &self.shortest_meaningful_word
    }

    pub fn get_values(&self) -> Vec<dashmap::mapref::multiple::RefMulti<'_, String, IndexEntry>> {
        self.index.iter().collect()
    }
}

impl Default for DocumentIndex {
	fn default() -> Self {
		Self::new()
	}
}

enum Message {
    NewJob(String, u32),
    Terminate,
}

#[derive(Debug)]
struct Adder {
    thread: Option<thread::JoinHandle<()>>,
}

impl Adder {
    fn new(
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        map: Arc<DashMap<String, IndexEntry>>,
    ) -> Adder {
        let splitter = Regex::new("\\W+").unwrap();
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(line, line_num) => {
                    let words: Vec<_> = splitter.split(&line).collect();
                    for word in words {
                        if !word.is_empty() {
                            add_word(Arc::clone(&map), &word.to_uppercase(), line_num);
                        }
                    }
                }
                Message::Terminate => break,
            }
        });

        Adder {
            thread: Some(thread),
        }
    }
}

fn add_word(map: Arc<DashMap<String, IndexEntry>>, word: &str, line_num: u32) {
    let key = String::from(word);
    let mut entry = map
        .entry(String::clone(&key))
        .or_insert(IndexEntry::new(&key));
    entry.add(line_num);
}
