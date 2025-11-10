#[cfg(test)]
mod tooling_tests {
    use std::fs;
    use std::path::Path;

    /// Test que verifica que rustfmt.toml existe
    #[test]
    fn rustfmt_config_exists() {
        assert!(Path::new("rustfmt.toml").exists(), "rustfmt.toml no existe");

        let content = fs::read_to_string("rustfmt.toml").expect("No se pudo leer rustfmt.toml");

        // Verificar configuraciones importantes
        assert!(
            content.contains("edition = \"2024\""),
            "rustfmt.toml debe usar edition 2024"
        );
    }

    /// Test que verifica que .clippy.toml existe
    #[test]
    fn clippy_config_exists() {
        assert!(Path::new(".clippy.toml").exists(), ".clippy.toml no existe");
    }

    /// Test que verifica que rust-toolchain.toml existe
    #[test]
    fn rust_toolchain_is_pinned() {
        assert!(
            Path::new("rust-toolchain.toml").exists(),
            "rust-toolchain.toml no existe"
        );

        let content =
            fs::read_to_string("rust-toolchain.toml").expect("No se pudo leer rust-toolchain.toml");

        assert!(
            content.contains("channel ="),
            "rust-toolchain.toml debe especificar el channel"
        );
        assert!(
            content.contains("rustfmt"),
            "rust-toolchain.toml debe incluir rustfmt"
        );
        assert!(
            content.contains("clippy"),
            "rust-toolchain.toml debe incluir clippy"
        );
    }
}
