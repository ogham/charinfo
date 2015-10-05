//! Command-line option parsing.

use std::fmt;

use getopts;


/// The usage string, which gets displayed when the user enters invalid
/// command-line options.
static USAGE: &'static str = "Usage:\n  charm [options] file";

/// The **Options** struct represents a parsed version of the user's
/// command-line options.
#[derive(PartialEq, Debug)]
pub struct Options {

    /// The flags that alter what gets displayed.
    pub flags: Flags,

    /// The path of the file to read, or stdout if `None`.
    pub input_file_name: Option<String>,
}

/// The part of the options for individual flags.
#[derive(PartialEq, Debug)]
pub struct Flags {
    pub bytes:           bool,
    pub show_names:      bool,
    pub show_scripts:    bool,
    pub show_widths:     bool,
}

#[allow(unused_results)]
impl Options {

    /// Call getopts on the given slice of command-line strings.
    pub fn getopts(args: &[String]) -> Result<Options, Misfire> {
        let mut opts = getopts::Options::new();
        opts.optflag("b", "bytes",     "show count in number of bytes, not characters");
        opts.optflag("n", "names",     "show unicode name of each character");
        opts.optflag("s", "scripts",   "show script for each character");
        opts.optflag("w", "widths",    "show width for each character");
        opts.optflag("",  "version",   "display version of program");
        opts.optflag("?", "help",      "show list of command-line options");

        let matches = match opts.parse(args) {
            Ok(m)   => m,
            Err(e)  => return Err(Misfire::InvalidOptions(e)),
        };

        if matches.opt_present("help") {
            return Err(Misfire::Help(opts.usage(USAGE)))
        }
        else if matches.opt_present("version") {
            return Err(Misfire::Version);
        }

        // The program can read from either standard input *or* it can read
        // from a file. You can't specify multiple files to read from, that
        // doesn't make sense. Should it act like cat, or what? We already
        // have cat for that!
        let input_file_name = match matches.free.len() {
            0 => None,
            1 => Some(matches.free[0].clone()),
            _ => return Err(Misfire::Help(opts.usage(USAGE))),
        };

        Ok(Options {
            flags: Flags {
                bytes:           matches.opt_present("bytes"),
                show_names:      matches.opt_present("names"),
                show_scripts:    matches.opt_present("scripts"),
                show_widths:     matches.opt_present("widths"),
            },
            input_file_name: input_file_name,
        })
    }
}


/// A thing that could happen instead of running.
pub enum Misfire {

    /// The `getopts` crate didn't like these arguments.
    InvalidOptions(getopts::Fail),

    /// The user asked for help. This contains an autogenerated help string
    /// from the `getopts` crate.
    Help(String),

    /// The user wanted the version number.
    Version,
}

impl Misfire {

    /// The OS exit status that this misfire should signify.
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
            Misfire::Version               => write!(f, "charm {}", env!("CARGO_PKG_VERSION")),
        }
    }
}
