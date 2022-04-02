use risclient::*;

#[tokio::main]
async fn main() {
    println!("Creating RIS client");
    let mut client = match RisClient::default() {
	Ok(client) => client,
	Err(e) => panic!("failed to create client: {:?}", e)
    };
    println!("Connecting to stream");
    let rx = match client.stream().await {
	Ok(tup) => tup,
	Err(e) => panic!("Failed to stream RIS messages: {:?}", e)
    };
    println!("Streaming responses");
    loop {
	let data = match rx.recv() {
	    Ok(message) => message,
	    Err(e) => panic!("receive error: {:?}", e)
	};
	println!("message: {:?}\r", data);
    }
}
