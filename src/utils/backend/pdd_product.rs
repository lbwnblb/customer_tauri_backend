use serde::Deserialize;
use serde_json::json;

use crate::utils::pinduoduo::pinduoduo_resp::GoodsDetailResult;
use super::BackendClient;

#[derive(Deserialize)]
struct GoodsIdData {
    goods_id: i64,
}

#[derive(Deserialize)]
struct GoodsIdResp {
    data: GoodsIdData,
}

#[derive(Deserialize)]
struct CodeResp {
    code: u16,
}

pub async fn pdd_product_exists(
    goods_id: i64,
    token: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let client = BackendClient::new().with_token(token);
    let resp: CodeResp = client
        .get_typed(&format!("/api/v1/pdd/product/{goods_id}"))
        .await?;
    Ok(resp.code == 200)
}

pub async fn sync_pdd_product(
    result: &GoodsDetailResult,
    token: &str,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let goods_id = result.goods_id.ok_or("缺少 goods_id")?;

    let body = json!({
        "result": {
            "goods_id":               goods_id,
            "goods_commit_id":        result.goods_commit_id,
            "goods_sn":               result.goods_sn,
            "out_goods_sn":           result.out_goods_sn,
            "mall_id":                result.mall_id,
            "mall_name":              result.mall_name,
            "goods_name":             result.goods_name,
            "goods_desc":             result.goods_desc,
            "share_desc":             result.share_desc,
            "market_price":           result.market_price,
            "min_group_price":        result.min_group_price,
            "max_group_price":        result.max_group_price,
            "thumb_url":              result.thumb_url,
            "hd_thumb_url":           result.hd_thumb_url,
            "cat_id_1":               result.cat_id_1,
            "cat_id_2":               result.cat_id_2,
            "cat_id_3":               result.cat_id_3,
            "cat_id_4":               result.cat_id_4,
            "cats":                   result.cats,
            "goods_type":             result.goods_type,
            "goods_type_name":        result.goods_type_name,
            "status":                 result.status,
            "is_pre_sale":            result.is_pre_sale,
            "is_refundable":          result.is_refundable,
            "shipment_limit_second":  result.shipment_limit_second,
            "goods_pattern":          0,
            "carousel_gallery":       result.carousel_gallery,
            "detail_gallery":         result.detail_gallery,
            "skus":                   result.skus,
            "groups":                 result.groups,
            "goods_options":          result.goods_options,
            "created_at":             result.created_at,
        }
    });

    let client = BackendClient::new().with_token(token);
    let resp: GoodsIdResp = client.post_typed("/api/v1/pdd/product", &body).await?;
    log::info!("[sync_pdd_product] 已同步 goods_id={}", resp.data.goods_id);
    Ok(resp.data.goods_id)
}
