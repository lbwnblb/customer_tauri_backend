pub fn pdd_redirect() -> String {
    r#"
        (function() {
            function checkAndNotify() {
                console.log('[pdd_redirect] checkAndNotify called, pathname=' + location.pathname);
                if (location.pathname.includes('/home') || location.pathname.includes('/login')  || location.pathname.includes('/chat-merchant')) {
                    console.log('[pdd_redirect] ' + location.pathname + ' detected, invoking shop_name_callback');
                    window.__TAURI__.core.invoke('shop_name_callback')
                        .then(() => console.log('[pdd_redirect] shop_name_callback invoke success'))
                        .catch(e => console.error('[pdd_redirect] shop_name_callback invoke failed:', e));
                }


            }

            const _pushState = history.pushState.bind(history);
            history.pushState = function(...args) {
                console.log('[pdd_redirect] pushState called, url=' + args[2]);
                _pushState(...args);
                checkAndNotify();
            };
            const _replaceState = history.replaceState.bind(history);
            history.replaceState = function(...args) {
                console.log('[pdd_redirect] replaceState called, url=' + args[2]);
                _replaceState(...args);
                checkAndNotify();
            };
            window.addEventListener('popstate', () => {
                console.log('[pdd_redirect] popstate fired');
                checkAndNotify();
            });

            window.addEventListener('load', function() {
                console.log('[pdd_redirect] load event, pathname=' + location.pathname);
                checkAndNotify();
            });

            console.log('[pdd_redirect] script init, pathname=' + location.pathname);
            checkAndNotify();
        })();
"#.to_string()
}
