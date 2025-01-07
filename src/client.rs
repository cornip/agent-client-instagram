use crate::utils::*;
use indexmap::IndexMap;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde_json::json;
use serde_json::Value;
use crate::types::*;
pub struct AgentClientInstagram {
    client: Client,
    username: String,
    password: String,
    two_factor_key: Option<String>,
    auth_token: Option<String>,
    device_id: String,
    family_device_id: String,
    machine_id: String,
    uid: String,
}

impl AgentClientInstagram {
    pub fn new(username: String, password: String, two_factor_key: Option<String>) -> Self {
        let device_id = generate_device_id();
        let headers = Self::default_headers(&device_id);
        let family_device_id = generate_uuid();
        let machine_id = generate_random_key();
        let client = Client::builder().default_headers(headers).build().unwrap();
        let uid = generate_numeric_id();
        Self {
            client,
            username,
            password,
            two_factor_key,
            auth_token: None,
            device_id,
            family_device_id,
            machine_id,
            uid,
        }
    }

    pub fn new_with_token(auth_token: String) -> Self {
        let device_id = generate_device_id();
        let family_device_id = generate_uuid();
        let machine_id = generate_random_key();
        let uid = generate_numeric_id();
        let mut auth_headers = Self::default_headers(&device_id);
        auth_headers.insert(
            "authorization",
            HeaderValue::from_str(&auth_token).unwrap(),
        );
        let client = Client::builder().default_headers(auth_headers).build().unwrap();

        Self {
            client,
            username: String::new(),
            password: String::new(),
            two_factor_key: None,
            auth_token: Some(auth_token),
            device_id,
            family_device_id,
            machine_id,
            uid,
        }
    }

    pub async fn login(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let encrypted_password = encrypt_password(&self.password).await;

        let params = json!({
            "client_input_params": {
                "sim_phones": [],
                "secure_family_device_id": "",
                "has_granted_read_contacts_permissions": 0,
                "auth_secure_device_id": "",
                "has_whatsapp_installed": 0,
                "password": encrypted_password,
                "sso_token_map_json_string": "",
                "event_flow": "login_manual",
                "password_contains_non_ascii": "false",
                "client_known_key_hash": "",
                "encrypted_msisdn": "",
                "has_granted_read_phone_permissions": 0,
                "app_manager_id": "",
                "should_show_nested_nta_from_aymh": 0,
                "device_id": self.device_id,
                "login_attempt_count": 1,
                "machine_id": self.machine_id,
                "accounts_list": [],
                "family_device_id": self.family_device_id,
                "fb_ig_device_id": [],
                "device_emails": [],
                "try_num": 4,
                "lois_settings": {
                    "lois_token": "",
                    "lara_override": ""
                },
                "event_step": "home_page",
                "headers_infra_flow_id": "",
                "openid_tokens": {},
                "contact_point": self.username
            },
            "server_params": {
                "should_trigger_override_login_2fa_action": 0,
                "is_from_logged_out": 0,
                "should_trigger_override_login_success_action": 0,
                "login_credential_type": "none",
                "server_login_source": "login",
                "waterfall_id": generate_uuid(),
                "login_source": "Login",
                "is_platform_login": 0,
                "INTERNAL__latency_qpl_marker_id": 36707139,
                "offline_experiment_group": "caa_iteration_v3_perf_ig_4",
                "is_from_landing_page": 0,
                "password_text_input_id": "9kb4g6:104",
                "is_from_empty_password": 0,
                "is_from_msplit_fallback": 0,
                "ar_event_source": "login_home_page",
                "qe_device_id": generate_uuid(),
                "username_text_input_id": "9kb4g6:103",
                "layered_homepage_experiment_group": null,
                "device_id": self.device_id,
                "INTERNAL__latency_qpl_instance_id": 5.7830688600186E13,
                "reg_flow_source": "login_home_native_integration_point",
                "is_caa_perf_enabled": 1,
                "credential_type": "password",
                "is_from_password_entry_page": 0,
                "caller": "gslr",
                "family_device_id": self.family_device_id,
                "is_from_assistive_id": 0,
                "access_flow_version": "F2_FLOW",
                "is_from_logged_in_switcher": 0
            }
        });

        let bk_client_context = serde_json::json!({
            "bloks_version": BLOKS_VERSION,
            "styles_id": "instagram"
        });
        let mut form_params = IndexMap::new();
        form_params.insert("params", params.to_string());
        form_params.insert("bk_client_context", bk_client_context.to_string());
        form_params.insert("bloks_versioning_id", BLOKS_VERSION.to_string());

        let response = self.client
            .post("https://i.instagram.com/api/v1/bloks/async_action/com.bloks.www.bloks.caa.login.async.send_login_request/")
            .headers(self.login_headers())
            .form(&form_params)
            .send()
            .await?;

        let mut json: Value = response.json().await?;
        if json.to_string().contains("two_step_verification_context") {
            if let Some(two_factor_key) = &self.two_factor_key {
                let code = generate_totp_code(two_factor_key)?;
                let verification_context = extract_two_factor_context(json.to_string());
                json = self.handle_two_factor(&verification_context, &code).await?;
            } else {
                return Err("Two factor authentication required but no key provided".into());
            }
        }
        self.auth_token = extract_bearer_token(json.to_string());
        
        if self.auth_token.is_some() {
            self.client = Client::builder()
                .default_headers(self.authorized_headers())
                .build()
                .unwrap();
        }
        Ok(())
    }

