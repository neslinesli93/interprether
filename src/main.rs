mod redis;

#[tokio::main]
async fn main() {
    crate::redis::test("ciao").await.unwrap();

    println!("Exiting");
}
