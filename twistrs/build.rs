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
    let mut array_string = String::from("static TLDS:[&'static str; ");
    
    // Calculate how many TLDs we actually have in the dictionary
    match read_lines("./dictionaries/subdomains-10000.txt") {
        Ok(lines) => {
            // We want to unwrap to make sure that we are able to fetch all TLDs
            let tlds = lines.map(|l| l.unwrap()).collect::<Vec<String>>();

            // Finalize the variable signature and break into newline to 
            // start populating the TLDs
            array_string.push_str(&tlds.len().to_string());
            array_string.push_str("] = [\r\n");

            // Start populating TLD contents
            for line in tlds.into_iter() {
                // Formatting some tabs (ASCII-20)
                array_string.push_str("\u{20}\u{20}\u{20}\u{20}\"");
                array_string.push_str(line.to_string().as_str());
                array_string.push_str("\",\r\n");
            }

            // Close off variable signature
            array_string.push_str("];\r\n");

            // Write out contents to the final Rust file artifact
            let out_dir = env::var("OUT_DIR").unwrap();
            let dest_path = Path::new(&out_dir).join("dictionaries.rs");
            fs::write(&dest_path, array_string).unwrap();            
        },
        Err(e) => panic!(format!(
            "unable to build library due to missing dictionary file(s): {}", e
        ))
    }
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