    async fn handle_two_factor(
        &mut self,
        verification_context: &str,
        code: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let params = json!({
            "client_input_params": {
                "auth_secure_device_id": "",
                "machine_id": self.machine_id,
                "code": code,
                "should_trust_device": 0,
                "family_device_id": self.family_device_id,
                "device_id": self.device_id
            },
            "server_params": {
                "INTERNAL__latency_qpl_marker_id": 36707139,
                "device_id": self.device_id,
                "challenge": "totp",
                "machine_id": null,
                "INTERNAL__latency_qpl_instance_id": 91439808700070i64,
                "two_step_verification_context": verification_context,
                "flow_source": "two_factor_login"
            }
        });

        let mut form_params = IndexMap::new();
        form_params.insert("params", params.to_string());
        form_params.insert(
            "bk_client_context",
            json!({
                "bloks_version": BLOKS_VERSION,
                "styles_id": "instagram"
            })
            .to_string(),
        );
        form_params.insert("bloks_versioning_id", BLOKS_VERSION.to_string());

        let response = self.client
            .post("https://i.instagram.com/api/v1/bloks/async_action/com.bloks.www.two_step_verification.verify_code.async/")
            .headers(self.login_headers())
            .form(&form_params)
            .send()
            .await?;

        let json: Value = response.json().await?;
        Ok(json)
    }

    fn default_headers(device_id: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-ig-app-locale", HeaderValue::from_static("en_US"));
        headers.insert("x-ig-device-locale", HeaderValue::from_static("en_US"));
        headers.insert("x-ig-mapped-locale", HeaderValue::from_static("en_US"));
        headers.insert(
            "x-ig-bandwidth-speed-kbps",
            HeaderValue::from_static("-1.000"),
        );
        headers.insert("x-ig-bandwidth-totalbytes-b", HeaderValue::from_static("0"));
        headers.insert("x-ig-bandwidth-totaltime-ms", HeaderValue::from_static("0"));
        headers.insert(
            "x-bloks-version-id",
            HeaderValue::from_static(BLOKS_VERSION),
        );
        headers.insert("x-ig-www-claim", HeaderValue::from_static("0"));
        headers.insert("x-ig-device-id", HeaderValue::from_str(device_id).unwrap());
        headers.insert("x-ig-android-id", HeaderValue::from_str(device_id).unwrap());
        headers.insert("x-ig-connection-type", HeaderValue::from_static("WIFI"));
        headers.insert("x-ig-capabilities", HeaderValue::from_static("3brTv10="));
        headers.insert("x-ig-app-id", HeaderValue::from_static("567067343352427"));
        headers.insert("user-agent", HeaderValue::from_static(USER_AGENT));
        headers.insert("accept-language", HeaderValue::from_static("en-US"));
        headers.insert("x-fb-http-engine", HeaderValue::from_static("Liger"));
        headers.insert("x-fb-client-ip", HeaderValue::from_static("True"));
        headers.insert("x-fb-server-cluster", HeaderValue::from_static("True"));
        headers
    }

