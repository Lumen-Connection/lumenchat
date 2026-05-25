fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Lumen Chat");
        res.set("FileDescription", "Lumen Chat — a local OpenRouter client");
        res.set("CompanyName", "Lumen Connection");
        res.set("LegalCopyright", "🄯 2026 Lumen Connection");
        res.compile().expect("failed to compile Windows resources");
    }
}