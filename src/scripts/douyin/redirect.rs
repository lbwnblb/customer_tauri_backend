pub fn douyin_redirect(webview_id: &str) -> String {
    r#"
        (function() {
                if (location.pathname.includes('/ffa/mshop/homepage/index')) {
                    window.__TAURI__.core.invoke('shop_name_callback');
                }
        })();
"#.replace("__WEBVIEW_ID__", webview_id)
}
