use std::io::{self, BufRead};
use std::{env, fs, path::Path};

fn main() {
    // The following build script converts a number of data assets
    // to be embedded directly into the libraries final binaries
    // without incurring any runtime costs.
    //
    // For more information on the internals as well as other
    // possible solutions, please review the following blog post.
    //
    // https://dev.to/rustyoctopus/generating-static-arrays-during-compile-time-in-rust-10d8
    let mut dicionary_output = String::from("");

    let mut tld_array_string = String::from(
        "#[allow(dead_code)]
                                                  static TLDS: [&str; ",
    );
    let mut keywords_array_string = String::from(
        "#[allow(dead_code)]
                                                 static KEYWORDS: [&str; ",
    );
    let mut whois_servers_string = String::from(
        "#[allow(dead_code)]
                                                  static WHOIS_RAW_JSON: &str = r#",
    );

    // Calculate how many TLDs we actually have in the dictionary
    match read_lines("./data/tlds.txt") {
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

                let tld = if line.chars().all(char::is_alphanumeric) {
                    line.to_string()
                } else {
                    punycode::encode(line.to_string().as_str()).unwrap()
                };

                tld_array_string.push_str(&tld[..]);
                tld_array_string.push_str("\",\r\n");
            }

            // Close off variable signature
            tld_array_string.push_str("];\r\n");
        }
        Err(e) => panic!(
            "{}",
            format!(
                "unable to build library due to missing dictionary file(s): {}",
                e
            )
        ),
    }

    match read_lines("./data/keywords.txt") {
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

                let tld = if line.chars().all(char::is_alphanumeric) {
                    line.to_string()
                } else {
                    punycode::encode(line.to_string().as_str()).unwrap()
                };

                keywords_array_string.push_str(&tld[..]);
                keywords_array_string.push_str("\",\r\n");
            }

            // Close off variable signature
            keywords_array_string.push_str("];\r\n");
        }
        Err(e) => panic!(
            "{}",
            format!(
                "unable to build library due to missing dictionary file(s): {}",
                e
            )
        ),
    }

    // Compile the WhoIs server config to later perform WhoIs lookups against
    match read_lines("./data/whois-servers.json") {
        Ok(lines) => {
            // Construct the in-memory JSON
            whois_servers_string.push('"');
            lines.for_each(|l| whois_servers_string.push_str(&l.unwrap()));
            whois_servers_string.push_str("\"#;");
        }
        Err(e) => panic!(
            "{}",
            format!(
                "unable to build library due to missing dictionary file(s): {}",
                e
            )
        ),
    }

    // Build the final output
    dicionary_output.push_str(&tld_array_string);
    dicionary_output.push('\n');
    dicionary_output.push_str(&keywords_array_string);
    dicionary_output.push('\n');
    dicionary_output.push_str(&whois_servers_string);

    // Write out contents to the final Rust file artifact
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("data.rs");
    fs::write(dest_path, dicionary_output).unwrap();
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
//
// This was taken from the official rust-lang docs:
// https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
