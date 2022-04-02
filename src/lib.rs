use std::error;

use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use std::sync::mpsc::{channel, Receiver};

#[macro_use] extern crate serde_derive;

fn default_timestamp() -> f32 {
    0.0
}

fn default_unknown_string() -> String {
    "unknown".to_string()
}

/// Represents the data portion of a response from the RIS API
/// Not all messages have data, such as the Ping/Pong messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RisResponseData {
    #[serde(default="default_timestamp")]
    timestamp: f32,
    #[serde(default="default_unknown_string")]
    peer: String,
    #[serde(default="default_unknown_string")]
    peer_asn: String,
    #[serde(default="default_unknown_string")]
    id: String,
    #[serde(default="default_unknown_string")]
    host: String,
    #[serde(rename = "type")]
    #[serde(default="default_unknown_string")]
    data_type: String
}

impl Default for RisResponseData {
    fn default() -> RisResponseData{
	RisResponseData {
	    timestamp: default_timestamp(),
	    peer: default_unknown_string(),
	    peer_asn: default_unknown_string(),
	    id: default_unknown_string(),
	    host: default_unknown_string(),
	    data_type: default_unknown_string(),
	}
    }
}

/// Represents the data portion of a request to the RIS API
/// Not all requests require data, so this is optional
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RisRequestData {
    host: Option<String>,
    #[serde(rename = "type")]
    data_type: Option<String>,
    require: Option<String>,
    path: Option<Vec<u32>> 
}


/// Represents a response from the RIS API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RisResponse {
    #[serde(default="default_unknown_string")]
    #[serde(rename = "type")]
    message_type: String,
    data: RisResponseData,
}

/// Represents a request to the RIS API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RisRequest {
    #[serde(rename = "type")]
    message_type: String,
    data: Option<RisRequestData>,
}

/// Represents a RIS client
pub struct RisClient {
    host: String,
    client_id: String,
}	
    
///
/// A base client for RIS Live
/// This handles the basic abstraction of connection and streaming from
/// RIS Live via the websocket interface.
///
impl RisClient {

    /// Returns a RisClient for the provided host, and uses the provided client_id in all requests
    ///
    /// # Arguments
    ///
    /// * `host` - A string which indicates the host to connect todo
    /// * `client_id` - A custom identifier for your client, this is useful for RIPE tracking and support purposes
    ///
    /// # Examples
    ///
    /// ```
    /// use risclient::RisClient;
    /// let client = RisClient::new("ris-live.ripe.net", "rust-risclient");
    /// ```    
    pub fn new(host: String, client_id: String) -> Result<RisClient, Box<dyn error::Error>> {
	Ok(RisClient {
	    host,
	    client_id,
	})
    }

    /// Returns a RisClient using the default host and client_id
    ///
    /// # Examples
    ///
    /// ```
    /// use risclient::RisClient;
    /// let client = RisClient::default();
    /// ```    
    pub fn default() -> Result<RisClient, Box<dyn error::Error>> {
	Ok(RisClient {
	    host: "ris-live.ripe.net".to_string(),
	    client_id: "rust-risclient".to_string(),
	})
    }

    /// Returns an async iterator of streamed RIS messages, using the provided filters.
    /// If you would like the full stream, you should use the `stream` method instead, to save yourself time.
    ///
    /// # Arguments
    ///
    /// * `host` - Optionally return messages for this RIS collector only. For a list of collectors, see here: https://www.ripe.net/analyse/internet-measurements/routing-information-service-ris/ris-raw-data
    /// * `data_type` - Optionally return messages of this type only. The API accepts "UPDATE", "OPEN", "NOTIFICATION", "KEEPALIVE" and "RIS_PEER_STATE".
    /// * `require` - Optionally filter on announcements or withdrawl messages. The API accepts "announcement" or "withdrawls". Set to `None` to return both.
    /// * `path` - Optionally return messages about the provided path. Set to `None` to return messages for all paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use risclient::RisClient;
    /// let client = RisClient::default();
    /// let rx = client.stream_custom(Some("rrc16".to_string()), None, None, None);
    /// loop {
    ///    let data = match rx.recv() {
    ///        Ok(message) => message,
    ///        Err(e) => panic!("receive error: {:?}", e)
    ///    };
    ///    println!("message: {:?}\r", data);
    /// }
    /// ```    
    pub async fn stream_custom(&mut self, host: Option<String>, data_type: Option<String>, require: Option<String>, path: Option<Vec<u32>>) -> Result<Receiver<RisResponse>, Box<dyn error::Error>> {
	let url = format!("wss://{}/v1/ws/?client={}", self.host, self.client_id);
	let handle = connect_async(url).await;
	match handle {
	    Ok(handle) => {
		let request = RisRequest {
		    message_type: "ris_subscribe".to_string(),
		    data: Some(RisRequestData {
			host,
			data_type,
			require,
			path,
		    })
		};
		let (mut tx, _) = handle;
		let message = match serde_json::to_string(&request) {
		    Ok(message) => message,
		    Err(e) => return Err(Box::new(e))
		};
		match tx.send(message.into()).await {
		    Ok(_) => {
			let (ctx, crx) = channel();
			let _result = tokio::spawn(async move {
			    while let Some(msg)= tx.next().await {
				match msg {
				    Ok(msg) => {
					let message = msg.to_string();
					let data: RisResponse = match serde_json::from_str(&message) {
					    Ok(data) => data,
					    // eof happens all the time, this usually means an empty line which won't parse as JSON
					    Err(ref e) if e.is_eof() => continue,
					    Err(e) => panic!("failed decoding message: {:?}, '{}'", e, message),
					};
					match ctx.send(data) {
					    Ok(_) => continue,
					    Err(e) => panic!("failed to send decoded message to channel: {:?}", e)
					}
				    },
				    Err(e) => panic!("failed to decode message: {:?}", e),
				}
			    }
			});
			Ok(crx)
		    },
		    Err(e) => Err(Box::new(e))
		}
	    },
	    Err(e) => Err(Box::new(e))
	}
    }

    /// Returns an async iterator of streamed RIS messages, with no filters.
    /// This is equivalent to calling `stream_custom(None, None, None, None)`
    ///
    /// # Examples
    ///
    /// ```
    /// use risclient::RisClient;
    /// let client = RisClient::default();
    /// let rx = client.stream();
    /// loop {
    ///    let data = match rx.recv() {
    ///        Ok(message) => message,
    ///        Err(e) => panic!("receive error: {:?}", e)
    ///    };
    ///    println!("message: {:?}\r", data);
    /// }
    /// ```    
    pub async fn stream(&mut self) -> Result<Receiver<RisResponse>, Box<dyn error::Error>> {
	self.stream_custom(None, None, None, None).await
    }
}
