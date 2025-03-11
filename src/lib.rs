//!
//! Rat is a rewrite of the coreutils default program "cat" on Rust programming language.
//! By JerryImMouse
//! 

use std::io::{Read, Write};

static IO_BUFSIZE: usize = 512 * 1024;

const RAT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RAT_NAME: &str = env!("CARGO_PKG_NAME");

static RAT_USAGE: &str = r#"
Usage: rat [OPTION]... [FILE]...
Concatenate FILE(s) to standard output.

With no FILE, or when FILE is -, read standard input.

  -A, --show-all           equivalent to -vET
  -b, --number-nonblank    number nonempty output lines, overrides -n
  -e                       equivalent to -vE
  -E, --show-ends          display $ at end of each line
  -n, --number             number all output lines
  -s, --squeeze-blank      suppress repeated empty output lines
  -t                       equivalent to -vT
  -T, --show-tabs          display TAB characters as ^I
  -u                       (ignored)
  -v, --show-nonprinting   use ^ and M- notation, except for LFD and TAB
      --help        display this help and exit
      --version     output version information and exit

Examples:
  rat f - g  Output f's contents, then standard input, then g's contents.
  rat        Copy standard input to standard output.
"#;

#[derive(Debug)]
enum Source {
    File(String, Option<std::fs::File>),
    Stdin(std::io::Stdin),
    #[cfg(test)]
    Mock(Option<Vec<String>>, usize, String),
}

impl Source {
    fn read_to_buf(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self {
            Source::File(path, file_option) => {
                if file_option.is_none() {
                    let file = std::fs::File::open(path)?;
                    *file_option = Some(file);
                }

                let file = file_option.as_mut().unwrap();

                let bytes_read = file.read(buf)?;
                Ok(bytes_read)
            }
            Source::Stdin(stdin) => {
                let bytes_read = stdin.read(buf)?;
    
                if bytes_read == 0 {
                    return Ok(0); // Properly handle EOF
                }

                Ok(bytes_read)
            },
            #[cfg(test)]
            Source::Mock(lines, pos, s) => {
                if lines.is_none() {
                    let collected_lines: Vec<String> = s.lines().map(|s| s.to_string()).collect();
                    *lines = Some(collected_lines);
                }
            
                let lines = lines.as_ref().unwrap();
            
                if *pos >= lines.len() {
                    return Ok(0);
                }
            
                let line = &lines[*pos];
                
                // TODO
                *pos += 1;
            
                Ok(line.len())
            }            
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::File(s, _) => write!(f, "{s}"),
            Source::Stdin(_) => write!(f, "stdin"),
            #[cfg(test)]
            Source::Mock(..) => write!(f, "mock"),
        }
    }
}

#[derive(Debug, Default)]
pub struct RatArgs {
    // display $ at end of each line
    show_ends: bool,
    // number all output lines
    number_lines: bool,
    // number nonempty output lines, overrides number_lines
    number_nonblank: bool,
    // suppress repeated empty output lines
    squeeze_blank: bool,
    // display TAB characters as ^I
    show_tabs: bool,
    // use ^ and M- notation, except for LFD and TAB
    show_nonprinting: bool,
    // sources to get data from
    files: Vec<Source>,

    // overrides all arguments above...
    version: bool, // show program version
    help: bool, // show help message
}

impl RatArgs {
    pub fn files(files: Vec<String>) -> Self {
        let files = files.iter().map(|f| Source::File(f.to_string(), None)).collect();
        Self {
            files,
            ..Self::default()
        }
    }

    pub fn new(raw: Vec<String>) -> Self {
        let slice = &raw[1..];
        let mut rat_args = RatArgs::default();

        // if no args provided - just use stdin as a source
        if raw.len() == 1 {
            rat_args.files.push(Source::Stdin(std::io::stdin()));
            return rat_args;
        }

        slice.iter().for_each(|arg| {
            if arg.contains("--") && &arg[1..=2] == "--" {
                match arg.as_str() {
                    "--help" => 
                        rat_args.help = true,
                    
                    "--version" => 
                        rat_args.version = true,

                    "--show-tabs" => 
                        rat_args.show_tabs = true,

                    "--number" => 
                        rat_args.number_lines = true,

                    "--number-nonblank" => 
                        rat_args.number_nonblank = true,

                    "--show-ends" => 
                        rat_args.show_ends = true,

                    "--show-nonprinting" => 
                        rat_args.show_nonprinting = true,

                    "--squeeze-blank" =>
                        rat_args.squeeze_blank = true,

                    "--show-all" => {
                        rat_args.show_nonprinting = true;
                        rat_args.show_ends = true;
                        rat_args.show_tabs = true;
                    },

                    _ => {} // TODO: output some warning message, maybe?
                }
            } else if arg == "-" && arg.len() == 1 {
                // stdin source is here baby
                rat_args.files.push(Source::Stdin(std::io::stdin()));
            } else if arg.contains("-") && arg.chars().nth(0).unwrap() == '-' {
                // get all chars as vec
                let chars = arg[1..].chars();
                chars.for_each(|c| {
                    match c {
                        'b' =>
                            rat_args.number_nonblank = true,
                        
                        'E' =>
                            rat_args.show_ends = true,

                        'n' => 
                            rat_args.number_lines = true,

                        's' => 
                            rat_args.squeeze_blank = true,

                        'T' =>
                            rat_args.show_tabs = true,
                        
                        'u' => 
                            todo!(), // tf is this?
                        
                        'v' =>
                            rat_args.show_nonprinting = true,
                        
                        't' => {
                            rat_args.show_tabs = true;
                            rat_args.show_nonprinting = true;
                        },

                        'e' => {
                            rat_args.show_nonprinting = true;
                            rat_args.show_ends = true;
                        },

                        'A' => {
                            rat_args.show_nonprinting = true;
                            rat_args.show_ends = true;
                            rat_args.show_tabs = true;
                        },

                        _ => {}
                    }
                });
            } else {
                rat_args.files
                    .push(Source::File(arg.into(), None));
            }
        });

        rat_args
    }
}

