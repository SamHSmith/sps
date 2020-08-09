fn main() {
    println!("Hello from rust!");
    #[cfg(feature = "joe")]{
        println!("Hello from joe aswell.");
    }
}
