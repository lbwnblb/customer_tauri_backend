use reqwest::Client;
use serde::Deserialize;
use tauri::{Webview, Window};

use crate::utils::douyin::feige_resp::{get_diagnose_product_detail, get_promotion_pack_h5};
use super::url;

#[derive(Deserialize)]
struct CodeResp {
    code: u8,
}

pub async fn product_exists(product_id: &str, token: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let resp = Client::new()
        .get(url(&format!("/api/v1/product/{product_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("后端返回错误 status={status}: {text}").into());
    }

    let body: CodeResp = resp.json().await?;
    Ok(body.code == 0)
}

pub async fn product_h5_exists(product_id: &str, token: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let resp = Client::new()
        .get(url(&format!("/api/v1/product/h5/{product_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("后端返回错误 status={status}: {text}").into());
    }

    let body: CodeResp = resp.json().await?;
    Ok(body.code == 0)
}

pub async fn sync_diagnose_product(
    webview_id: &str,
    webview: &Webview,
    product_id: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = get_diagnose_product_detail(webview_id, webview, product_id).await?;
    let data = resp.data.ok_or("diagnose response 缺少 data")?;
    let pid = data.product_id.ok_or("缺少 product_id")?;

    let body = serde_json::json!({
        "product_id":             pid,
        "product_name":           data.product_name.unwrap_or_default(),
        "product_status":         data.product_status.unwrap_or(0),
        "product_check_status":   data.product_check_status.unwrap_or(0),
        "img":                    data.img.unwrap_or_default(),
        "problem_num_to_improve": data.problem_num_to_improve.unwrap_or(0),
        "field_problem":          serde_json::to_value(&data.field_problem).unwrap_or_default(),
        "list_operation":         serde_json::to_value(&data.list_operation).unwrap_or_default(),
        "is_checking":            data.is_checking.unwrap_or(false),
        "diagnose_recommend":     serde_json::to_value(&data.diagnose_recommend).unwrap_or_default(),
        "quality_score":          data.quality_score.unwrap_or(0),
        "quality_level":          data.quality_level.unwrap_or_default(),
        "suggest_field_problem":  serde_json::to_value(&data.suggest_field_problem).unwrap_or_default(),
        "conversion_revenue_msg": data.conversion_revenue_msg.unwrap_or_default(),
    });

    let response = Client::new()
        .post(url("/api/v1/product"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("后端返回错误 status={status}: {text}").into());
    }

    log::info!("[sync_diagnose_product] 已同步 product_id={pid}");
    Ok(())
}

pub async fn sync_promotion_h5(
    window: &Window,
    webview_id: &str,
    product_id: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = get_promotion_pack_h5(window, webview_id, product_id).await?;
    let body = serde_json::to_value(&resp)?;

    let response = Client::new()
        .post(url("/api/v1/product/h5"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("后端返回错误 status={status}: {text}").into());
    }

    log::info!("[sync_promotion_h5] 已同步 product_id={product_id}");
    Ok(())
}
