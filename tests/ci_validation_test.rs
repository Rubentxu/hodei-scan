#[cfg(test)]
mod ci_tests {
    use std::fs;
    use std::path::Path;

    /// Test que verifica que el workflow de CI existe y es válido
    #[test]
    fn ci_yaml_exists_and_is_valid() {
        let ci_path = Path::new(".github/workflows/ci.yml");
        assert!(
            ci_path.exists(),
            "CI workflow file '.github/workflows/ci.yml' no existe"
        );

        // Leer el contenido del archivo
        let content = fs::read_to_string(ci_path)
            .expect("No se pudo leer el archivo CI");

        // Verificar que contiene elementos esenciales del workflow
        assert!(
            content.contains("name: CI"),
            "Workflow debe tener 'name: CI'"
        );

        assert!(
            content.contains("on:"),
            "Workflow debe tener触发器 'on:'"
        );

        assert!(
            content.contains("push:"),
            "Workflow debe tener 'push:' trigger"
        );

        assert!(
            content.contains("pull_request:"),
            "Workflow debe tener 'pull_request:' trigger"
        );

        assert!(
            content.contains("jobs:"),
            "Workflow debe tener 'jobs:'"
        );

        // Verificar jobs específicos
        assert!(
            content.contains("test:"),
            "Workflow debe tener job 'test'"
        );

        assert!(
            content.contains("clippy:"),
            "Workflow debe tener job 'clippy'"
        );

        assert!(
            content.contains("fmt:"),
            "Workflow debe tener job 'fmt'"
        );

        assert!(
            content.contains("security:"),
            "Workflow debe tener job 'security'"
        );

        assert!(
            content.contains("coverage:"),
            "Workflow debe tener job 'coverage'"
        );

        // Verificar que usa acciones oficiales
        assert!(
            content.contains("actions/checkout@v"),
            "Workflow debe usar actions/checkout"
        );

        assert!(
            content.contains("dtolnay/rust-toolchain"),
            "Workflow debe usar dtolnay/rust-toolchain"
        );

        println!("✅ CI workflow validation passed");
    }
}
