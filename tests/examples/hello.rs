use balanced_ternary::terscii;

const MESSAGE: &str = "Hello, World!";

fn main() {
    let encoded = terscii::encode_str(MESSAGE).unwrap();
    let unbalanced = terscii::unbalanced_str(&encoded);

    let decoded = terscii::decode_codes(&encoded).unwrap();

    let balanced = terscii::balanced_str(&encoded);

    println!("encoded (unbalanced): {unbalanced}");
    println!("encoded (balanced):   {balanced}");
    println!("decoded:              {decoded}");
}
