struct Body {
    iss: String,
    nbf: i32,
    vc: ...,
}

fn main() {
    let img = image::open("/Users/jsvana/Desktop/ca_qr.png").unwrap();

    // convert to gray scale
    let img_gray = img.into_luma8();

    // create a decoder
    let mut decoder = quircs::Quirc::default();

    // identify all qr codes
    let codes = decoder.identify(
        img_gray.width() as usize,
        img_gray.height() as usize,
        &img_gray,
    );

    for code in codes {
        let code = code.expect("failed to extract qr code");
        let decoded = code.decode().expect("failed to decode qr code");
        let code = std::str::from_utf8(&decoded.payload).unwrap();

        let parts: Vec<&str> = code.split('/').collect();

        let digits = parts.get(1).expect("malformed token");

        let mut letters: Vec<char> = Vec::new();

        for chunk in digits.chars().collect::<Vec<char>>().chunks(2) {
            let chunk: String = chunk.iter().collect();
            let chunk_as_int = chunk.parse::<u32>().expect("failed to parse int");

            letters.push(std::char::from_u32(chunk_as_int + 45).expect("unknown char"));
        }

        let output: String = letters.into_iter().collect();

        let parts: Vec<&str> = output.split('.').collect();
        let contents = base64::decode_config(parts[1], base64::URL_SAFE_NO_PAD)
            .expect("failed to decode base64");
        let decoded = inflate::inflate_bytes(&contents).unwrap();
        let decoded = std::str::from_utf8(&decoded).unwrap();

        println!("{}", decoded);
    }
}
