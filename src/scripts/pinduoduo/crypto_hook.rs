pub fn create_pdd_crypto_hook() -> String {
    r#"
(function() {
    if (window.__pdd_crypto_hook_installed) return;
    window.__pdd_crypto_hook_installed = true;

    var _counter = 0;

    /* ── Embedded MD5 (RFC 1321, ES5) ──────────────────────────────────── */
    var _md5 = (function(){
        function add(a,b){var l=(a&0xFFFF)+(b&0xFFFF),m=(a>>16)+(b>>16)+(l>>16);return(m<<16)|(l&0xFFFF);}
        function rol(n,c){return(n<<c)|(n>>>(32-c));}
        function cmn(q,a,b,x,s,t){return add(rol(add(add(a,q),add(x,t)),s),b);}
        function F(a,b,c,d,x,s,t){return cmn((b&c)|(~b&d),a,b,x,s,t);}
        function G(a,b,c,d,x,s,t){return cmn((b&d)|(c&~d),a,b,x,s,t);}
        function H(a,b,c,d,x,s,t){return cmn(b^c^d,a,b,x,s,t);}
        function I(a,b,c,d,x,s,t){return cmn(c^(b|~d),a,b,x,s,t);}
        function blks(s){
            var n=((s.length+8)>>6)+1,b=new Array(n*16);
            for(var i=0;i<b.length;i++)b[i]=0;
            for(var i=0;i<s.length;i++)b[i>>2]|=s.charCodeAt(i)<<(i%4*8);
            b[s.length>>2]|=0x80<<(s.length%4*8);
            b[n*16-2]=s.length*8;
            return b;
        }
        function hex(w){var s='';for(var i=0;i<4;i++)s+=('0'+((w>>i*8)&0xFF).toString(16)).slice(-2);return s;}
        return function md5(str){
            str=unescape(encodeURIComponent(str));
            var x=blks(str),a=1732584193,b=-271733879,c=-1732584194,d=271733878;
            for(var i=0;i<x.length;i+=16){
                var A=a,B=b,C=c,D=d;
                a=F(a,b,c,d,x[i+0],7,-680876936);   d=F(d,a,b,c,x[i+1],12,-389564586);
                c=F(c,d,a,b,x[i+2],17,606105819);   b=F(b,c,d,a,x[i+3],22,-1044525330);
                a=F(a,b,c,d,x[i+4],7,-176418897);   d=F(d,a,b,c,x[i+5],12,1200080426);
                c=F(c,d,a,b,x[i+6],17,-1473231341); b=F(b,c,d,a,x[i+7],22,-45705983);
                a=F(a,b,c,d,x[i+8],7,1770035416);   d=F(d,a,b,c,x[i+9],12,-1958414417);
                c=F(c,d,a,b,x[i+10],17,-42063);      b=F(b,c,d,a,x[i+11],22,-1990404162);
                a=F(a,b,c,d,x[i+12],7,1804603682);  d=F(d,a,b,c,x[i+13],12,-40341101);
                c=F(c,d,a,b,x[i+14],17,-1502002290);b=F(b,c,d,a,x[i+15],22,1236535329);
                a=G(a,b,c,d,x[i+1],5,-165796510);   d=G(d,a,b,c,x[i+6],9,-1069501632);
                c=G(c,d,a,b,x[i+11],14,643717713);  b=G(b,c,d,a,x[i+0],20,-373897302);
                a=G(a,b,c,d,x[i+5],5,-701558691);   d=G(d,a,b,c,x[i+10],9,38016083);
                c=G(c,d,a,b,x[i+15],14,-660478335); b=G(b,c,d,a,x[i+4],20,-405537848);
                a=G(a,b,c,d,x[i+9],5,568446438);    d=G(d,a,b,c,x[i+14],9,-1019803690);
                c=G(c,d,a,b,x[i+3],14,-187363961);  b=G(b,c,d,a,x[i+8],20,1163531501);
                a=G(a,b,c,d,x[i+13],5,-1444681467); d=G(d,a,b,c,x[i+2],9,-51403784);
                c=G(c,d,a,b,x[i+7],14,1735328473);  b=G(b,c,d,a,x[i+12],20,-1926607734);
                a=H(a,b,c,d,x[i+5],4,-378558);      d=H(d,a,b,c,x[i+8],11,-2022574463);
                c=H(c,d,a,b,x[i+11],16,1839030562); b=H(b,c,d,a,x[i+14],23,-35309556);
                a=H(a,b,c,d,x[i+1],4,-1530992060);  d=H(d,a,b,c,x[i+4],11,1272893353);
                c=H(c,d,a,b,x[i+7],16,-155497632);  b=H(b,c,d,a,x[i+10],23,-1094730640);
                a=H(a,b,c,d,x[i+13],4,681279174);   d=H(d,a,b,c,x[i+0],11,-358537222);
                c=H(c,d,a,b,x[i+3],16,-722521979);  b=H(b,c,d,a,x[i+6],23,76029189);
                a=H(a,b,c,d,x[i+9],4,-640364487);   d=H(d,a,b,c,x[i+12],11,-421815835);
                c=H(c,d,a,b,x[i+15],16,530742520);  b=H(b,c,d,a,x[i+2],23,-995338651);
                a=I(a,b,c,d,x[i+0],6,-198630844);   d=I(d,a,b,c,x[i+7],10,1126891415);
                c=I(c,d,a,b,x[i+14],15,-1416354905);b=I(b,c,d,a,x[i+5],21,-57434055);
                a=I(a,b,c,d,x[i+12],6,1700485571);  d=I(d,a,b,c,x[i+3],10,-1894986606);
                c=I(c,d,a,b,x[i+10],15,-1051523);   b=I(b,c,d,a,x[i+1],21,-2054922799);
                a=I(a,b,c,d,x[i+8],6,1873313359);   d=I(d,a,b,c,x[i+15],10,-30611744);
                c=I(c,d,a,b,x[i+6],15,-1560198380); b=I(b,c,d,a,x[i+13],21,1309151649);
                a=I(a,b,c,d,x[i+4],6,-145523070);   d=I(d,a,b,c,x[i+11],10,-1120210379);
                c=I(c,d,a,b,x[i+2],15,718787259);   b=I(b,c,d,a,x[i+9],21,-343485551);
                a=add(a,A);b=add(b,B);c=add(c,C);d=add(d,D);
            }
            return hex(a)+hex(b)+hex(c)+hex(d);
        };
    })();
    /* ─────────────────────────────────────────────────────────────────────── */

    // Optional extras enriched from webpack modules (best-effort)
    var _extra = { rndStr: null, anti: null, hash: null };

    function findWebpackReq() {
        // webpack 4: webpackJsonp
        if (window.webpackJsonp && typeof window.webpackJsonp.push === 'function') {
            try {
                var r = null, id = '_pc_' + Date.now();
                window.webpackJsonp.push([[id], {[id]: function(m,e,fn){ r=fn; }}, [id]]);
                if (r) return r;
            } catch(e) {}
        }
        // webpack 5: any webpackChunk* array
        try {
            var wkeys = Object.keys(window);
            for (var wi = 0; wi < wkeys.length; wi++) {
                var wk = wkeys[wi];
                if (/^webpackChunk/.test(wk) && Array.isArray(window[wk])) {
                    try {
                        var r = null, id = '_pc_' + Date.now();
                        window[wk].push([[id], {[id]: function(m,e,fn){ r=fn; }}, [id]]);
                        if (r) return r;
                    } catch(e) {}
                }
            }
        } catch(e) {}
        return window.__webpack_require__ || null;
    }

    function tryEnrich() {
        // Already have everything we care about
        if (_extra.rndStr && _extra.anti) return true;

        // Check window globals first (cheapest path)
        if (!_extra.anti && typeof window.getRiskCtrCrawlerInfo === 'function') {
            _extra.anti = function(){ return window.getRiskCtrCrawlerInfo(); };
        }

        var req = findWebpackReq();
        if (!req) return !!_extra.rndStr && !!_extra.anti;

        // Probe a small set of module IDs (originals + ±2 neighbors)
        var ids = [50,49,51,52,53, 275,273,274,276,277, 622,620,621,623,624,
                   100,150,200,250,300,400,500,600,700,800,900,1000];
        for (var i = 0; i < ids.length; i++) {
            if (_extra.rndStr && _extra.anti && _extra.hash) break;
            try {
                var m = req(ids[i]);
                var obj = m && (m.default || m);
                if (!obj) continue;
                if (!_extra.rndStr && typeof obj.randomToString === 'function') {
                    (function(o){ _extra.rndStr = function(){ return o.randomToString(); }; })(obj);
                }
                if (!_extra.anti && typeof obj.getRiskCtrCrawlerInfo === 'function') {
                    (function(o){ _extra.anti = function(){ return o.getRiskCtrCrawlerInfo(); }; })(obj);
                }
                // module 622 exports a plain function(uid, mallId, requestId)
                if (!_extra.hash && typeof m === 'function' && m.length === 3 && !m.default) {
                    (function(fn){ _extra.hash = fn; })(m);
                }
            } catch(e) {}
        }
        return !!_extra.rndStr && !!_extra.anti;
    }

    // Install __pddComputeCrypto immediately — embedded MD5 guarantees it always works.
    // Extra functions (randomToString / getRiskCtrCrawlerInfo / hash) are filled in by
    // tryEnrich() and picked up on the next call because __pddComputeCrypto reads _extra
    // at call-time, not at install-time.
    window.__pddComputeCrypto = function(uid, content, callbackId) {
        try {
            var g    = window.global_mall_id || '';
            var gUid = window.global_uid || '';
            var salt = 'rpZigw#&iy$!KQD8';

            var part1  = _md5([g, gUid, uid, content, salt].join('@')).slice(0, 16);
            var rndStr = (_extra.rndStr && _extra.rndStr()) || Math.random().toString(36).slice(2);
            var part2  = _md5(rndStr).slice(0, 16);
            var random = part1 + part2;

            _counter = (_counter >= 1000000) ? 1 : _counter + 1;
            var k = Date.now() + _counter;

            var hash = '';
            try { if (_extra.hash) hash = _extra.hash(uid + '', g, k) || ''; } catch(e) {}

            var antiContent = '';
            var antiContentOuter = '';
            try {
                antiContent = (_extra.anti && _extra.anti())
                              || (window.getRiskCtrCrawlerInfo && window.getRiskCtrCrawlerInfo())
                              || '';
            } catch(e) {}
            try {
                antiContentOuter = (_extra.anti && _extra.anti())
                                   || (window.getRiskCtrCrawlerInfo && window.getRiskCtrCrawlerInfo())
                                   || '';
            } catch(e) {}

            window.__TAURI__.core.invoke('pdd_crypto_callback', {
                callbackId:       callbackId,
                antiContent:      antiContent,
                antiContentOuter: antiContentOuter,
                hash:             hash,
                random:           random,
                requestId:        k
            }).catch(function(e) {
                console.error('[pdd_crypto] invoke 失败:', e);
            });
        } catch(e) {
            console.error('[pdd_crypto] 计算失败:', e);
        }
    };

    console.log('[pdd_crypto] hook 已就绪（内置 MD5）');

    // Best-effort: enrich _extra with webpack functions
    tryEnrich();

    window.addEventListener('load', function() {
        if (tryEnrich()) return;
        var timer = setInterval(function() {
            if (tryEnrich()) clearInterval(timer);
        }, 500);
    });
})();
    "#.to_string()
}
