use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Write};
use std::path::Path;
use std::{env, fs};

const CHUNK_SIZE: usize = 100;
const CHAT_LOG: &str = "<div class=\"chatlog\">";
const MESSAGE_CONTAINER: &str = "<div class=\"chatlog__message-group\">";
const POSTAMBLE: &str = "<div class=\"postamble\">";
const CHAT_LOG_LENGTH: usize = CHAT_LOG.len();

const LOADING_CHARS: &str = "|/-\\";
const LOADING_CHARS_LENGTH: usize = LOADING_CHARS.len();

pub enum Operation {
    SingleFile,
    Directory,
}

pub struct Config {
    path: Box<Path>,
    pub operation: Operation,
}

impl Config {
    pub fn new(args: env::Args) -> Result<Config, String> {
        let args: Vec<String> = args.collect();

        if args.len() < 2 {
            return Err(format!("Usage: {} chat.html", args[0]));
        }

        let path = fs::canonicalize(Path::new(&args[1])).expect("Unable to expand path");

        let operation = if path.is_dir() {
            Operation::Directory
        } else {
            Operation::SingleFile
        };

        Ok(Config {
            path: Box::from(path),
            operation,
        })
    }

    pub fn path(&self) -> &Path {
        &*self.path
    }

    pub fn paths(&self) -> Vec<Box<Path>> {
        fs::read_dir(&self.path)
            .expect("Unable to read files")
            .filter_map(|x| x.ok())
            .map(|x| x.path())
            .filter(|x| Some(OsStr::new("html")) == x.extension())
            .map(Box::from)
            .collect()
    }
}

pub fn split_file(path: &Path) {
    eprintln!("Reading contents from {:?}...", path);
    let content = read_file(path).expect("Unable to read file");
    let chatlog_index = content.find(CHAT_LOG).unwrap();

    eprintln!("Creating slices...");
    let offsets = extract_offsets(&content, chatlog_index);

    eprintln!("Creating the output dir...");
    let directory = create_directory(&path);

    write_files(
        &content,
        &offsets,
        &directory,
        chatlog_index + CHAT_LOG_LENGTH,
    );

    eprintln!("All files created")
}

fn read_file(path: &Path) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn navigation_links(index: usize, last: bool) -> String {
    let mut navigation = String::from("<span>");

    if index != 0 {
        navigation.push_str(format!("<a href=\"{}.html\">Previous</a>", index - 1).as_str());
    }
    if !last {
        navigation.push_str(format!(" <a href=\"{}.html\">Next</a>", index + 1).as_str());
    }

    navigation.push_str("</span>");

    navigation
}

fn open_or_create_file(path: &Path) -> File {
    if path.exists() {
        OpenOptions::new().write(true).open(path).unwrap()
    } else {
        File::create(path).unwrap()
    }
}

fn extract_offsets(content: &str, mut current_offset: usize) -> Vec<usize> {
    let mut count = 0;
    let mut offsets = vec![];

    // Find all message containers, chunk them into
    while let Some(offset) = &content[current_offset + 1..].find(MESSAGE_CONTAINER) {
        count += 1;
        current_offset += offset + 1;
        if count % CHUNK_SIZE == 0 {
            offsets.push(current_offset)
        }
    }

    // Find the postamble
    let last_offset = content[current_offset + 1..].find(POSTAMBLE).unwrap();
    offsets.push(current_offset + last_offset - 1);

    offsets
}

fn create_directory(path: &Path) -> Box<Path> {
    let filename = path.file_stem().unwrap();
    let directory = path.parent().unwrap().join(filename);

    if !directory.exists() {
        fs::create_dir(&directory).unwrap()
    }

    Box::from(directory)
}

fn update_progress(index: usize, count: usize) {
    let loading_char = &LOADING_CHARS
        .chars()
        .nth(index % LOADING_CHARS_LENGTH)
        .unwrap();

    eprint!("\rCreating slice {}/{} {}", index + 1, count, loading_char);
}

fn write_files(content: &String, offsets: &Vec<usize>, directory: &Path, mut prev_offset: usize) {
    let chunk_count = offsets.len();
    let header = &content[..prev_offset];

    for (index, offset) in offsets.iter().enumerate() {
        update_progress(index, chunk_count);

        let path = directory.join(format!("{}.html", index));
        let mut file = open_or_create_file(&path);
        let navigation_links = navigation_links(index, chunk_count == index + 1);
        let messages = &content[prev_offset..*offset];

        file.write(header.as_bytes()).unwrap(); // Header
        file.write(navigation_links.as_bytes()).unwrap(); // Navigation
        file.write(messages.as_bytes()).unwrap(); // Messages
        file.write(navigation_links.as_bytes()).unwrap(); // Navigation
        file.write(b"</body></html>").unwrap(); // Closing tags

        prev_offset = *offset;
    }
    eprintln!("{} ", 8u8 as char)
}
