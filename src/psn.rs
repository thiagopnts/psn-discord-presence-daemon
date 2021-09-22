use serde::{Deserialize, Serialize};

const TOKEN_ENDPOINT: &str = "https://ca.account.sony.com/api/authz/v3/oauth/token";
const CLIENT_AUTH: &str =
    "YWM4ZDE2MWEtZDk2Ni00NzI4LWIwZWEtZmZlYzIyZjY5ZWRjOkRFaXhFcVhYQ2RYZHdqMHY=";
const LOGIN_URL: &str = "https://ca.account.sony.com/api/authz/v3/oauth/authorize?response_type=code&app_context=inapp_ios&device_profile=mobile&extraQueryParams=%7B%0A%20%20%20%20PlatformPrivacyWs1%20%3D%20minimal%3B%0A%7D&token_format=jwt&access_type=offline&scope=psn%3Amobile.v1%20psn%3Aclientapp&service_entity=urn%3Aservice-entity%3Apsn&ui=pr&smcid=psapp%253Asettings-entrance&darkmode=true&redirect_uri=com.playstation.PlayStationApp%3A%2F%2Fredirect&support_scheme=sneiprls&client_id=ac8d161a-d966-4728-b0ea-ffec22f69edc&duid=0000000d0004008088347AA0C79542D3B656EBB51CE3EBE1&device_base_font_size=10&elements_visibility=no_aclink&service_logo=ps";
const PRESENCE_ENDPOINT: &str =
    "https://m.np.playstation.net/api/userProfile/v1/internal/users/me/basicPresences?type=primary";
const ACCOUNT_ENDPOINT: &str = "https://dms.api.playstation.com/api/v1/devices/accounts/me";

#[derive(Serialize, Deserialize, Debug)]
pub struct GameTitleInfo {
    #[serde(alias = "npTitleId")]
    np_title_id: String,
    #[serde(alias = "titleName")]
    pub title_name: String,
    format: String,
    #[serde(alias = "launchPlatform")]
    launch_platform: String,
    #[serde(alias = "conceptIconUrl")]
    pub concept_icon_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlatformInfo {
    #[serde(alias = "onlineStatus")]
    online_status: String,
    #[serde(alias = "lastOnlineDate")]
    last_online_date: String,
    platform: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct BasicPresence {
    availability: String,
    #[serde(alias = "lastAvailableDate", default)]
    last_available_date: String,
    #[serde(alias = "primaryPlatformInfo")]
    primary_platform_info: PlatformInfo,
    #[serde(alias = "gameTitleInfoList", default)]
    game_title_info: Vec<GameTitleInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Presence {
    #[serde(alias = "basicPresence")]
    basic_presence: BasicPresence,
}

impl Presence {
    pub fn status(&self) -> String {
        self.basic_presence
            .primary_platform_info
            .online_status
            .clone()
    }

    pub fn platform(&self) -> String {
        self.basic_presence.primary_platform_info.platform.clone()
    }

    pub fn is_playing(&self) -> bool {
        self.basic_presence.game_title_info.len() > 0
    }

    pub fn current_game(&self) -> Option<String> {
        if !self.is_playing() {
            return None;
        }
        let game = self.basic_presence.game_title_info.get(0).unwrap();
        Some(game.title_name.clone())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DeviceType {
    PS3,
    PS4,
    PS5,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PSNDevices {
    #[serde(alias = "deviceId")]
    device_id: String,
    #[serde(alias = "deviceType")]
    device_type: DeviceType,
    #[serde(alias = "activationType")]
    activation_type: String,
    #[serde(alias = "activationDate")]
    activation_date: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    #[serde(alias = "accountId")]
    pub account_id: String,
    #[serde(alias = "accountDevices")]
    account_devices: Vec<PSNDevices>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Avatar {
    size: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonalDetail {
    #[serde(alias = "firstName")]
    first_name: String,
    #[serde(alias = "lastName")]
    last_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    #[serde(alias = "onlineId")]
    pub online_id: String,
    #[serde(alias = "personalDetail")]
    personal_detail: PersonalDetail,
    #[serde(alias = "aboutMe")]
    about_me: String,
    avatars: Vec<Avatar>,
    languages: Vec<String>,
    #[serde(alias = "isPlus")]
    is_plus: bool,
    #[serde(alias = "isOfficiallyVerified")]
    is_officially_verified: bool,
    #[serde(alias = "isMe")]
    is_me: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    pub access_token: String,
    expires_in: u64,
    refresh_token: String,
    refresh_token_expires_in: u64,
    id_token: String,
    token_type: String,
    scope: String,
}

pub struct Client {
    http_client: reqwest::Client,
}

impl Client {
    pub fn default() -> Client {
        Client {
            http_client: reqwest::Client::new(),
        }
    }
    pub async fn get_access_token(
        &self,
        code: String,
    ) -> Result<Token, Box<dyn std::error::Error>> {
        let params = [
            ("smcid", "psapp%3Asettings-entrance"),
            ("access_type", "offline"),
            ("code", &code),
            ("service_logo", "ps"),
            ("ui", "pr"),
            ("elements_visibility", "no_aclink"),
            ("redirect_uri", "com.playstation.PlayStationApp://redirect"),
            ("support_scheme", "sneiprls"),
            ("grant_type", "authorization_code"),
            ("darkmode", "true"),
            ("token_format", "jwt"),
            ("device_profile", "mobile"),
            ("app_context", "inapp_ios"),
            ("extraQueryParams", "{ PlatformPrivacyWs1 = minimal; }"),
        ];
        let res = self
            .http_client
            .post(TOKEN_ENDPOINT)
            .header("Authorization", format!("Basic {}", CLIENT_AUTH))
            .header("Content-type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;
        let token = res.json::<Token>().await?;
        Ok(token)
    }

    pub async fn get_profile(
        &self,
        token: Token,
        account_id: String,
    ) -> Result<Profile, Box<dyn std::error::Error>> {
        self.make_get_call(
            &format!(
                "https://m.np.playstation.net/api/userProfile/v1/internal/users/{}/profiles",
                account_id
            ),
            token,
        )
        .await
    }

    pub async fn get_account(&self, token: Token) -> Result<Account, Box<dyn std::error::Error>> {
        self.make_get_call(ACCOUNT_ENDPOINT, token).await
    }

    pub async fn get_presence(&self, token: Token) -> Result<Presence, Box<dyn std::error::Error>> {
        self.make_get_call(PRESENCE_ENDPOINT, token).await
    }

    async fn make_get_call<T>(
        &self,
        endpoint: &str,
        token: Token,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        let res = self
            .http_client
            .get(endpoint)
            .header("Authorization", format!("Bearer {}", token.access_token))
            .send()
            .await?;
        let result = res.json::<T>().await?;
        Ok(result)
    }
}
