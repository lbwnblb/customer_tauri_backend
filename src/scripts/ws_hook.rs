pub fn create_ws_hook() -> String {
    r#"
(function () {
  'use strict';

  const OriginalWebSocket = window.WebSocket;
  const { invoke } = window.__TAURI__.core;

  // 保存拦截到的 ws 实例
  window.__WS_INSTANCE__ = null;

  function emit(event, detail = {}) {
    invoke('on_ws', { event: { event, ...detail, timestamp: Date.now() } });
  }

  async function toArrayBuffer(data) {
    if (data instanceof Blob) return await data.arrayBuffer();
    if (data instanceof ArrayBuffer) return data;
    if (ArrayBuffer.isView(data)) return data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength);
    return new TextEncoder().encode(data).buffer;
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

    // 保存实例
    window.__WS_INSTANCE__ = ws;

    emit('connect', meta);

    ws.addEventListener('message', async (e) => {
      const buf = await toArrayBuffer(e.data);
      invoke('on_ws_recv', buf);
    });

    const origSend = ws.send.bind(ws);
    ws.send = async function (data) {
      try {
        const buf = await toArrayBuffer(data);
        invoke('on_ws_send', buf);
      } catch (_) {}
      origSend(data);
    };

    ws.addEventListener('open', () => emit('open', meta));
    ws.addEventListener('close', (e) => {
      window.__WS_INSTANCE__ = null;
      emit('close', { code: e.code, reason: e.reason, ...meta });
    });
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