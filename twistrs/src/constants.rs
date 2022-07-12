use fancy_regex::Regex;
use phf::phf_map;

#[cfg(feature = "whois_lookup")]
use whois_rust::WhoIs;

use hyper::client::Client;
use hyper::client::HttpConnector;

// Include further constants such as dictionaries that are
// generated during compile time.
include!(concat!(env!("OUT_DIR"), "/data.rs"));

lazy_static! {

    /// IDNA filter regex used to reduce number of domain permutations
    /// that are generated and validated.
    ///
    /// The regex is taken from [dnstwist](https://github.com/elceef/dnstwist/blob/5368e465c35355c43d189b093acf41773e869d25/dnstwist.py#L213-L227).
    pub static ref IDNA_FILTER_REGEX: Regex = Regex::new("(?=^.{4,253}$)(^((?!-)[a-zA-Z0-9-]{1,63}(?<!-)\\.)+[a-zA-Z]{2,63}\\.?$)").unwrap();

    pub static ref KEYBOARD_LAYOUTS: Vec<&'static phf::Map<char, &'static str>> = vec![
        &QWERTY_KEYBOARD_LAYOUT,
        &QWERTZ_KEYBOARD_LAYOUT,
        &AZERTY_KEYBOARD_LAYOUT
    ];


    /// Global HTTP client we use throughout the library
    pub static ref HTTP_CLIENT: Client<HttpConnector> = Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .http2_only(false)
        .http1_read_buf_exact_size(1024)
        .retry_canceled_requests(false)
        .build(http_connector());
}

// This is currently a bit annoying, however since the WHOIS lookup table
// is build at runtime, and is feature-gated, we cannot have this activated
// within the original lazy_static! macro. We would need to block the
// entire macro behind the feature gate instead.
#[cfg(feature = "whois_lookup")]
lazy_static! {
    pub static ref WHOIS: WhoIs = WhoIs::from_string(WHOIS_RAW_JSON).unwrap();
}

/// Internal helper to create an HTTP Connector
fn http_connector() -> HttpConnector {
    let mut c = HttpConnector::new();
    c.set_recv_buffer_size(Some(1024));
    c.set_connect_timeout(Some(std::time::Duration::new(5, 0)));
    c.enforce_http(true);
    c
}

/// Static list of lowercase ASCII characters.
pub static ASCII_LOWER: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

static QWERTY_KEYBOARD_LAYOUT: phf::Map<char, &'static str> = phf_map! {
    '1' => "2q",
    '2' => "3wq1",
    '3' => "4ew2",
    '4' => "5re3",
    '5' => "6tr4",
    '6' => "7yt5",
    '7' => "8uy6",
    '8' => "9iu7",
    '9' => "0oi8",
    '0' => "po9",
    'q' => "12wa",
    'w' => "3esaq2",
    'e' => "4rdsw3",
    'r' => "5tfde4",
    't' => "6ygfr5",
    'y' => "7uhgt6",
    'u' => "8ijhy7",
    'i' => "9okju8",
    'o' => "0plki9",
    'p' => "lo0",
    'a' => "qwsz",
    's' => "edxzaw",
    'd' => "rfcxse",
    'f' => "tgvcdr",
    'g' => "yhbvft",
    'h' => "ujnbgy",
    'j' => "ikmnhu",
    'k' => "olmji",
    'l' => "kop",
    'z' => "asx",
    'x' => "zsdc",
    'c' => "xdfv",
    'v' => "cfgb",
    'b' => "vghn",
    'n' => "bhjm",
    'm' => "njk"
};

static QWERTZ_KEYBOARD_LAYOUT: phf::Map<char, &'static str> = phf_map! {
    '1'=> "2q",
    '2'=> "3wq1",
    '3'=> "4ew2",
    '4'=> "5re3",
    '5'=> "6tr4",
    '6'=> "7zt5",
    '7'=> "8uz6",
    '8'=> "9iu7",
    '9'=> "0oi8",
    '0'=> "po9",
    'q'=> "12wa",
    'w'=> "3esaq2",
    'e'=> "4rdsw3",
    'r'=> "5tfde4",
    't'=> "6zgfr5",
    'z'=> "7uhgt6",
    'u'=> "8ijhz7",
    'i'=> "9okju8",
    'o'=> "0plki9",
    'p'=> "lo0",
    'a'=> "qwsy",
    's'=> "edxyaw",
    'd'=> "rfcxse",
    'f'=> "tgvcdr",
    'g'=> "zhbvft",
    'h'=> "ujnbgz",
    'j'=> "ikmnhu",
    'k'=> "olmji",
    'l'=> "kop",
    'y'=> "asx",
    'x'=> "ysdc",
    'c'=> "xdfv",
    'v'=> "cfgb",
    'b'=> "vghn",
    'n'=> "bhjm",
    'm'=> "njk"
};

static AZERTY_KEYBOARD_LAYOUT: phf::Map<char, &'static str> = phf_map! {
    '1'=> "2a",
    '2'=> "3za1",
    '3'=> "4ez2",
    '4'=> "5re3",
    '5'=> "6tr4",
    '6'=> "7yt5",
    '7'=> "8uy6",
    '8'=> "9iu7",
    '9'=> "0oi8",
    '0'=> "po9",
    'a'=> "2zq1",
    'z'=> "3esqa2",
    'e'=> "4rdsz3",
    'r'=> "5tfde4",
    't'=> "6ygfr5",
    'y'=> "7uhgt6",
    'u'=> "8ijhy7",
    'i'=> "9okju8",
    'o'=> "0plki9",
    'p'=> "lo0m",
    'q'=> "zswa",
    's'=> "edxwqz",
    'd'=> "rfcxse",
    'f'=> "tgvcdr",
    'g'=> "yhbvft",
    'h'=> "ujnbgy",
    'j'=> "iknhu",
    'k'=> "olji",
    'l'=> "kopm",
    'm'=> "lp",
    'w'=> "sxq",
    'x'=> "wsdc",
    'c'=> "xdfv",
    'v'=> "cfgb",
    'b'=> "vghn",
    'n'=> "bhj"
};

pub static HOMOGLYPHS: phf::Map<char, &'static str> = phf_map! {
    'a' => "àáâãäåɑạǎăȧą",
    'b' => "dʙɓḃḅḇƅ",
    'c' => "eƈċćçčĉo",
    'd' => "bɗđďɖḑḋḍḏḓ",
    'e' => "céèêëēĕěėẹęȩɇḛ",
    'f' => "ƒḟ",
    'g' => "qɢɡġğǵģĝǧǥ",
    'h' => "ĥȟħɦḧḩⱨḣḥḫẖ",
    'i' => "1líìïıɩǐĭỉịɨȋī",
    'j' => "ʝɉ",
    'k' => "ḳḵⱪķ",
    'l' => "1iɫł",
    'm' => "nṁṃᴍɱḿ",
    'n' => "mrńṅṇṉñņǹňꞑ",
    'o' => "0ȯọỏơóö",
    'p' => "ƿƥṕṗ",
    'q' => "gʠ",
    'r' => "ʀɼɽŕŗřɍɾȓȑṙṛṟ",
    's' => "ʂśṣṡșŝš",
    't' => "ţŧṫṭțƫ",
    'u' => "ᴜǔŭüʉùúûũūųưůűȕȗụ",
    'v' => "ṿⱱᶌṽⱴ",
    'w' => "ŵẁẃẅⱳẇẉẘ",
    'y' => "ʏýÿŷƴȳɏỿẏỵ",
    'z' => "ʐżźᴢƶẓẕⱬ"
};

pub static VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
