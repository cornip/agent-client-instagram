use serde::Deserialize;

#[derive(Debug)]
pub struct ImageUpload {
    pub data: Vec<u8>,
    pub image_type: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct PostConfig {
    pub upload_id: String,
    pub caption: Option<String>,
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Deserialize)]
pub struct CommentUser {
    pub pk: i64,
    pub username: String,
    pub full_name: String,
    pub is_private: bool,
    pub is_verified: bool,
    pub profile_pic_url: String,
    pub pk_id: String,
    pub id: String,
    pub has_onboarded_to_text_post_app: bool,
    pub strong_id__: String,
    pub fbid_v2: i64,
    pub profile_pic_id: String,
    pub is_mentionable: bool,
    pub latest_reel_media: i64,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub pk: String,
    pub user_id: i64,
    pub text: String,
    pub created_at: i64,
    pub comment_like_count: Option<i64>,
    pub user: CommentUser,
    pub child_comment_count: Option<i64>,
    pub r#type: Option<i32>,
    pub did_report_as_spam: Option<bool>,
    pub created_at_utc: Option<i64>,
    pub created_at_for_fb_app: Option<i64>,
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub bit_flags: Option<i32>,
    pub share_enabled: Option<bool>,
    pub is_ranked_comment: Option<bool>,
    pub media_id: Option<i64>,
    pub comment_index: Option<i32>,
    pub strong_id__: Option<String>,
    pub is_covered: Option<bool>,
    pub inline_composer_display_condition: Option<String>,
    pub has_liked_comment: Option<bool>,
    pub private_reply_status: Option<i32>,
    pub preview_child_comments: Option<Vec<Comment>>,
    pub other_preview_users: Option<Vec<CommentUser>>,
}

#[derive(Debug, Deserialize)]
pub struct CommentsResponse {
    pub caption: Option<Comment>,
    pub comment_count: i64,
    pub comments: Vec<Comment>,
    pub has_more_comments: bool,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CommentCreateResponse {
    pub comment: Comment,
    pub comment_creation_key: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct TimelineResponse {
    pub num_results: i32,
    pub more_available: bool,
    pub auto_load_more_enabled: bool,
    pub is_direct_v2_enabled: bool,
    pub next_max_id: Option<String>,
    pub view_state_version: String,
    pub client_feed_changelist_applied: bool,
    pub request_id: String,
    pub pull_to_refresh_window_ms: i64,
    pub preload_distance: i32,
    pub status: String,
    pub feed_items: Vec<FeedItem>,
}

#[derive(Debug, Deserialize)]
pub struct FeedItem {
    pub media_or_ad: MediaOrAd,
}

#[derive(Debug, Deserialize)]
pub struct MediaOrAd {
    pub taken_at: i64,
    pub pk: i64,
    pub id: String,
    pub media_type: i32,
    pub code: String,
    pub caption: Option<Caption>,
    pub image_versions2: ImageVersions2,
    pub original_width: i32, 
    pub original_height: i32,
    pub user: User,
    pub like_count: i32,
    pub has_liked: bool,
    pub comment_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct Caption {
    pub text: String,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct ImageVersions2 {
    pub candidates: Vec<ImageCandidate>,
}

#[derive(Debug, Deserialize)]
pub struct ImageCandidate {
    pub width: i32,
    pub height: i32,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub pk: i64,
    pub username: String,
    pub full_name: String,
    pub is_private: bool,
    pub profile_pic_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ClipsMetadata {
    pub music_info: Option<MusicInfo>,
    pub original_sound_info: Option<OriginalSoundInfo>,
    pub audio_type: Option<String>,
    pub music_canonical_id: Option<String>,
    pub featured_label: Option<String>,
    pub is_shared_to_fb: Option<bool>,
}
#[derive(Debug, Deserialize)]
pub struct MusicInfo {
    pub music_asset_info: Option<MusicAssetInfo>,
    pub music_consumption_info: Option<MusicConsumptionInfo>,
}
#[derive(Debug, Deserialize)]
pub struct MusicAssetInfo {
    pub audio_cluster_id: String,
    pub id: String,
    pub title: String,
    pub sanitized_title: Option<String>,
    pub subtitle: String,
    pub display_artist: String,
    pub artist_id: Option<String>,
    pub cover_artwork_uri: String,
    pub cover_artwork_thumbnail_uri: String,
    pub progressive_download_url: String,
    pub duration_in_ms: u64,
    pub dark_message: Option<String>,
    pub allows_saving: bool,
    pub ig_username: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct MusicConsumptionInfo {
    pub is_eligible_for_preview: bool,
    pub preview_start_time_in_ms: Option<u64>,
    pub preview_duration_in_ms: Option<u64>,
    pub should_mute_audio: bool,
    pub should_mute_audio_reason: String,
    pub should_block_audio: bool,
    pub should_block_audio_reason: String,
}
#[derive(Debug, Deserialize)]
pub struct OriginalSoundInfo {
    pub audio_asset_id: String,
    pub music_canonical_id: Option<String>,
    pub progressive_download_url: String,
    pub duration_in_ms: u64,
    pub dash_manifest: String,
    pub ig_artist: User,
    pub should_mute_audio: bool,
    pub hide_remixing: bool,
    pub original_media_id: String,
    pub time_created: u64,
    pub original_audio_title: String,
    pub consumption_info: MusicConsumptionInfo,
    pub can_remix_be_shared_to_fb: bool,
    pub formatted_clips_media_count: Option<String>,
    pub allow_creator_to_rename: bool,
    pub is_explicit: bool,
    pub has_lyrics: bool,
}
