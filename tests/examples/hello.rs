use balanced_ternary::{terscii};

const MESSAGE: &str = "Hello, World!";

fn main() {
    // TERSCII: encode a string to balanced ternary trytes, decode back
    // only supports 0-81 ascii range
    let encoded = terscii::encode_str(MESSAGE).unwrap();
    println!("\nstring operations:");
    println!("  encoded:  {encoded}");
    let decoded = terscii::decode_str(&encoded).unwrap();
    println!("  decoded:  {decoded}");
}
