pub fn create_intercepted_webview() -> String {
    r#"
(function() {
    if (window.__feige_intercept_installed) return;
    window.__feige_intercept_installed = true;

    const safeInvoke = (payload) => {
        try {
            if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
                window.__TAURI__.core.invoke('on_request', { payload })
                    .catch(() => {}); // 吞掉 rejection,避免污染页面
            }
        } catch (_) {}
    };

    const log = (type, method, url, body) => {
        try {
            const urlObj = new URL(url, location.href);
            const params = Object.fromEntries(urlObj.searchParams.entries());
            const info = {
                type, method,
                url: urlObj.origin + urlObj.pathname,
                query: params,
                body: null
            };

            if (body) {
                if (typeof body === 'string') {
                    try { info.body = JSON.parse(body); } catch { info.body = body; }
                } else if (body instanceof URLSearchParams) {
                    info.body = Object.fromEntries(body.entries());
                } else if (typeof FormData !== 'undefined' && body instanceof FormData) {
                    const obj = {};
                    body.forEach((v, k) => { obj[k] = v; });
                    info.body = obj;
                }
                // ReadableStream / Blob 等就跳过,别去读它,否则会消费掉 body
            }

            console.log('[拦截 ' + type + ']', method, urlObj.pathname, info);
            safeInvoke(JSON.stringify(info));
        } catch (e) {
            // 任何异常都不能往外抛
        }
    };

    // ---- fetch ----
    try {
        const originalFetch = window.fetch;
        if (typeof originalFetch === 'function') {
            window.fetch = function(input, init) {
                try {
                    const url = input instanceof Request ? input.url : String(input);
                    const method = (init && init.method) ||
                                   (input instanceof Request ? input.method : 'GET');
                    const body = (init && init.body) || null;
                    log('fetch', String(method).toUpperCase(), url, body);
                } catch (_) {}
                // 关键:用 window 作为 this,不要用调用方的 this
                return originalFetch.apply(window, arguments);
            };
        }
    } catch (_) {}

    // ---- XHR ----
    try {
        const proto = XMLHttpRequest.prototype;
        const originalOpen = proto.open;
        const originalSend = proto.send;
        proto.open = function(method, url) {
            try {
                Object.defineProperty(this, '__fg_method', { value: method, writable: true, configurable: true });
                Object.defineProperty(this, '__fg_url', { value: url, writable: true, configurable: true });
            } catch (_) {}
            return originalOpen.apply(this, arguments);
        };
        proto.send = function(body) {
            try {
                log('xhr', String(this.__fg_method || 'GET').toUpperCase(), this.__fg_url || '', body);
            } catch (_) {}
            return originalSend.apply(this, arguments);
        };
    } catch (_) {}

    // ---- sendBeacon ----
    try {
        if (navigator.sendBeacon) {
            const originalBeacon = navigator.sendBeacon.bind(navigator);
            navigator.sendBeacon = function(url, data) {
                try { log('beacon', 'POST', url, data); } catch (_) {}
                return originalBeacon(url, data);
            };
        }
    } catch (_) {}

    console.log('[请求拦截] 已启动');
})();
    "#.to_string()
}