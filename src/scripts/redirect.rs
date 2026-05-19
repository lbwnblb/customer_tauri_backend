pub fn douyin_redirect(webview_id: &str) -> String {
    r#"
    (function() {
    var __webviewId = '__WEBVIEW_ID__';
    var LOG_PREFIX = '[douyin_redirect][' + __webviewId + ']';
    var lastUrl = window.location.href;

    console.log(LOG_PREFIX, '脚本注入完成, 当前URL:', lastUrl);

    function handleBackendTask(task) {
        console.log(LOG_PREFIX, '收到任务:', JSON.stringify(task));
        switch (task.shop_task_type) {
            case 1:
                console.log(LOG_PREFIX, '收到类型1任务:', task.data_str);
                break;
            default:
                console.log(LOG_PREFIX, '收到未知类型任务:', task.shop_task_type, task.data_str);
                break;
        }
    }

    function tryRegisterChannel() {
        if (window.__shopChannelRegistered) return;
        if (!window.__TAURI__ || !window.__TAURI__.core) {
            // 静默重试,不打日志避免刷屏
            setTimeout(tryRegisterChannel, 200);
            return;
        }
        console.log(LOG_PREFIX, '开始注册 shop_channel');
        window.__shopChannelRegistered = true;
        try {
            var channel = new window.__TAURI__.core.Channel();
            channel.onmessage = function(data) {
                handleBackendTask(data);
            };
            window.__TAURI__.core.invoke('add_shop_channel', {
                channel: channel,
                webviewId: __webviewId
            }).then(function(r) {
                console.log(LOG_PREFIX, '注册成功:', JSON.stringify(r));
            }).catch(function(e) {
                console.error(LOG_PREFIX, '注册失败:', e);
                window.__shopChannelRegistered = false;
            });
        } catch(e) {
            console.error(LOG_PREFIX, '初始化异常:', e);
            window.__shopChannelRegistered = false;
        }
    }

    function checkAndRedirect() {
        var url = window.location.href;
        console.log(LOG_PREFIX, '检查URL:', url);

        if (url.includes('https://fxg.jinritemai.com/ffa/mshop/homepage/index')) {
            console.log(LOG_PREFIX, '匹配到 fxg 死页面, 跳转到 im 工作台');
            window.location.href = 'https://im.jinritemai.com/pc_seller_v2/main/workspace';
            return;
        }
        if (url.startsWith('https://im.jinritemai.com/')) {
            console.log(LOG_PREFIX, '匹配到 im 页面, 尝试注册 channel');
            tryRegisterChannel();
        }
    }

    function onUrlMaybeChanged() {
        var url = window.location.href;
        if (url === lastUrl) return;
        console.log(LOG_PREFIX, 'URL 变化:', lastUrl, '->', url);
        lastUrl = url;
        checkAndRedirect();
    }

    // 1. 首次注入时检查一次(处理完整页面加载)
    checkAndRedirect();

    // 2. 监听浏览器前进/后退
    window.addEventListener('popstate', onUrlMaybeChanged);

    // 3. 监听 hash 变化
    window.addEventListener('hashchange', onUrlMaybeChanged);

    // 4. 劫持 History API,监听 SPA 路由跳转
    ['pushState', 'replaceState'].forEach(function(method) {
        var original = history[method];
        history[method] = function() {
            var result = original.apply(this, arguments);
            onUrlMaybeChanged();
            return result;
        };
    });

    console.log(LOG_PREFIX, '监听已就绪');
})();
"#.replace("__WEBVIEW_ID__", webview_id)
}