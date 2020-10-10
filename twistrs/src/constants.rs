use phf::phf_map;
use fancy_regex::Regex;
use publicsuffix::List;

/// Static list of lowercase ASCII characters.
// Stack allocate these at compile time
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

lazy_static! {

    /// IDNA filter regex used to reduce number of domain permutations
    /// that are generated and validated.
    /// 
    /// The regex is taken from [dnstwist](https://github.com/elceef/dnstwist/blob/5368e465c35355c43d189b093acf41773e869d25/dnstwist.py#L213-L227).
    pub static ref IDNA_FILTER_REGEX: Regex = Regex::new("(?=^.{4,253}$)(^((?!-)[a-zA-Z0-9-]{1,63}(?<!-)\\.)+[a-zA-Z]{2,63}\\.?$)").unwrap();

    // @CLEANUP(jdb): Right now this is going to always incur a runtime
    //                overhead since we need to always fetch the list
    //                over HTTP. In the future, this should be kept lo-
    //                cally and call List::from_str instead.
    //
    //                At first we used List::from_path and pointed to a
    //                local .dat file containing the TLDs, however giv-
    //                en that this is a library, this is not the right
    //                way to go about it.
    //
    //  Ref: https://docs.rs/publicsuffix/1.5.4/publicsuffix/struct.List.html
    pub static ref EFFECTIVE_TLDS: List =
        List::fetch().unwrap();

    pub static ref KEYBOARD_LAYOUTS: Vec<&'static phf::Map<char, &'static str>> = vec![
        &QWERTY_KEYBOARD_LAYOUT,
        &QWERTZ_KEYBOARD_LAYOUT,
        &AZERTY_KEYBOARD_LAYOUT
    ];
}
