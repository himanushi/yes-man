fn main() {
    // ESP32 (xtensa) ターゲットの場合のみ embuild を使用
    // ホストでのテスト時は何もしない
    #[cfg(feature = "embuild")]
    {
        let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
        if target_arch == "xtensa" {
            embuild::espidf::sysenv::output();
        }
    }
}
