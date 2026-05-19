pub fn create_intercepted_webview() -> String {
    r#"
(function() {
    if (window.__feige_intercept_installed) return;
    window.__feige_intercept_installed = true;

    // ★ 防递归锁 ★
    // invoke('on_request') 底层走 fetch (Tauri IPC),
    // 如果不加锁: fetch → 拦截 → invoke → fetch → 拦截 → … 无限递归卡死
    let _busy = false;

    const safeInvoke = (payload) => {
        try {
            if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
                window.__TAURI__.core.invoke('on_request', { payload })
                    .catch(() => {});
            }
        } catch (_) {}
    };

    const log = (type, method, url, body) => {
        try {
            let base = location.href;
            if (!base || base === 'about:blank') base = 'https://placeholder.invalid';
            const urlObj = new URL(url, base);
            const params = Object.fromEntries(urlObj.searchParams.entries());
            const info = {
                type, method,
                url: urlObj.origin + urlObj.pathname,
                query: params,
                body: null
            };

            if (body) {
                if (typeof body === 'string') {
                    try { info.body = JSON.parse(body); } catch(_) { info.body = body; }
                } else if (body instanceof URLSearchParams) {
                    info.body = Object.fromEntries(body.entries());
                } else if (typeof FormData !== 'undefined' && body instanceof FormData) {
                    const obj = {};
                    body.forEach((v, k) => { obj[k] = v; });
                    info.body = obj;
                }
            }

            safeInvoke(JSON.stringify(info));
        } catch (_) {}
    };

    // ---- fetch ----
    try {
        const _origFetch = window.fetch;
        if (typeof _origFetch === 'function') {
            window.fetch = new Proxy(_origFetch, {
                apply(target, thisArg, argsList) {
                    if (!_busy) {
                        _busy = true;
                        try {
                            const [input, init] = argsList;
                            const url = (input instanceof Request) ? input.url : String(input || '');
                            const method = (init && init.method) ||
                                           ((input instanceof Request) ? input.method : 'GET');
                            const body = (init && init.body) || null;
                            log('fetch', String(method).toUpperCase(), url, body);
                        } catch (_) {}
                        _busy = false;
                    }
                    return Reflect.apply(target, thisArg, argsList);
                }
            });
        }
    } catch (_) {}

    // ---- XHR ----
    try {
        const proto = XMLHttpRequest.prototype;
        const _origOpen = proto.open;
        const _origSend = proto.send;
        const xhrMap = new WeakMap();

        proto.open = new Proxy(_origOpen, {
            apply(target, thisArg, argsList) {
                try { xhrMap.set(thisArg, { m: argsList[0], u: argsList[1] }); } catch (_) {}
                return Reflect.apply(target, thisArg, argsList);
            }
        });

        proto.send = new Proxy(_origSend, {
            apply(target, thisArg, argsList) {
                if (!_busy) {
                    _busy = true;
                    try {
                        const meta = xhrMap.get(thisArg);
                        if (meta) {
                            log('xhr', String(meta.m || 'GET').toUpperCase(), meta.u || '', argsList[0]);
                        }
                    } catch (_) {}
                    _busy = false;
                }
                return Reflect.apply(target, thisArg, argsList);
            }
        });
    } catch (_) {}

    // ---- sendBeacon ----
    try {
        if (navigator.sendBeacon) {
            const _origBeacon = navigator.sendBeacon;
            navigator.sendBeacon = new Proxy(_origBeacon, {
                apply(target, thisArg, argsList) {
                    if (!_busy) {
                        _busy = true;
                        try { log('beacon', 'POST', argsList[0], argsList[1]); } catch (_) {}
                        _busy = false;
                    }
                    return Reflect.apply(target, thisArg, argsList);
                }
            });
        }
    } catch (_) {}

    console.log('[请求拦截] 已启动');
})();
    "#.to_string()
}