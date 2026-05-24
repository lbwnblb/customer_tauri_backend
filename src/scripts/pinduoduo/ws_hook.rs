pub fn create_pdd_ws_hook() -> String {
    r#"
(function () {
  'use strict';

  const OriginalWebSocket = window.WebSocket;
  const { invoke } = window.__TAURI__.core;

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
    if (!new URL(url).hostname.endsWith('titan-ws.pinduoduo.com')) return ws;

    const params = new URL(url).searchParams;
    const meta = {
      url,
      token: params.get('token') || '',
      aid: params.get('aid') || '',
      deviceId: params.get('device_id') || '',
    };

    window.__WS_INSTANCE__ = ws;

    emit('connect', meta);

    ws.binaryType = 'arraybuffer';

    ws.addEventListener('message', async (e) => {
      if (typeof e.data === 'string') return;
      const buf = await toArrayBuffer(e.data);
      invoke('pdd_ws_recv', { data: Array.from(new Uint8Array(buf)) });
    });

    const origSend = ws.send.bind(ws);
    ws.send = async function (data) {
      try {
        if (typeof data !== 'string') {
          const buf = await toArrayBuffer(data);
          invoke('pdd_ws_send', { data: Array.from(new Uint8Array(buf)) });
        }
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
