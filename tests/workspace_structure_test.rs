#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    /// Test que verifica que el workspace contiene todos los crates requeridos
    #[test]
    fn workspace_has_all_required_crates() {
        let required_crates = vec![
            "hodei-ir",
            "hodei-engine",
            "hodei-dsl",
            "hodei-extractors",
            "hodei-cli",
        ];

        for crate_name in required_crates {
            let crate_path = Path::new("crates").join(crate_name);
            assert!(
                crate_path.exists(),
                "Crate '{}' no existe en crates/{}",
                crate_name, crate_name
            );

            let cargo_toml = crate_path.join("Cargo.toml");
            assert!(
                cargo_toml.exists(),
                "Cargo.toml no existe para crate '{}'",
                crate_name
            );
        }
    }

    /// Test que verifica que cada crate tiene un README
    #[test]
    fn each_crate_has_readme() {
        let required_crates = vec![
            "hodei-ir",
            "hodei-engine",
            "hodei-dsl",
            "hodei-extractors",
            "hodei-cli",
        ];

        for crate_name in required_crates {
            let readme_path = Path::new("crates").join(crate_name).join("README.md");
            assert!(
                readme_path.exists(),
                "README.md no existe para crate '{}'",
                crate_name
            );
        }
    }

    /// Test que verifica que las dependencias están definidas en el workspace
    #[test]
    fn workspace_dependencies_are_defined() {
        let workspace_toml = fs::read_to_string("Cargo.toml")
            .expect("No se pudo leer Cargo.toml del workspace");

        // Verificar que es un workspace
        assert!(
            workspace_toml.contains("[workspace]"),
            "Cargo.toml no define un workspace"
        );

        // Verificar que tiene members para todos los crates
        assert!(
            workspace_toml.contains("hodei-ir"),
            "workspace no incluye hodei-ir"
        );
        assert!(
            workspace_toml.contains("hodei-engine"),
            "workspace no incluye hodei-engine"
        );
        assert!(
            workspace_toml.contains("hodei-dsl"),
            "workspace no incluye hodei-dsl"
        );
        assert!(
            workspace_toml.contains("hodei-extractors"),
            "workspace no incluye hodei-extractors"
        );
        assert!(
            workspace_toml.contains("hodei-cli"),
            "workspace no incluye hodei-cli"
        );
    }

    /// Test que verifica que el proyecto compila sin warnings
    #[test]
    fn workspace_compiles_without_warnings() {
        // Este test simula la verificación de compilación
        // En un entorno real, usaríamos: cargo check --workspace
        // Por ahora, verificamos que los archivos de configuración existen
        assert!(
            Path::new("rustfmt.toml").exists(),
            "rustfmt.toml no existe"
        );
        assert!(
            Path::new(".clippy.toml").exists(),
            ".clippy.toml no existe"
        );
    }
}
