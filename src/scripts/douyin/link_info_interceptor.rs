pub fn create_link_info_hook() -> String {
    r#"
(function () {
  'use strict';
  if (window.__LINK_INFO_INTERCEPTOR__) return;
  window.__LINK_INFO_INTERCEPTOR__ = true;

  const TARGET = '/chat/api/backstage/conversation/get_link_info';

  function isTarget(url) {
    return url.includes(TARGET);
  }

  function sendToRust(url, status, text) {
    window.__TAURI__.core.invoke('on_get_link_info', {
      url: url,
      status: status,
      body: text,
    }).catch(function (e) {
      console.error('[LinkInfoInterceptor] invoke 失败:', e);
    });
  }

  // ========== 拦截 fetch ==========
  const _fetch = window.fetch;
  window.fetch = async function (...args) {
    const url = typeof args[0] === 'string'
      ? args[0]
      : args[0] instanceof Request ? args[0].url : String(args[0]);

    if (!isTarget(url)) return _fetch.apply(this, args);

    console.log('[LinkInfoInterceptor] fetch 请求:', url);
    const resp = await _fetch.apply(this, args);
    const clone = resp.clone();
    const text = await clone.text();

    console.log('[LinkInfoInterceptor] fetch 响应:', resp.status, text.length, 'chars');
    sendToRust(url, resp.status, text);

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
      console.log('[LinkInfoInterceptor] XHR 请求:', this._intMethod, this._intUrl);

      const origCb = this.onreadystatechange;
      this.onreadystatechange = function (...a) {
        if (this.readyState === 4) {
          try {
            var rt = this.responseType;
            var text = (rt === 'json')
              ? JSON.stringify(this.response)
              : this.responseText;

            console.log('[LinkInfoInterceptor] XHR 响应:', this.status, text.length, 'chars');
            sendToRust(this._intUrl, this.status, text);
          } catch (e) {
            console.warn('[LinkInfoInterceptor] 读取响应失败:', e);
          }
        }
        if (typeof origCb === 'function') origCb.apply(this, a);
      };
    }
    return _send.apply(this, args);
  };

  console.log('[LinkInfoInterceptor] get_link_info 拦截已就绪');
})();
"#
    .to_string()
}
