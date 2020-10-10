
use punycode;

use std::io::{self, BufRead};
use std::{env, fs, path::Path};


fn main() {
    // The following build script converts a number of dictionaries
    // to be embedded directly into the libraries final binaries
    // without incurring any runtime costs.
    // 
    // For more information on the internals as well as other
    // possible solutions, please review the following blog post.
    //
    // https://dev.to/rustyoctopus/generating-static-arrays-during-compile-time-in-rust-10d8
    let mut dicionary_output = String::from("");
    let mut tld_array_string = String::from("static TLDS: [&'static str; ");
    let mut keywords_array_string = String::from("static KEYWORDS: [&'static str; ");
    
    // Calculate how many TLDs we actually have in the dictionary
    match read_lines("./dictionaries/tlds.txt") {
        Ok(lines) => {
            // We want to unwrap to make sure that we are able to fetch all TLDs
            let tlds = lines.map(|l| l.unwrap()).collect::<Vec<String>>();

            // Finalize the variable signature and break into newline to 
            // start populating the TLDs
            tld_array_string.push_str(&tlds.len().to_string());
            tld_array_string.push_str("] = [\r\n");

            // Start populating TLD contents
            for line in tlds.into_iter() {
                // Formatting some tabs (ASCII-20)
                tld_array_string.push_str("\u{20}\u{20}\u{20}\u{20}\"");

                let tld;

                if line.chars().all(char::is_alphanumeric) {
                    tld = line.to_string();
                } else {
                    tld = punycode::encode(line.to_string().as_str()).unwrap()
                }

                tld_array_string.push_str(&tld[..]);
                tld_array_string.push_str("\",\r\n");
            }

            // Close off variable signature
            tld_array_string.push_str("];\r\n");     
        },
        Err(e) => panic!(format!(
            "unable to build library due to missing dictionary file(s): {}", e
        ))
    }

    match read_lines("./dictionaries/keywords.txt") {
        Ok(lines) => {
            // We want to unwrap to make sure that we are able to fetch all TLDs
            let tlds = lines.map(|l| l.unwrap()).collect::<Vec<String>>();

            // Finalize the variable signature and break into newline to 
            // start populating the TLDs
            keywords_array_string.push_str(&tlds.len().to_string());
            keywords_array_string.push_str("] = [\r\n");

            // Start populating TLD contents
            for line in tlds.into_iter() {
                // Formatting some tabs (ASCII-20)
                keywords_array_string.push_str("\u{20}\u{20}\u{20}\u{20}\"");

                let tld;

                if line.chars().all(char::is_alphanumeric) {
                    tld = line.to_string();
                } else {
                    tld = punycode::encode(line.to_string().as_str()).unwrap()
                }

                keywords_array_string.push_str(&tld[..]);
                keywords_array_string.push_str("\",\r\n");
            }

            // Close off variable signature
            keywords_array_string.push_str("];\r\n");
        },
        Err(e) => panic!(format!(
            "unable to build library due to missing dictionary file(s): {}", e
        ))
    }
    
    // Start building the final output
    dicionary_output.push_str(&tld_array_string);
    dicionary_output.push_str("\n");
    dicionary_output.push_str(&keywords_array_string);

    // Write out contents to the final Rust file artifact
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("dictionaries.rs");
    fs::write(&dest_path, dicionary_output).unwrap();       
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
//
// This was taken from the official rust-lang docs:
// https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where P: AsRef<Path>, {
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}