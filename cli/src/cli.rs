use getopts::Options;

pub struct Cli {
    /// Default: false. Activates debug outputs.
    pub debug: bool,

    /// Default: false. Activates verbose outputs.
    pub verbose: bool,

    /// Default: false. Input PNG may have errors. However, in case an error is encountered, the program will proceed by default till the end as long as it does not panic.
    /// If true, the program will panic on the first error.
    pub fail_fast: bool,

    /// Each PNG Chunk has last 4 bytes as a CRC value, which is based on
    /// the preceding chunk type and chunk data bytes of itself and is used to
    /// check if the chunk is corrupt. Validating it will decrease the performance but can find out corrupt chunks or CRC value.
    ///
    /// Default: `true`. Will be `should_validate_crc` in parser.rs.
    pub validate_crc: bool,

    /// Required. Path to the input PNG file.
    // todo change to PathBuf
    input_file: Option<String>,
}

/// There are better options out there,
/// but using getopts significantly reduces the binary size
impl Cli {
    pub fn new() -> Self {
        return Cli {
            debug: false,
            verbose: false,
            fail_fast: false,
            validate_crc: true,
            input_file: None,
        };
    }

    pub fn init(&mut self) {
        let args: Vec<String> = std::env::args().collect();

        let mut opts = Options::new();
        opts.optflag("f", "fail-fast", "[Default]: false. Input PNG may have multiple errors. In case an error is encountered, the program will proceed by default till the end as long as it does not panic. If this flag is supplied, the program will stop upon first error. 
        If true, the program will panic on the first error.");
        opts.optflag(
            "v",
            "verbose",
            "[Default]: false. Activates verbose outputs.",
        );
        opts.optflag(
            "d",
            "debug",
            "[Default]: false. Activates debug outputs.",
        );
        opts.optflagopt(
            "",
            "validate-crc",
            "[Default]: true. Validates crc. Takes more time to finish the program.",
            "false | true",
        );
        opts.optflag(
            "h",
            "help",
            "print this help menu",
        );
        opts.reqopt(
            "i",
            "input",
            "[Required] Path to the input PNG file.",
            "PATH_TO_PNG_FILE",
        );

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                println!("{}", f.to_string());
                self.print_usage(opts);
                std::process::exit(1);
            }
        };

        if matches.opt_present("h") {
            self.print_usage(opts);
            std::process::exit(1);
        }

        println!(
            "
    /_____/\\/__/\\ /__/\\/______/\\         /_____/\\ /_____/\\     
    \\:::_ \\ \\::\\_\\\\  \\ \\::::__\\/__ ______\\:::_ \\ \\\\::::_\\/_    
     \\:(_) \\ \\:. `-\\  \\ \\:\\ /____//______/\\:(_) ) )\\:\\/___/\\   
      \\: ___\\/\\:. _    \\ \\:\\\\_  _\\\\__::::\\/\\: __ `\\ \\_::._\\:\\  
       \\ \\ \\   \\. \\`-\\  \\ \\:\\_\\ \\ \\         \\ \\ `\\ \\ \\/____\\:\\ 
        \\_\\/    \\__\\/ \\__\\/\\_____\\/          \\_\\/ \\_\\/\\_____\\/ 
        "
        );
        self.verbose = matches.opt_present("v");
        self.debug = matches.opt_present("d");
        self.fail_fast = matches.opt_present("f");
        match matches
            .opt_str("validate-crc")
            .unwrap_or("".to_string())
            .as_str()
        {
            "false" => self.validate_crc = false,
            "true" | "" => self.validate_crc = true,
            _ => {
                println!("--validate-crc should be either 'false', 'true' or should not be supplied (implies true)");
                self.print_usage(opts);
                std::process::exit(1);
            }
        };
        self.input_file = matches.opt_str("i");
    }

    pub fn get_input_file_path(&self) -> &str {
        return &self
            .input_file
            .as_ref()
            .expect("Input file must be initialized");
    }

    fn print_usage(&self, opts: Options) {
        let brief = format!("Usage: png-rs [-i|--input PATH_TO_PNG_FILE] [-h|--help] [-v|--verbose] [-d|--debug] [-f|--fail-fast] [--validate-crc false|true]");
        print!("{}", opts.usage(&brief));
    }
}
