/// 生成 JS 拦截脚本，用于 initialization_script 注入
pub fn create_http_response_hook() -> String {
    r#"
(function () {
  'use strict';
  if (window.__HTTP_RESPONSE_INTERCEPTOR__) return;
  window.__HTTP_RESPONSE_INTERCEPTOR__ = true;

  const TARGETS = [
    '/pigeon_im/v1/message/get_message_by_index_v2_range',
    '/pigeon_im/v1/message/get_by_conversation',
  ];

  function isTarget(url) {
    return TARGETS.some(function (t) { return url.includes(t); });
  }

  function sendToRust(url, status, bytes) {
    window.__TAURI__.core.invoke('on_http_response_intercepted', {
      url: url,
      status: status,
      body: Array.from(bytes),
    }).catch(function (e) {
      console.error('[HttpInterceptor] invoke 失败:', e);
    });
  }

  // ========== 拦截 fetch ==========
  const _fetch = window.fetch;
  window.fetch = async function (...args) {
    const url = typeof args[0] === 'string'
      ? args[0]
      : args[0] instanceof Request ? args[0].url : String(args[0]);

    if (!isTarget(url)) return _fetch.apply(this, args);

    console.log('[HttpInterceptor] fetch 请求:', url);
    const resp = await _fetch.apply(this, args);
    const clone = resp.clone();
    const buf = await clone.arrayBuffer();
    const bytes = new Uint8Array(buf);

    console.log('[HttpInterceptor] fetch 响应:', resp.status, bytes.length, 'bytes');
    sendToRust(url, resp.status, bytes);

    return resp;
  };

  // ========== 拦截 XMLHttpRequest ==========
  const _open = XMLHttpRequest.prototype.open;
  const _send = XMLHttpRequest.prototype.send;

  XMLHttpRequest.prototype.open = function (method, url, ...rest) {
    this._intUrl = url;
    this._intMethod = method;
    return _open.call(this, method, url, ...rest);
  };

  XMLHttpRequest.prototype.send = function (...args) {
    if (typeof this._intUrl === 'string' && isTarget(this._intUrl)) {
      console.log('[HttpInterceptor] XHR 请求:', this._intMethod, this._intUrl);

      const origCb = this.onreadystatechange;
      this.onreadystatechange = function (...a) {
        if (this.readyState === 4) {
          try {
            var bytes;
            var rt = this.responseType;

            if (rt === 'arraybuffer') {
              bytes = new Uint8Array(this.response);
            } else if (rt === 'blob') {
              var blob = this.response;
              var xhrUrl = this._intUrl;
              var xhrStatus = this.status;
              blob.arrayBuffer().then(function (buf) {
                sendToRust(xhrUrl, xhrStatus, new Uint8Array(buf));
              });
              if (typeof origCb === 'function') origCb.apply(this, a);
              return;
            } else {
              // text / json / 空 → 转成 bytes
              var text = (rt === 'json')
                ? JSON.stringify(this.response)
                : this.responseText;
              bytes = new TextEncoder().encode(text);
            }

            console.log('[HttpInterceptor] XHR 响应:', this.status, bytes.length, 'bytes');
            sendToRust(this._intUrl, this.status, bytes);
          } catch (e) {
            console.warn('[HttpInterceptor] 读取响应失败:', e);
          }
        }
        if (typeof origCb === 'function') origCb.apply(this, a);
      };
    }
    return _send.apply(this, args);
  };

  console.log('[HttpInterceptor] 响应拦截已就绪, 目标:', TARGETS);
})();
"#
        .to_string()
}