use crate::activity::Activity;
use crate::discord_ipc::DiscordIpc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub struct Client {
    http_client: reqwest::Client,
    rpc_client: DiscordIpc,
    app_id: String,
    auth_token: String,
    pid: i32,
}

impl<'a> Client {
    pub fn default(app_id: String) -> Self {
        let http_client = reqwest::Client::new();
        let rpc_client = DiscordIpc {
            client_id: app_id.clone(),
            connected: false,
            socket: None,
        };
        Self {
            http_client,
            app_id: app_id.clone(),
            rpc_client,
            auth_token: "".to_string(),
            // fixed value for now, once things stabilize we can use std::process:id()
            pid: 666,
        }
    }

    pub async fn set_activity(
        &mut self,
        activity: Activity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.rpc_client.connect()?;
        self.rpc_client.send(
            json!({
                "cmd": "SET_ACTIVITY",
                "args": {
                    "pid": self.pid,
                    "activity": activity,
                },
                "nonce": "0",
            }),
            1,
        )?;
        self.rpc_client.close()
    }

    pub async fn clear_activity(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.rpc_client.connect()?;
        self.rpc_client.send(
            json!({
                "cmd": "SET_ACTIVITY",
                "args": {
                    "pid": self.pid,
                },
                "nonce": "0",
            }),
            1,
        )?;
        self.rpc_client.close()
    }

    pub async fn assets(&self) -> Result<Vec<Asset>, Box<dyn std::error::Error + Send + Sync>> {
        self.make_get_call(&format!(
            "https://discordapp.com/api/oauth2/applications/{}/assets",
            self.app_id
        ))
        .await
    }

    pub async fn create_asset(
        &self,
        name: String,
        image: &str,
    ) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
        let res = self
            .http_client
            .post(&format!(
                "https://discordapp.com/api/oauth2/applications/{}/assets",
                self.app_id
            ))
            .header("Authorization", &self.auth_token)
            .json(&CreateAsset::new(name.clone(), image.to_string()))
            .send()
            .await?;
        let j = serde_json::to_string(&CreateAsset::new(name, image.to_string()))?;
        let asset = res.json().await?;
        println!("{:?}", asset);
        Ok(asset)
    }

    async fn make_get_call<T>(
        &self,
        endpoint: &str,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        let res = self.http_client.get(endpoint).send().await?;
        let result = res.json::<T>().await?;
        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateAsset {
    image: String,
    name: String,
    #[serde(rename = "type")]
    kind: i32,
}

impl CreateAsset {
    fn new(name: String, image: String) -> Self {
        CreateAsset {
            image,
            name,
            kind: 1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Asset {
    id: String,
    #[serde(alias = "type")]
    kind: i32,
    pub name: String,
}

impl Asset {
    pub fn new(id: String, kind: i32, name: String) -> Self {
        Self { id, kind, name }
    }
}
