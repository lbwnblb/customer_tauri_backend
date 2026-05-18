pub fn auto_zoom() -> String {
    r#"
    window.addEventListener('load', function() {
        let contentWidth = document.documentElement.scrollWidth;
        let viewportWidth = window.innerWidth;
        if (contentWidth > viewportWidth) {
            document.body.style.zoom = (viewportWidth / contentWidth).toString();
        }
    });
"#.to_string()
}
