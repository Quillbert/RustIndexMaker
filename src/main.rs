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
	
	let mut out = String::new();

    let mut outfile = BufWriter::new(outfile);

    out.push_str(&file_name.trim());
    out.push_str("\nNumer of distinct words: ");
    out.push_str(&index.size().to_string());
    out.push_str("\nLongest Word: ");
    out.push_str(&index.get_longest_word());
    out.push_str("\nMost Frequent Word: ");
    out.push_str(&index.get_most_frequent_word());
    out.push_str("\nLeast Frequent Word: ");
    out.push_str(&index.get_least_frequent_word());
    out.push_str("\nShortest Meaningful Word: ");
    out.push_str(&index.get_shortest_meaningful_word());
    out.push_str("\n");

    for entry in entries {
        out.push_str(&entry.to_string());
        out.push_str("\n");
    }

	outfile.write_all(out.as_bytes())?;

    Ok(())
}
