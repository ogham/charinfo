#![feature(exit_status, io, unicode)]

extern crate getopts;
extern crate term;
extern crate unicode_names;

extern crate unicode_width;
use unicode_width::UnicodeWidthChar;

use std::io;
use std::io::Read;
use std::env;
use std::fmt;


fn main() {
    let args: Vec<_> = env::args().collect();
    match Options::getopts(&args[..]) {
        Ok(options)   => {
            let thing = io::stdin();
            let stdin = thing.lock().chars();
            CharInfo::new(options, stdin).run();
        },
        Err(misfire)  => {
            println!("{}", misfire);
            env::set_exit_status(misfire.exit_status());
        },
    }
}



struct CharInfo<I> {
    options: Options,
    count: u64,

    input: I,
}

impl<I, E> CharInfo<I>
    where I: Iterator<Item=Result<char, E>>,
          E: fmt::Display {

    fn new(options: Options, iterator: I) -> CharInfo<I> {
        CharInfo {
            count:    if options.bytes { 0 } else { 1 },
            options:  options,
            input:    iterator,
        }
    }

    fn run(mut self) {
        let mut t = term::stdout().unwrap();
        for input in self.input {
            match input {
                Ok(c) => {
                    let char_type = CharType::of(c);

                    if char_type == CharType::Control {
                        t.fg(term::color::GREEN).unwrap();
                    }
                    else if char_type == CharType::Combining {
                        t.fg(term::color::MAGENTA).unwrap();
                    }

                    print!("{:>5}: {} = {}", self.count, CharDisplay(c), NumDisplay(c));

                    if self.options.show_names {
                        if let Some(name) = unicode_names::name(c) {
                            print!(" ({})", name);
                        }
                    }

                    if char_type != CharType::Normal {
                        t.reset().unwrap();
                    }

                    self.count += if self.options.bytes { c.len_utf8() as u64 }
                                                                  else { 1u64 };
                    print!("\n");
                },
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}



struct Options {
    bytes:       bool,
    show_names:  bool,
}

impl Options {
    pub fn getopts(args: &[String]) -> Result<Options, Misfire> {
        let mut opts = getopts::Options::new();
        opts.optflag("b", "bytes",     "show count in number of bytes, not characters");
        opts.optflag("n", "names",     "show unicode name of each character");
        opts.optflag("",  "version",   "display version of program");
        opts.optflag("?", "help",      "show list of command-line options");

        let matches = match opts.parse(args) {
            Ok(m) => m,
            Err(e) => return Err(Misfire::InvalidOptions(e)),
        };

        if matches.opt_present("help") {
            return Err(Misfire::Help(opts.usage("Usage:\n  charinfo [options] < file")))
        }
        else if matches.opt_present("version") {
            return Err(Misfire::Version);
        }

        Ok(Options {
            bytes:       matches.opt_present("bytes"),
            show_names:  matches.opt_present("names"),
        })
    }
}



enum Misfire {
    InvalidOptions(getopts::Fail),
    Help(String),
    Version,
}

impl Misfire {
    pub fn exit_status(&self) -> i32 {
        if let Misfire::Help(_) = *self { 2 }
                                   else { 3 }
    }
}

impl fmt::Display for Misfire {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Misfire::InvalidOptions(ref e) => write!(f, "{}", e),
            Misfire::Help(ref text)        => write!(f, "{}", text),
            Misfire::Version               => write!(f, "charinfo {}", env!("CARGO_PKG_VERSION")),
        }
    }
}



#[derive(PartialEq)]
enum CharType {
    Normal,
    Combining,
    Control,
}

impl CharType {
    fn of(c: char) -> CharType {
        let num = c as u32;

        if c.is_control() {
            CharType::Control
        }
        else if num >= 0x300 && num < 0x370 {
            CharType::Combining
        }
        else {
            CharType::Normal
        }
    }
}



struct CharDisplay(char);

impl fmt::Display for CharDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let number = self.0 as u32;

        if number <= 9 {
            write!(f, " #{} ", number)
        }
        else if number as u32 <= 31 {
            write!(f, " #{}", number)
        }
        else if number >= 0x300 && number < 0x370 {
            write!(f, " ' {}'", self.0)
        }
        else if UnicodeWidthChar::width(self.0) == Some(1) {
            write!(f, " '{}'", self.0)
        }
        else {
            write!(f, "'{}'", self.0)
        }
    }
}


struct NumDisplay(char);

impl fmt::Display for NumDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buffer = [0; 4];  // Four bytes can hold any character
        let bytes_written = self.0.encode_utf8(&mut buffer).unwrap();

        try!(write!(f, "{:0>2x}", buffer[0]));

        for index in 1 .. bytes_written {
            try!(write!(f, " {:0>2x}", buffer[index]));
        }

        Ok(())
    }
}
