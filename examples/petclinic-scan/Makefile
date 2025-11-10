# Makefile para hodei-scan - PetClinic Example
# Uso: make <target>

.PHONY: help scan scan-security scan-quality scan-testing build-docker run-docker test clean

# Configuración
PROJECT_DIR ?= /tmp/spring-petclinic
DOCKER_IMAGE ?= hodei-scan:latest
RULES_DIR ?= rules
CONFIG_FILE ?= config/quality-gates.yml
OUTPUT_DIR ?= reports

# Colores
BLUE=\033[0;34m
GREEN=\033[0;32m
YELLOW=\033[1;33m
RED=\033[0;31m
NC=\033[0m

help: ## Mostrar ayuda
	@echo "$(BLUE)Hodei Scan - PetClinic Example$(NC)"
	@echo ""
	@echo "$(GREEN)Targets disponibles:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(GREEN)Ejemplos:$(NC)"
	@echo "  make scan                    # Escaneo completo"
	@echo "  make scan-security           # Solo seguridad"
	@echo "  make build-docker            # Construir imagen Docker"
	@echo "  make run-docker              # Ejecutar con Docker"

scan: ## Escaneo completo con quality gates
	@echo "$(BLUE)🔍 Ejecutando escaneo completo...$(NC)"
	@./scan-petclinic.sh $(PROJECT_DIR)
	@echo "$(GREEN)✅ Escaneo completado. Ver reportes en $(OUTPUT_DIR)/$(NC)"

scan-security: ## Solo escaneo de seguridad
	@echo "$(BLUE)🔒 Ejecutando escaneo de seguridad...$(NC)"
	@mkdir -p $(OUTPUT_DIR)
	@hodei-scan scan \
		--rules $(RULES_DIR)/security.rules \
		--output $(OUTPUT_DIR)/security.json \
		--format json \
		$(PROJECT_DIR)/src/
	@echo "$(GREEN)✅ Escaneo de seguridad completado$(NC)"

scan-quality: ## Solo escaneo de calidad
	@echo "$(BLUE)📊 Ejecutando escaneo de calidad...$(NC)"
	@mkdir -p $(OUTPUT_DIR)
	@hodei-scan scan \
		--rules $(RULES_DIR)/quality.rules \
		--output $(OUTPUT_DIR)/quality.json \
		--format json \
		$(PROJECT_DIR)/src/
	@echo "$(GREEN)✅ Escaneo de calidad completado$(NC)"

scan-testing: ## Solo escaneo de testing
	@echo "$(BLUE)🧪 Ejecutando escaneo de testing...$(NC)"
	@mkdir -p $(OUTPUT_DIR)
	@hodei-scan scan \
		--rules $(RULES_DIR)/testing.rules \
		--output $(OUTPUT_DIR)/testing.json \
		--format json \
		$(PROJECT_DIR)/src/
	@echo "$(GREEN)✅ Escaneo de testing completado$(NC)"

build-docker: ## Construir imagen Docker
	@echo "$(BLUE)🐳 Construyendo imagen Docker...$(NC)"
	@docker build -t $(DOCKER_IMAGE) .
	@echo "$(GREEN)✅ Imagen construida: $(DOCKER_IMAGE)$(NC)"

run-docker: ## Ejecutar escaneo con Docker
	@echo "$(BLUE)🐳 Ejecutando escaneo con Docker...$(NC)"
	@docker run --rm \
		-v $(PROJECT_DIR):/app/src \
		-v $(PWD)/$(OUTPUT_DIR):/app/reports \
		$(DOCKER_IMAGE) \
		scan
	@echo "$(GREEN)✅ Escaneo con Docker completado$(NC)"

run-docker-security: ## Ejecutar escaneo de seguridad con Docker
	@echo "$(BLUE)🐳 Ejecutando escaneo de seguridad con Docker...$(NC)"
	@docker run --rm \
		-v $(PROJECT_DIR):/app/src \
		-v $(PWD)/$(OUTPUT_DIR):/app/reports \
		$(DOCKER_IMAGE) \
		scan-security
	@echo "$(GREEN)✅ Escaneo de seguridad con Docker completado$(NC)"

test: ## Ejecutar tests del proyecto Java
	@echo "$(BLUE)🧪 Ejecutando tests de PetClinic...$(NC)"
	@cd $(PROJECT_DIR) && \
		./mvnw clean test && \
		./mvnw jacoco:report
	@echo "$(GREEN)✅ Tests completados$(NC)"
	@echo "$(YELLOW)📊 Ver reporte de cobertura:$(NC)"
	@echo "  $(PROJECT_DIR)/target/site/jacoco/index.html"

validate-gates: ## Validar quality gates sin escanear
	@echo "$(BLUE)🚦 Validando configuración de quality gates...$(NC)"
	@if [ -f $(CONFIG_FILE) ]; then \
		echo "✅ Config file existe: $(CONFIG_FILE)"; \
		jq . $(CONFIG_FILE) > /dev/null && echo "✅ Config YAML válido"; \
	else \
		echo "❌ Config file no encontrado: $(CONFIG_FILE)"; \
		exit 1; \
	fi
	@if [ -d $(RULES_DIR) ]; then \
		echo "✅ Rules directory existe: $(RULES_DIR)"; \
		echo "📋 Rules encontradas:"; \
		ls -1 $(RULES_DIR)/*.rules; \
	else \
		echo "❌ Rules directory no encontrado: $(RULES_DIR)"; \
		exit 1; \
	fi
	@echo "$(GREEN)✅ Configuración validada$(NC)"

docker-shell: ## Abrir shell en contenedor Docker
	@echo "$(BLUE)🐳 Abriendo shell en contenedor...$(NC)"
	@docker run --rm -it \
		-v $(PROJECT_DIR):/app/src \
		-v $(PWD)/$(OUTPUT_DIR):/app/reports \
		$(DOCKER_IMAGE) \
		/bin/bash

setup: ## Configurar entorno de desarrollo
	@echo "$(BLUE)⚙️ Configurando entorno...$(NC)"
	@echo "📦 Clonando Spring PetClinic..."
	@[ -d $(PROJECT_DIR) ] || git clone https://github.com/spring-projects/spring-petclinic.git $(PROJECT_DIR)
	@chmod +x scan-petclinic.sh
	@echo "$(GREEN)✅ Entorno configurado$(NC)"

clean: ## Limpiar reportes generados
	@echo "$(BLUE)🧹 Limpiando...$(NC)"
	@rm -rf $(OUTPUT_DIR)
	@docker rmi $(DOCKER_IMAGE) 2>/dev/null || true
	@echo "$(GREEN)✅ Limpieza completada$(NC)"

show-report: ## Mostrar reporte en navegador
	@echo "$(BLUE)🌐 Abriendo reporte...$(NC)"
	@if [ -f $(OUTPUT_DIR)/full-scan.html ]; then \
		@if command -v xdg-open >/dev/null 2>&1; then \
			xdg-open $(OUTPUT_DIR)/full-scan.html; \
		elif command -v open >/dev/null 2>&1; then \
			open $(OUTPUT_DIR)/full-scan.html; \
		else \
			echo "No se puede abrir el navegador automáticamente"; \
			echo "Ver manualmente: $(OUTPUT_DIR)/full-scan.html"; \
		fi \
	else \
		echo "❌ Reporte no encontrado: $(OUTPUT_DIR)/full-scan.html"; \
		echo "Ejecuta 'make scan' primero"; \
		exit 1; \
	fi

github-setup: ## Configurar GitHub Actions en tu repositorio
	@echo "$(BLUE)🐙 Configurando GitHub Actions...$(NC)"
	@mkdir -p .github/workflows
	@cp .github/workflows/hodei-scan.yml .github/workflows/ 2>/dev/null || true
	@echo "$(GREEN)✅ GitHub Actions configurado$(NC)"
	@echo "$(YELLOW)📝 Recuerda hacer commit y push del archivo .github/workflows/hodei-scan.yml$(NC)"

ci-test: ## Simular pipeline de CI localmente
	@echo "$(BLUE)🔄 Simulando pipeline de CI...$(NC)"
	@echo "  1️⃣ Setup..."
	@./mvnw --version 2>/dev/null || echo "  Maven no disponible (simulado)"
	@echo "  2️⃣ Build..."
	@echo "  3️⃣ Test..."
	@echo "  4️⃣ Hodei Scan..."
	@make scan
	@echo "  5️⃣ Quality Gate Validation..."
	@if [ -f $(OUTPUT_DIR)/full-scan.json ]; then \
		COVERAGE=$$(jq -r '.summary.coverage.percentage // 0' $(OUTPUT_DIR)/full-scan.json 2>/dev/null || echo "0"); \
		echo "  Cobertura: $$COVERAGE%"; \
		if (( $$(echo "$$COVERAGE < 80" | bc -l) )); then \
			echo "  ❌ Cobertura insuficiente"; \
			exit 1; \
		fi; \
		echo "  ✅ Quality gates pasados"; \
	else \
		echo "  ❌ No se encontró reporte"; \
		exit 1; \
	fi
	@echo "$(GREEN)✅ Pipeline de CI simulado exitosamente$(NC)"

.PHONY: help
