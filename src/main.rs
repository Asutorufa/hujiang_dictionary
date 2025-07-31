use hj_rust::jp;

#[tokio::main]
async fn main() {
    println!("{:?}", jp::get("疎か", "jc").await.unwrap());
}
