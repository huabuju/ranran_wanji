pub fn build_http_client(source_name: &str) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| format!("创建 {} 网络客户端失败: {}", source_name, error))
}

pub async fn fetch_text(
    client: &reqwest::Client,
    url: &str,
    source_name: &str,
) -> Result<String, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("请求 {} 页面失败: {} ({})", source_name, url, error))?;

    if !response.status().is_success() {
        return Err(format!(
            "请求 {} 页面失败: {} (HTTP {})",
            source_name,
            url,
            response.status()
        ));
    }

    response
        .text()
        .await
        .map_err(|error| format!("读取 {} 页面内容失败: {} ({})", source_name, url, error))
}
