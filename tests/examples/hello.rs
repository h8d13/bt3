use balanced_ternary::terscii;

const MESSAGE: &str = "Hello, World!";

fn main() {
    let encoded: Vec<_> = MESSAGE.chars()
        .map(|c| terscii::encode_tryte(c).unwrap())
        .collect();
    println!("encoded: {}", encoded.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" "));

    let decoded: String = encoded.iter()
        .map(|&t| terscii::decode_tryte(t).unwrap())
        .collect();
    println!("decoded: {}", decoded);
}
