pub fn create_ws_hook() -> String {
    r#"
(function () {
  'use strict';

  const OriginalWebSocket = window.WebSocket;
  const { invoke } = window.__TAURI__.core;

  function emit(event, detail = {}) {
    invoke('on_ws', { event: { event, ...detail, timestamp: Date.now() } });
  }

  window.WebSocket = function (url, protocols) {
    const ws = new OriginalWebSocket(url, protocols);
    if (!new URL(url).hostname.endsWith('jinritemai.com')) return ws;

    const params = new URL(url).searchParams;
    const meta = {
      url,
      token: params.get('token') || '',
      aid: params.get('aid') || '',
      deviceId: params.get('device_id') || '',
    };

    emit('connect', meta);

    ws.addEventListener('message', async (e) => {
      const buf = e.data instanceof Blob ? await e.data.arrayBuffer() : e.data;
      invoke('on_ws_binary', buf instanceof ArrayBuffer ? buf : new Uint8Array(buf).buffer);
    });

    ws.addEventListener('open', () => emit('open', meta));
    ws.addEventListener('close', (e) => emit('close', { code: e.code, reason: e.reason, ...meta }));
    ws.addEventListener('error', () => emit('error', meta));

    return ws;
  };

  window.WebSocket.prototype = OriginalWebSocket.prototype;
  window.WebSocket.CONNECTING = 0;
  window.WebSocket.OPEN = 1;
  window.WebSocket.CLOSING = 2;
  window.WebSocket.CLOSED = 3;
})();
"#
        .to_string()
}