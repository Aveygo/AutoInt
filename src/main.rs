mod embedding;


fn main() {
    
    
    let embedding = embedding::RustPotion::new();
    println!("{:?}", embedding.encode("test"));


    
}