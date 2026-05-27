pub fn create_promotion_h5_interceptor(webview_label: &str) -> String {
    let label = webview_label.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        r#"
(function() {{
    if (window.__PROMO_H5_INTERCEPTOR__) return;
    window.__PROMO_H5_INTERCEPTOR__ = true;

    const TARGET = '/aweme/v2/shop/promotion/pack/h5';
    const LABEL = "{label}";

    function report(text) {{
        try {{
            if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {{
                window.__TAURI__.core.invoke('on_promotion_pack_h5', {{
                    webviewLabel: LABEL,
                    body: text,
                }}).catch(function(e) {{
                    console.error('[PromoH5] invoke 失败', e);
                }});
            }}
        }} catch(_) {{}}
    }}

    const _fetch = window.fetch;
    if (typeof _fetch === 'function') {{
        window.fetch = async function() {{
            const args = Array.prototype.slice.call(arguments);
            const url = (args[0] instanceof Request) ? args[0].url : String(args[0] || '');
            const resp = await _fetch.apply(this, args);
            if (url.includes(TARGET)) {{
                resp.clone().text().then(report).catch(function() {{}});
            }}
            return resp;
        }};
    }}

    const _xhrOpen = XMLHttpRequest.prototype.open;
    const _xhrSend = XMLHttpRequest.prototype.send;

    XMLHttpRequest.prototype.open = function(method, url) {{
        this._promoUrl = url;
        return _xhrOpen.apply(this, arguments);
    }};

    XMLHttpRequest.prototype.send = function() {{
        if (typeof this._promoUrl === 'string' && this._promoUrl.includes(TARGET)) {{
            const origCb = this.onreadystatechange;
            const self = this;
            this.onreadystatechange = function() {{
                if (self.readyState === 4) {{
                    try {{
                        var text = self.responseType === 'json'
                            ? JSON.stringify(self.response)
                            : self.responseText;
                        report(text);
                    }} catch(_) {{}}
                }}
                if (typeof origCb === 'function') origCb.apply(self, arguments);
            }};
        }}
        return _xhrSend.apply(this, arguments);
    }};

    console.log('[PromoH5Interceptor] 已就绪, label=' + LABEL);
}})();
"#
    )
}