    fn login_headers(&self) -> HeaderMap {
        let mut headers = Self::default_headers(&self.device_id);
        headers.insert(
            "content-type",
            HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
        );
        headers
    }

    fn authorized_headers(&self) -> HeaderMap {
        let mut headers = self.login_headers();
        if let Some(token) = &self.auth_token {
            headers.insert(
                "authorization",
                HeaderValue::from_str(token).unwrap(),
            );
        }
        headers
    }

    pub fn get_auth_token(&self) -> Option<String> {
        self.auth_token.clone()
    }
    
    pub async fn like_post(&self, media_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://i.instagram.com/api/v1/media/{}/like/",
            media_id
        );

        let params = json!({
            "num_visible_media_notes": "0",
            "is_2m_enabled": "false",
            "inventory_source": "explore_story",
            "delivery_class": "organic",
            "tap_source": "button",
            "media_id": media_id,
            "fully_visible_media_note_ids": "",
            "radio_type": "wifi-none",
            "_uid": "72002970302",
            "_uuid": self.device_id,
            "nav_chain": "MainFeedFragment:feed_timeline:1:cold_start:1736193024.120::",
            "is_from_swipe": "false",
            "logging_info_token": generate_uuid(),
            "recs_ix": "0",
            "is_carousel_bumped_post": "false",
            "container_module": "feed_timeline",
            "feed_position": "1"
        });

        let signed_body = format!("SIGNATURE.{}", params.to_string());
        let mut form_params = IndexMap::new();
        form_params.insert("signed_body", signed_body);
        form_params.insert("d", "0".to_string());

        let response = self.client
            .post(url)
            .headers(self.authorized_headers())
            .form(&form_params)
            .send()
            .await?;

        let json: Value = response.json().await?;
        
        if json["status"] != "ok" {
            return Err(format!("Failed to like post: {}", json).into());
        }

