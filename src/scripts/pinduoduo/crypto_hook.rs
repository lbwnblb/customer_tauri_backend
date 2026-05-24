pub fn create_pdd_crypto_hook() -> String {
    r#"
(function() {
    if (window.__pdd_crypto_hook_installed) return;
    window.__pdd_crypto_hook_installed = true;

    var _reqCounter = 0;

    function tryInstall() {
        if (!window.webpackJsonp || typeof window.webpackJsonp.push !== 'function') {
            return false;
        }
        try {
            var req = null;
            var captureId = '_pdd_cap_' + Date.now();
            // 注入假 chunk 以捕获 webpack 内部 require 函数
            window.webpackJsonp.push([[captureId], {
                [captureId]: function(m, e, r) { req = r; }
            }, [captureId]]);

            if (!req) return false;

            var mod50 = null, mod275 = null, mod622 = null;
            try { mod50  = req(50);  } catch(e) {}
            try { mod275 = req(275); } catch(e) {}
            try { mod622 = req(622); } catch(e) {}

            var baseUtil = mod50  && (mod50.default  || mod50);
            var md5Fn    = mod275 && (mod275.default || mod275);
            var hashFn   = mod622 && (mod622.default || mod622);

            if (!baseUtil || !md5Fn) {
                console.warn('[pdd_crypto] baseUtil 或 md5 模块未找到，mod50=', mod50, 'mod275=', mod275);
                return false;
            }

            window.__pddComputeCrypto = function(uid, content, callbackId) {
                try {
                    var g    = window.global_mall_id || '';
                    var gUid = window.global_uid     || '';

                    // random = md5(mall_id@global_uid@uid@content@salt)[0:16] + md5(randomToString())[0:16]
                    var salt  = 'rpZigw#&iy$!KQD8';
                    var part1 = md5Fn([g, gUid, uid, content, salt].join('@')).slice(0, 16);
                    var rndStr = (baseUtil.randomToString && baseUtil.randomToString())
                                 || Math.random().toString(36).slice(2);
                    var part2 = md5Fn(rndStr).slice(0, 16);
                    var random = part1 + part2;

                    // request_id: 与 bundle 中 v() 函数逻辑一致
                    _reqCounter = (_reqCounter >= 1000000) ? 1 : _reqCounter + 1;
                    var k = Date.now() + _reqCounter;

                    // hash via module 622（失败时降级为空字符串）
                    var hash = '';
                    try { if (hashFn) hash = hashFn(uid + '', g, k) || ''; } catch(e) {}

                    // anti_content（风控令牌）
                    var antiContent = '';
                    try {
                        antiContent = (baseUtil.getRiskCtrCrawlerInfo && baseUtil.getRiskCtrCrawlerInfo()) || '';
                    } catch(e) {}

                    window.__TAURI__.core.invoke('pdd_crypto_callback', {
                        callbackId:  callbackId,
                        antiContent: antiContent,
                        hash:        hash,
                        random:      random,
                        requestId:   k
                    }).catch(function(e) {
                        console.error('[pdd_crypto] invoke 失败:', e);
                    });
                } catch(e) {
                    console.error('[pdd_crypto] 计算失败:', e);
                }
            };

            console.log('[pdd_crypto] hook 安装成功，hashFn=' + !!hashFn);
            return true;
        } catch(e) {
            console.error('[pdd_crypto] 安装错误:', e);
            return false;
        }
    }

    window.addEventListener('load', function() {
        if (tryInstall()) return;
        var attempts = 0;
        var timer = setInterval(function() {
            if (tryInstall() || ++attempts >= 20) clearInterval(timer);
        }, 500);
    });
})();
    "#.to_string()
}
