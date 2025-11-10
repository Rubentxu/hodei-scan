#[cfg(test)]
mod adr_tests {
    use std::fs;
    use std::path::Path;

    /// Test que verifica que el directorio de ADRs existe
    #[test]
    fn adr_directory_exists() {
        assert!(
            Path::new("docs/adr").exists(),
            "Directorio 'docs/adr' no existe"
        );
    }

    /// Test que verifica que los ADRs requeridos existen
    #[test]
    fn required_adrs_exist() {
        let required_adrs = vec![
            "docs/adr/ADR-001-rust-language.md",
            "docs/adr/ADR-002-hexagonal-architecture.md",
        ];

        for adr_path in required_adrs {
            assert!(Path::new(adr_path).exists(), "ADR '{}' no existe", adr_path);
        }
    }

    /// Test que verifica que el INDEX.md lista todas las decisiones
    #[test]
    fn adr_index_lists_all_decisions() {
        let index_path = "docs/adr/INDEX.md";
        assert!(
            Path::new(index_path).exists(),
            "INDEX.md no existe en docs/adr/"
        );

        let content = fs::read_to_string(index_path).expect("No se pudo leer INDEX.md");

        // Verificar que lista las ADRs
        assert!(content.contains("ADR-001"), "INDEX.md debe listar ADR-001");
        assert!(content.contains("ADR-002"), "INDEX.md debe listar ADR-002");
    }
}
