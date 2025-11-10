# Dockerfile para hodei-scan
# Permite ejecutar escaneos en entornos aislados

FROM openjdk:17-jdk-slim

# Metadatos
LABEL maintainer="hodei-scan team"
LABEL description="Hodei Scan - Static Code Analysis for Java/Spring"
LABEL version="0.1.0"

# Variables de entorno
ENV HODEI_SCAN_VERSION=0.1.0
ENV JAVA_OPTS="-Xmx2g -Xms512m"

# Instalar dependencias
RUN apt-get update && apt-get install -y \
    curl \
    jq \
    bc \
    git \
    && rm -rf /var/lib/apt/lists/*

# Crear directorio de trabajo
WORKDIR /app

# Descargar e instalar hodei-scan (simulado)
RUN echo "Simulando descarga de hodei-scan..." && \
    mkdir -p /app/bin && \
    echo '#!/bin/bash' > /app/bin/hodei-scan && \
    echo 'echo "Simulated hodei-scan v0.1.0"' >> /app/bin/hodei-scan && \
    chmod +x /app/bin/hodei-scan

# Instalar dependencias de Maven
RUN curl -s https://get.sdkman.io | bash
ENV SDKMAN_DIR="$HOME/.sdkman"
RUN bash -c "source $SDKMAN_DIR/bin/sdkman-init.sh && sdk install maven 3.9.4"

# Configurar PATH
ENV PATH="/app/bin:$PATH"
ENV MAVEN_HOME="$HOME/.sdkman/candidates/maven/current"

# Copiar archivos de configuración
COPY config/quality-gates.yml /app/config/
COPY rules/ /app/rules/

# Crear directorios para reportes
RUN mkdir -p /app/reports

# Volumen para código a escanear
VOLUME ["/app/src"]

# Puerto para dashboard web (opcional)
EXPOSE 8080

# Configuración por defecto
CMD ["hodei-scan", "scan", "--help"]

# Script de entrada
COPY docker-entrypoint.sh /app/
RUN chmod +x /app/docker-entrypoint.sh
ENTRYPOINT ["/app/docker-entrypoint.sh"]

# Comandos de ejemplo
# docker build -t hodei-scan:latest .
# docker run --rm -v $(pwd):/app/src hodei-scan:latest scan --rules rules/security.rules src/
