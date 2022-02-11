use index_maker_multithread::document_index::DocumentIndex;
use index_maker_multithread::index_entry::IndexEntry;
use rayon::prelude::*;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Write;
use std::io::{BufReader, BufWriter, Read};
use std::process;
use std::time::Instant;

fn main() {
    let mut infile_name = String::new();

    let args: Vec<_> = env::args().collect();

    if args.len() > 1 {
        infile_name = String::from(&args[1]);
    } else {
        print!("Enter input file name: ");
        io::stdout().flush().expect("Flush failed");
        io::stdin()
            .read_line(&mut infile_name)
            .unwrap_or_else(|err| {
                eprintln!("Problem parsing input: {}", err);
                process::exit(1);
            });
    }

    let mut outfile_name = String::new();

    if args.len() > 2 {
        outfile_name = String::from(&args[2]);
    } else {
        print!("Enter output file name: ");
        io::stdout().flush().expect("Flush failed");
        io::stdin()
            .read_line(&mut outfile_name)
            .unwrap_or_else(|err| {
                eprintln!("Problem parsing input: {}", err);
                process::exit(1);
            });
    }

    let start = Instant::now();

    let infile = File::open(&infile_name.trim()).unwrap_or_else(|err| {
        eprintln!("Problem opening file: {}", err);
        process::exit(1);
    });

    let mut reader = BufReader::new(infile);

    let mut contents = String::new();

    reader.read_to_string(&mut contents).unwrap_or_else(|err| {
        eprintln!("Problem reading file: {}", err);
        process::exit(1);
    });

    let mut output_file = File::create(&outfile_name.trim()).unwrap_or_else(|err| {
        eprintln!("Problem creating file: {}", err);
        process::exit(1);
    });

    let computation_start = Instant::now();

    let mut index = DocumentIndex::new();

    let mut line_num = 1;

    for line in contents.lines() {
        index.add_all_words(line, line_num);
        line_num += 1;
    }

    index.join();

    let insert_time = computation_start.elapsed().as_secs_f64();

    index.update_info();

    let mut entries = index.get_values();
    entries.par_sort_unstable_by_key(|key| key.get_word());

    let comp_time = computation_start.elapsed().as_secs_f64();

    write_to_outfile(&mut output_file, &index, &outfile_name, entries).unwrap_or_else(|err| {
        eprintln!("Error writing to file: {}", err);
        process::exit(1);
    });

    let final_time = start.elapsed().as_secs_f64();

    println!("Insert completed in {} seconds.", insert_time);
    println!("Logic completed in {} seconds.", comp_time);
    println!("Completed in {} seconds.", final_time);
}

fn write_to_outfile(
    outfile: &mut File,
    index: &DocumentIndex,
    file_name: &str,
    entries: Vec<dashmap::mapref::multiple::RefMulti<'_, String, IndexEntry>>,
) -> Result<(), Box<dyn Error>> {
    let mut outfile = BufWriter::new(outfile);

    outfile.write_all(file_name.trim().as_bytes())?;
    outfile.write_all(b"\nNumer of distinct words: ")?;
    outfile.write_all(index.size().to_string().as_bytes())?;
    outfile.write_all(b"\nLongest Word: ")?;
    outfile.write_all(index.get_longest_word().as_bytes())?;
    outfile.write_all(b"\nMost Frequent Word: ")?;
    outfile.write_all(index.get_most_frequent_word().as_bytes())?;
    outfile.write_all(b"\nLeast Frequent Word: ")?;
    outfile.write_all(index.get_least_frequent_word().as_bytes())?;
    outfile.write_all(b"\nShortest Meaningful Word: ")?;
    outfile.write_all(index.get_shortest_meaningful_word().as_bytes())?;
    outfile.write_all(b"\n")?;

    for entry in entries {
        outfile.write_all(entry.to_string().as_bytes())?;
        outfile.write_all(b"\n")?;
    }

    Ok(())
}
