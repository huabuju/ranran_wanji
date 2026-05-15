use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubApkListRequest {
    pub repo: String,
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubApkListResponse {
    pub repo: String,
    pub release_count: usize,
    pub asset_count: usize,
    pub assets: Vec<GitHubApkAssetItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubApkAssetItem {
    pub release_tag: String,
    pub release_name: String,
    pub release_page_url: String,
    pub asset_name: String,
    pub download_url: String,
    pub size: u64,
    pub download_count: u64,
    pub updated_at: String,
    pub published_at: String,
    pub is_prerelease: bool,
    pub is_draft: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseDto {
    tag_name: Option<String>,
    name: Option<String>,
    html_url: Option<String>,
    draft: bool,
    prerelease: bool,
    published_at: Option<String>,
    assets: Vec<GitHubAssetDto>,
}

#[derive(Debug, Deserialize)]
struct GitHubAssetDto {
    name: Option<String>,
    browser_download_url: Option<String>,
    size: Option<u64>,
    download_count: Option<u64>,
    updated_at: Option<String>,
}

fn normalize_repo_input(input: &str) -> Result<String, String> {
    let trimmed = input.trim().trim_matches('/');
    if trimmed.is_empty() {
        return Err("请输入 GitHub 仓库，例如 topjohnwu/Magisk".to_string());
    }

    if let Some(path) = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
    {
        return parse_repo_path(path);
    }

    if let Some(path) = trimmed
        .strip_prefix("https://www.github.com/")
        .or_else(|| trimmed.strip_prefix("http://www.github.com/"))
    {
        return parse_repo_path(path);
    }

    parse_repo_path(trimmed)
}

fn parse_repo_path(path: &str) -> Result<String, String> {
    let segments: Vec<&str> = path
        .split('/')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect();

    if segments.len() < 2 {
        return Err("仓库格式不正确，请使用 owner/repo 或 GitHub 仓库地址".to_string());
    }

    let owner = segments[0];
    let repo = segments[1].strip_suffix(".git").unwrap_or(segments[1]);
    if owner.is_empty() || repo.is_empty() {
        return Err("仓库格式不正确，请使用 owner/repo 或 GitHub 仓库地址".to_string());
    }

    Ok(format!("{owner}/{repo}"))
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("RanranToolkit/0.1 GitHub APK Browser")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| format!("创建 GitHub 请求客户端失败: {error}"))
}

async fn fetch_release_page(
    client: &reqwest::Client,
    repo: &str,
    token: Option<&str>,
    page: usize,
) -> Result<Vec<GitHubReleaseDto>, String> {
    let url = format!("https://api.github.com/repos/{repo}/releases?per_page=100&page={page}");
    let mut request = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json");
    if let Some(token) = token.filter(|value| !value.trim().is_empty()) {
        request = request.bearer_auth(token.trim());
    }
    let response = request
        .send()
        .await
        .map_err(|error| format!("请求 GitHub Releases 失败: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let rate_remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let body = response.text().await.unwrap_or_default();
        if status.as_u16() == 404 {
            return Err(format!("未找到仓库 {repo}，请确认 owner/repo 是否正确"));
        }
        if status.as_u16() == 401 {
            return Err("GitHub Token 无效、已过期，或没有访问该仓库 Releases 的权限".to_string());
        }
        if status.as_u16() == 403 && rate_remaining == "0" {
            return Err("GitHub API 访问频率已达匿名上限，请挂梯或稍后再试".to_string());
        }
        let detail = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|value| value.get("message").and_then(|message| message.as_str()).map(str::to_string))
            .unwrap_or(body);
        return Err(format!("GitHub API 返回异常状态 {}: {}", status.as_u16(), detail.trim()));
    }

    let body = response
        .text()
        .await
        .map_err(|error| format!("读取 GitHub Releases 响应失败: {error}"))?;

    serde_json::from_str::<Vec<GitHubReleaseDto>>(&body)
        .map_err(|error| format!("解析 GitHub Releases 响应失败: {error}"))
}

#[tauri::command]
pub async fn fetch_github_apk_assets(
    request: GitHubApkListRequest,
) -> Result<GitHubApkListResponse, String> {
    let repo = normalize_repo_input(&request.repo)?;
    let token = request
        .token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let client = build_client()?;
    let mut release_count = 0usize;
    let mut assets = Vec::new();

    for page in 1..=20 {
        let releases = fetch_release_page(&client, &repo, token, page).await?;
        if releases.is_empty() {
            break;
        }

        release_count += releases.len();
        for release in releases {
            let release_tag = release.tag_name.unwrap_or_default();
            let release_name = release
                .name
                .filter(|item| !item.trim().is_empty())
                .unwrap_or_else(|| release_tag.clone());
            let release_page_url = release.html_url.unwrap_or_default();
            let published_at = release.published_at.unwrap_or_default();

            for asset in release.assets {
                let asset_name = asset.name.unwrap_or_default();
                if !asset_name.to_ascii_lowercase().ends_with(".apk") {
                    continue;
                }

                let download_url = asset.browser_download_url.unwrap_or_default();
                if download_url.trim().is_empty() {
                    continue;
                }

                assets.push(GitHubApkAssetItem {
                    release_tag: release_tag.clone(),
                    release_name: release_name.clone(),
                    release_page_url: release_page_url.clone(),
                    asset_name,
                    download_url,
                    size: asset.size.unwrap_or(0),
                    download_count: asset.download_count.unwrap_or(0),
                    updated_at: asset.updated_at.unwrap_or_default(),
                    published_at: published_at.clone(),
                    is_prerelease: release.prerelease,
                    is_draft: release.draft,
                });
            }
        }
    }

    Ok(GitHubApkListResponse {
        repo,
        release_count,
        asset_count: assets.len(),
        assets,
    })
}
