//! Trafilatura Rust Port - Main Binary
//!
//! A command-line tool for extracting content from web pages, inspired by
//! Python's trafilatura library. This binary provides a CLI interface to the
//! trafilatura-rs library for text extraction.

use std::process;
use log::{info, debug, error};
use env_logger::Env;

use trafilatura::{
    cli::{build_cli, parse_args, process_input, format_result, write_output}
};

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Parse command-line arguments
    let matches = build_cli().get_matches();
    
    // Set log level
    if matches.get_flag("verbose") {
        log::set_max_level(log::LevelFilter::Debug);
    }
    
    // Parse arguments and handle errors
    let (config, input_source, output_file) = match parse_args(&matches) {
        Ok(result) => result,
        Err(err) => {
            error!("Error parsing arguments: {}", err);
            process::exit(1);
        }
    };
    
    debug!("Configuration: {:?}", config);
    debug!("Processing input: {}", input_source);
    
    // Process the input
    match process_input(&config, &input_source) {
        Ok(result) => {
            info!("Successfully extracted content");
            debug!("Extraction result: {:?}", result);
            
            // Format the result according to the output format
            let formatted_output = format_result(&result, config.output_format);
            
            // Write the output
            if let Err(err) = write_output(&formatted_output, output_file) {
                error!("Error writing output: {}", err);
                process::exit(1);
            }
        },
        Err(err) => {
            error!("Error extracting content: {}", err);
            process::exit(1);
        }
    }
}