        Ok(())
    }

    pub async fn comment_post(
        &self,
        media_id: &str,
        comment_text: &str,
        reply_to_comment_id: Option<&str>
    ) -> Result<CommentCreateResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "https://i.instagram.com/api/v1/media/{}/comment/",
            media_id
        );

        let comment_creation_key = generate_uuid();
        let mut params = json!({
            "num_visible_media_notes": "0",
            "include_media_code": "true",
            "user_breadcrumb": gen_user_breadcrumb(comment_text.len() as u32),
            "inventory_source": "explore_story",
            "starting_clips_media_id": "null",
            "comment_creation_key": comment_creation_key,
            "delivery_class": "organic",
            "idempotence_token": comment_creation_key,
            "client_position": "1",
            "fully_visible_media_note_ids": "[]",
            "carousel_child_mentions": "[]",
            "include_e2ee_mentioned_user_list": "true",
            "include_carousel_child_mentions": "false",
            "is_from_carousel_child_thread": "false",
            "carousel_index": "-1",
            "radio_type": "wifi-none",
            "_uid": "72002970302",
            "is_text_app_xpost_attempt": "false",
            "_uuid": self.device_id,
            "nav_chain": "MainFeedFragment:feed_timeline:1:cold_start:1736193024.120::,CommentListBottomsheetFragment:comments_v2:4:button:1736270506.774::",
            "logging_info_token": generate_uuid(),
            "comment_text": comment_text,
            "recs_ix": "0",
            "is_carousel_bumped_post": "false",
            "container_module": "comments_v2_feed_timeline",
            "feed_position": "1",
            "ranking_session_id": generate_uuid()
        });

        if let Some(reply_id) = reply_to_comment_id {
            params.as_object_mut().unwrap().insert("replied_to_comment_id".to_string(), json!(reply_id));
        }

        let signed_body = format!("SIGNATURE.{}", params.to_string());
        let mut form_params = IndexMap::new();
        form_params.insert("signed_body", signed_body);

        let response = self.client
            .post(url)
            .headers(self.authorized_headers())
            .form(&form_params)
            .send()
            .await?;

        let comment_response: CommentCreateResponse = response.json().await?;
        
        if comment_response.status != "ok" {
            return Err(format!("Failed to comment on post. Status: {}", comment_response.status).into());
        }

        Ok(comment_response)
    }

    pub async fn like_comment(
        &self,
        comment_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://i.instagram.com/api/v1/media/{}/comment_like/",
            comment_id
        );

        let params = json!({
            "inventory_source": "explore_story",
            "delivery_class": "organic",
            "ranking_info_token": generate_uuid(),
            "_uid": "72002970302",
            "_uuid": self.device_id,
            "nav_chain": "MainFeedFragment:feed_timeline:1:cold_start:1736193024.120::,CommentListBottomsheetFragment:comments_v2:4:button:1736270506.774::",
            "is_reply_highlight": "false",
            "is_carousel_bumped_post": "false",
            "container_module": "comments_v2_feed_timeline",
            "is_feed_preview_comment": "false",
            "feed_position": "1"
        });

        let signed_body = format!("SIGNATURE.{}", params.to_string());
        let mut form_params = IndexMap::new();
        form_params.insert("signed_body", signed_body);

        let response = self.client
            .post(url)
            .headers(self.authorized_headers())
            .form(&form_params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to like comment: {}", response.status()).into());
        }

        Ok(())
    }

    pub async fn get_timeline(&self) -> Result<TimelineResponse, Box<dyn std::error::Error>> {
        let session_id = generate_uuid();
        let mut form_params = IndexMap::new();
        form_params.insert("has_camera_permission", "1");
        form_params.insert("device_id", &self.device_id);
        form_params.insert("_uuid", &self.device_id);
        form_params.insert("phone_id", &self.family_device_id);
        form_params.insert("is_charging", "1");
        form_params.insert("is_dark_mode", "0");
        form_params.insert("will_sound_on", "0");
        form_params.insert("battery_level", "100");
        form_params.insert("timezone_offset", "28800");
        form_params.insert("is_pull_to_refresh", "1");
        form_params.insert("bloks_versioning_id", BLOKS_VERSION);
        form_params.insert("session_id", session_id.as_str());

        let response = self.client
            .post("https://i.instagram.com/api/v1/feed/timeline/")
            .headers(self.authorized_headers())
            .form(&form_params)
            .send()
            .await?;

        let timeline: TimelineResponse = response.json().await?;
        Ok(timeline)
    }

    pub async fn unlike_post(&self, media_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://i.instagram.com/api/v1/media/{}/unlike/",
            media_id
        );

        let params = json!({
            "num_visible_media_notes": "0",
            "is_2m_enabled": "false",
            "inventory_source": "explore_story",
            "delivery_class": "organic",
            "tap_source": "button",
            "media_id": media_id,
            "fully_visible_media_note_ids": "",
            "radio_type": "wifi-none",
            "_uid": self.uid,
            "_uuid": self.device_id,
            "nav_chain": "MainFeedFragment:feed_timeline:1:cold_start:1736193024.120::",
            "is_from_swipe": "false",
            "logging_info_token": generate_uuid(),
            "is_carousel_bumped_post": "false",
            "container_module": "feed_timeline",
            "feed_position": "0"
        });

        let signed_body = format!("SIGNATURE.{}", params.to_string());
        let mut form_params = IndexMap::new();
        form_params.insert("signed_body", signed_body);
        form_params.insert("d", "0".to_string());

        let response = self.client
            .post(url)
            .headers(self.authorized_headers())
            .form(&form_params)
            .send()
            .await?;

        let json: Value = response.json().await?;
        
        if json["status"] != "ok" {
            return Err(format!("Failed to unlike post: {}", json).into());
        }

        Ok(())
    }

    pub async fn follow_user(&self, username: &str) -> Result<(), Box<dyn std::error::Error>> {
        let user_id = self.get_user_id(username).await?;
        let url = format!(
            "https://i.instagram.com/api/v1/friendships/create/{}/",
            user_id
        );

        let params = json!({
            "include_follow_friction_check": "1",
            "user_id": user_id,
            "radio_type": "wifi-none",
            "_uid": self.uid,
            "device_id": self.device_id,
            "_uuid": self.device_id,
            "container_module": "feed_contextual_profile"
        });

        let signed_body = format!("SIGNATURE.{}", params.to_string());
        let mut form_params = IndexMap::new();
        form_params.insert("signed_body", signed_body);

        let response = self.client
            .post(url)
            .headers(self.authorized_headers())
            .form(&form_params)
            .send()
            .await?;

        let json: Value = response.json().await?;
        
        if json["status"] != "ok" {
            return Err(format!("Failed to follow user: {}", json).into());
        }

        Ok(())
    }

    pub async fn unfollow_user(&self, username: &str) -> Result<(), Box<dyn std::error::Error>> {
        let user_id = self.get_user_id(username).await?;
        let url = format!(
            "https://i.instagram.com/api/v1/friendships/destroy/{}/",
            user_id
        );

        let params = json!({
            "include_follow_friction_check": "1",
            "user_id": user_id,
            "radio_type": "wifi-none",
            "_uid": self.uid,
            "_uuid": self.device_id,
            "container_module": "following_sheet"
        });

        let signed_body = format!("SIGNATURE.{}", params.to_string());
        let mut form_params = IndexMap::new();
        form_params.insert("signed_body", signed_body);

        let response = self.client
            .post(url)
            .headers(self.authorized_headers())
            .form(&form_params)
            .send()
            .await?;

        let json: Value = response.json().await?;
        
        if json["status"] != "ok" {
            return Err(format!("Failed to unfollow user: {}", json).into());
        }

        Ok(())
    }

    pub async fn upload_photo(&self, image_data: &[u8], image_type: &str, width: u32, height: u32) -> Result<String, Box<dyn std::error::Error>> {
        let upload_id = format!("{}", chrono::Utc::now().timestamp_millis());
        let entity_name = format!("{}_{}_{}",upload_id, 0, rand::random::<u32>());
        let waterfall_id = generate_uuid();
        
        let image_compression = json!({
            "lib_name": "libwebp",
            "lib_version": "28",
            "quality": "95",
            "original_width": width,
            "original_height": height,
            "msssim": 0.9967840909957886,
            "ssim": 0.9987573834728883
        }).to_string();

        let retry_context = json!({
            "num_reupload": 0,
            "num_step_manual_retry": 0,
            "num_step_auto_retry": 0
        }).to_string();

        let rupload_params = json!({
            "upload_id": upload_id,
            "session_id": upload_id,
            "media_type": "1",
            "upload_engine_config_enum": "0",
            "image_compression": image_compression,
            "xsharing_user_ids": "[]",
            "retry_context": retry_context
        });

        let mut headers = self.authorized_headers();
        headers.insert("x-entity-length", HeaderValue::from_str(&image_data.len().to_string())?);
        headers.insert("x-entity-name", HeaderValue::from_str(&entity_name)?);
        headers.insert("x_fb_photo_waterfall_id", HeaderValue::from_str(&waterfall_id)?);
        headers.insert("x-instagram-rupload-params", HeaderValue::from_str(&rupload_params.to_string())?);
        headers.insert("x-entity-type", HeaderValue::from_str(image_type)?);
        headers.insert("offset", HeaderValue::from_static("0"));
        headers.insert("content-type", HeaderValue::from_static("application/octet-stream"));

        let url = format!(
            "https://i.instagram.com/rupload_igphoto/{}",
            entity_name
        );

        let response = self.client
            .post(url)
            .headers(headers)
            .body(image_data.to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to upload photo: {}", response.status()).into());
        }

        Ok(upload_id)
    }

    pub async fn configure_post(&self, config: &PostConfig) -> Result<String, Box<dyn std::error::Error>> {
        let params = json!({
            "app_attribution_android_namespace": "",
            "camera_entry_point": "360",
            "camera_session_id": generate_uuid(),
            "original_height": config.height.to_string(),
            "include_e2ee_mentioned_user_list": "1",
            "hide_from_profile_grid": "false",
            "scene_capture_type": "",
            "timezone_offset": "28800",
            "source_type": "4",
            "_uid": self.uid,
            "device_id": self.device_id,
            "_uuid": self.device_id,
            "creation_tool_info": "[]",
            "creation_logger_session_id": generate_uuid(),
            "caption": config.caption.as_deref().unwrap_or(""),
            "audience": "default",
            "upload_id": config.upload_id,
            "publish_id": "1",
            "original_width": config.width.to_string(),
            "edits": {
                "filter_type": 0,
                "filter_strength": 1.0,
                "crop_original_size": [config.width as f32, config.height as f32],
                "crop_center": [0.0, 0.0],
                "crop_zoom": 1.0
            },
            "extra": {
                "source_width": config.width,
                "source_height": config.height
            },
            "device": {
                "manufacturer": "Asus",
                "model": "ASUS_I003DD",
                "android_version": 28,
                "android_release": "9"
            }
        });

        let signed_body = format!("SIGNATURE.{}", params.to_string());
        let mut form_params = IndexMap::new();
        form_params.insert("signed_body", signed_body);

        let response = self.client
            .post("https://i.instagram.com/api/v1/media/configure/")
            .headers(self.authorized_headers())
            .form(&form_params)
            .send()
            .await?;

        let json: Value = response.json().await?;
        
        if json["status"] != "ok" {
            return Err("Failed to configure post".into());
        }

        if let Some(media) = json["media"].as_object() {
            if let Some(id) = media["id"].as_str() {
                return Ok(id.to_string());
            }
        }

        Err("Could not extract media ID from response".into())
    }

    pub async fn create_post(
        &self,
        image: ImageUpload,
        caption: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let upload_id = self.upload_photo(&image.data, &image.image_type, image.width, image.height).await?;

        let config = PostConfig {
            upload_id,
            width: image.width,
            height: image.height,
            caption,
        };

        self.configure_post(&config).await
    }

    pub async fn get_user_id(&self, username: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://i.instagram.com/api/v1/discover/chaining/?module=profile&target_username={}&from_module=self_profile&profile_chaining_check=true",
            username
        );

        let response = self.client
            .get(url)
            .headers(self.authorized_headers())
            .send()
            .await?;

        let json: Value = response.json().await?;
        
        if json["status"] == "ok" {
            if let Some(user) = json["user"].as_object() {
                if let Some(pk) = user["pk_id"].as_str() {
                    return Ok(pk.to_string());
                }
            }
        }

        Err(format!("Could not find user ID for username: {}", username).into())
    }

    pub async fn get_comments(&self, media_id: &str) -> Result<CommentsResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "https://i.instagram.com/api/v1/media/{}/stream_comments/?can_support_threading=true&is_carousel_bumped_post=false",
            media_id
        );

        let mut headers = self.authorized_headers();
        headers.insert(
            "x-ig-nav-chain",
            HeaderValue::from_static("MainFeedFragment:feed_timeline:1:cold_start:::")
        );

        let response = self.client
            .get(url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get comments: {}", response.status()).into());
        }

        let comments: CommentsResponse = response.json().await?;
        Ok(comments)
    }
}
