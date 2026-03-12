use balanced_ternary::terscii;

const MESSAGE: &str = "Hello, World!";

fn main() {
    let encoded = terscii::encode_str(MESSAGE).unwrap();
    let decoded = terscii::decode_codes(&encoded).unwrap();

    let repr: Vec<String> = encoded.iter().map(|c| c.to_string()).collect();
    println!("original: {MESSAGE}");
    println!("encoded:  {}", repr.join(" "));
    println!("decoded:  {decoded}");
}