#[derive(Debug)]
pub struct Rat<T: Write> {
    args: RatArgs,
    write_to: T,
}

impl<T: Write> Rat<T> {
    pub fn new(args: RatArgs, write_to: T) -> Self {
        Self { args, write_to }
    }

    pub fn exec(mut self) -> Self {
        let args = &mut self.args;

        if args.help {
            println!("{}", RAT_USAGE);
            return self;
        }

        if args.version {
            println!("{} {}", RAT_NAME, RAT_VERSION);
            return self;
        }

        let mut index = 1u64;

        let mut prev_byte = b'\n';
        let mut buf = [0u8; IO_BUFSIZE];

        // i should explain now, this one exists because of -s flag
        // in original cat.c its logic implented via counting newlines, but i think this is more simple
        let mut prev_prev_byte = b' ';

        for source in self.args.files.iter_mut() {
            loop {
                match source.read_to_buf(&mut buf) {
                    Ok(0) => break,
                    Ok(size) => {
                        let mut out_buf = [0u8; IO_BUFSIZE];
                        let mut out_pos = 0;
                        for byte in &mut buf[..size] {
                            if out_pos >= out_buf.len() {
                                self.write_to.write_all(&out_buf[..out_pos]).unwrap();
                                out_pos = 0; // Reset after flush
                            }
        
                            if self.args.squeeze_blank && *byte == b'\n' && prev_byte == b'\n' && prev_prev_byte == b'\n' {
                                continue;
                            }
                            if ((self.args.number_lines && !self.args.number_nonblank) || (self.args.number_nonblank && *byte != b'\n')) && prev_byte == b'\n' {
                                let num = format!("{index:6} ");
                                out_buf[out_pos..out_pos + num.len()].copy_from_slice(num.as_bytes());
                                out_pos += num.len();
                                index += 1;
                            }
        
                            if self.args.show_nonprinting {
                                if *byte >= 128 {
                                    out_buf[out_pos..out_pos + 2].copy_from_slice(b"M-");
                                    out_pos += 2;
                                    *byte -= 128;
                                }
        
                                if *byte < 32 || *byte == 127 {
                                    out_buf[out_pos] = b'^';
                                    out_buf[out_pos + 1] = *byte ^ 0x40;
                                    out_pos += 2;
                                    continue;
                                }
                            }
        
                            if self.args.show_tabs && *byte == b'\t' {
                                out_buf[out_pos..out_pos + 2].copy_from_slice(b"^I");
                                out_pos += 2;
                            } else {
                                out_buf[out_pos] = *byte;
                                out_pos += 1;
                            }
        
                            prev_prev_byte = prev_byte;
                            prev_byte = *byte;
                        }
                        self.write_to.write_all(&out_buf[..out_pos]).unwrap();
                    }
                    Err(_) => break,
                }
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! rat_args_test {
        ($name:ident, $flag:expr, $($field:ident => $expected:expr),+) => {
            #[test]
            #[allow(non_snake_case)]
            fn $name() {
                let args = vec!["path/to/rat".to_string(), $flag.to_string()];
                let rat_args = RatArgs::new(args);
    
                $(
                    assert_eq!(rat_args.$field, $expected, "Failed on {} for flag {}", stringify!($field), $flag);
                )+

                assert!(rat_args.files.is_empty());
            }
        };
    }

    rat_args_test!(rat_args_E, "-E",
        show_tabs => false,
        show_nonprinting => false,
        show_ends => true,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_b, "-b",
        show_tabs => false,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => true,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_n, "-n",
        show_tabs => false,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => false,
        number_lines => true,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_A, "-A",
        show_tabs => true,
        show_nonprinting => true,
        show_ends => true,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_T, "-T",
        show_tabs => true,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_s, "-s",
        show_tabs => false,
        squeeze_blank => true,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_v, "-v",
        show_tabs => false,
        squeeze_blank => false,
        show_nonprinting => true,
        show_ends => false,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_e, "-e",
        show_tabs => false,
        squeeze_blank => false,
        show_nonprinting => true,
        show_ends => true,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_t, "-t",
        show_tabs => true,
        squeeze_blank => false,
        show_nonprinting => true,
        show_ends => false,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_show_all, "--show-all",
        show_tabs => true,
        show_nonprinting => true,
        show_ends => true,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_number_nonblank, "--number-nonblank",
        show_tabs => false,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => true,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_show_ends, "--show-ends",
        show_tabs => false,
        show_nonprinting => false,
        show_ends => true,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_number, "--number",
        show_tabs => false,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => false,
        number_lines => true,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_squeeze_blank, "--squeeze-blank",
        show_tabs => false,
        squeeze_blank => true,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_show_tabs, "--show-tabs",
        show_tabs => true,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_show_nonprinting, "--show-nonprinting",
        show_tabs => false,
        squeeze_blank => false,
        show_nonprinting => true,
        show_ends => false,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => false
    );

    rat_args_test!(rat_args_help, "--help",
        show_tabs => false,
        squeeze_blank => false,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => false,
        number_lines => false,
        version => false,
        help => true
    );

    rat_args_test!(rat_args_version, "--version",
        show_tabs => false,
        squeeze_blank => false,
        show_nonprinting => false,
        show_ends => false,
        number_nonblank => false,
        number_lines => false,
        version => true,
        help => false
    );
}
