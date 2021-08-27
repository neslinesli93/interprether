use eth_oracle_rs::redis;

#[tokio::main]
async fn main() {
    redis::test("ciao").await.unwrap();

    println!("Exiting");
}
