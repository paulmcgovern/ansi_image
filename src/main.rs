// TODO:
// When file is a symlink? How to get length, etc. Will img lib read symlink?
// Process more than one image file.
// Output font size. default 8x16

//
// if let Err(err) = response {
//    log::error!("Failed to execute query: {}", err);
// }
//

// Hapiness 喜

use unicode_segmentation::UnicodeSegmentation;
use terminal_size::{Width, Height, terminal_size};
use std::env;
use std::fs;
use std::error::Error;

use log::LevelFilter;
use log::debug;
use simple_logger::SimpleLogger;

use getopts::Options;

use ansi_image::ColorChar; 
use ansi_image::DisplayMode;
use ansi_image::AnsiImageError;

fn print_help(opts: Options) {
    let summary = "Usage: ansi_image [OPTION] [IMAGE FILE]";
    print!("{}", opts.usage(&summary));
}

// Panics unless filename is a regular
// file of more than zero bytes.
fn check_file(filename: &str) -> Result<u64,Box<dyn Error>> {

    let meta = fs::metadata(filename)?;

    if meta.file_type().is_dir() {
        return Err(Box::new(AnsiImageError::new("Filename is a directory.")));
    }

    let len = meta.len();

    if len == 0 {
        return Err(Box::new(AnsiImageError::new("File is zero-length.")));        
    }

    Ok(len)
}

fn get_terminal_size() -> Result<(u16, u16),AnsiImageError> {

    match terminal_size() {
        Some((Width(w), Height(h))) => Ok((w, h)),
        None => Err(AnsiImageError::new("Failed to get terminal dimensions."))
    }
}

// Get command-line param as integer. Panics if bad value.
fn get_int_arg(arg_name: &str, matches: &getopts::Matches, default: u16) -> u16 {

    let mut val = default;
    
    match matches.opt_str(arg_name) {
        Some(w) => {
            match w.parse::<u16>() {
                Ok(v) => val = v,
                Err(e) => panic!("Bad value for {}: {}", arg_name, e)
            }
        },
        None => ()
    }

    val
}


fn get_display_mode(matches: &getopts::Matches) -> u8 {

    // Set display mode flags.
    let mut mode:u8 = 0;

    if matches.opt_present("c"){
   
        mode = mode | match matches.opt_present("r") {
            true => DisplayMode::REVERSE as u8,
            false => DisplayMode::NORMAL as u8,
        };
   
    } else {
   
        mode = mode | DisplayMode::REVERSE as u8;
    }
   
    mode = mode | match matches.opt_present("b"){
        true => DisplayMode::BLINK as u8,
        false => DisplayMode::NORMAL as u8
    };

    mode
}


fn main() -> Result<(), AnsiImageError> {

    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("", "help", "How to use this program.");
    opts.optflag("v", "verbose", "Verbose output");
    opts.optflag("r", "", "Background reverse");
    opts.optflag("b", "", "Blink");
    opts.optopt("x", "", "Output width, in characters.", "NUMBER");
    opts.optopt("y", "", "Output height, in characters.", "NUMBER");
    opts.optopt("c", "", "Character", "CHARACTER");
    opts.optopt("", "font-width", "Font width, in pixels", "NUMBER");
    opts.optopt("", "font-height", "Font height, in pixels", "NUMBER");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(_) => { panic!("Unable to parse arguments.") }
    };

    // Set logging level to Debug if
    // the verbose flag is set.
    SimpleLogger::new()
    .with_level(match matches.opt_present("v"){
        true => LevelFilter::Debug,
        false => LevelFilter::Error
    }).init()
    .unwrap();


    if matches.opt_present("help") {
        print_help(opts);
        return Ok(());
    }

    if matches.free.is_empty() {
        panic!("No image filename in arguments.");
    }

    let out_char = match matches.opt_str("c"){
        Some(c) => c,
        None => String::from(ColorChar::DEFAULT_CHAR)
    };

    // UTF-8 characters are more than one
    // byte, so go by grapheme count
    if out_char.graphemes(true).count() > 1 {
        panic!("Bad value for character: {}", out_char)
    }



    let filename = &matches.free[0];

    debug!("Image file: '{}'", filename);

    // File must exist, be a regular
    // file, and more than zero bytes
    if let Err(e) = check_file(&filename){
        panic!("{}", e);
    }
    

    let result = get_terminal_size();
    
    if let Err(e) = result {
        panic!("{}", e);
    }

    let term_dims = result.unwrap();
    
    debug!("Terminal is {} x {} characters.", term_dims.0, term_dims.1);
    
    // Get output width and height, in characters.
    // Default to console size.
    let out_width = get_int_arg("x", &matches, term_dims.0);
    let out_height = get_int_arg("y", &matches, term_dims.1);

    debug!("Output {} x {} characters", out_width, out_height);

    // Get character size in terminal.
    // Defaults to 8x16 pixels.
    let font_w_px = get_int_arg("font-width", &matches, 8);
    let font_h_px = get_int_arg("font-height", &matches, 16);
    
    let ansi_mode = get_display_mode(&matches);

    let img_str = match ansi_image::img_to_ansi(filename, 
                                          out_width.into(),
                                          out_height.into(),
                                          font_w_px.into(), 
                                          font_h_px.into(),
                                          &out_char,
                                          ansi_mode) {
        Ok(s) => s,
        Err(e) => panic!("{}", e)
    };

    println!("{}", img_str);

    Ok(())
}

