pub fn create_ws_hook() -> String {
    r#"
(function () {
  'use strict';

  const OriginalWebSocket = window.WebSocket;
  const { invoke } = window.__TAURI__.core;

  function bufferToBase64(buffer) {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) binary += String.fromCharCode(bytes[i]);
    return btoa(binary);
  }

  async function toTransferable(data) {
    if (typeof data === 'string') return { type: 'text', payload: data };
    if (data instanceof ArrayBuffer) return { type: 'binary', payload: bufferToBase64(data) };
    if (data instanceof Blob) return { type: 'binary', payload: bufferToBase64(await data.arrayBuffer()) };
    return { type: 'unknown', payload: String(data) };
  }

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
      const t = await toTransferable(e.data);
      emit('message', { direction: 'incoming', dataType: t.type, payload: t.payload });
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