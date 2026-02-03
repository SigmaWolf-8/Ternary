# Kong Gateway Data Plane for PlenumNET
# Deploy to Render, Railway, or any Docker host

FROM kong/kong-gateway:3.6

# Set Kong to data plane mode
ENV KONG_ROLE=data_plane
ENV KONG_DATABASE=off
ENV KONG_VITALS=off
ENV KONG_CLUSTER_MTLS=pki
ENV KONG_LUA_SSL_TRUSTED_CERTIFICATE=system
ENV KONG_KONNECT_MODE=on
ENV KONG_PROXY_LISTEN=0.0.0.0:8000, 0.0.0.0:8443 ssl
ENV KONG_STATUS_LISTEN=0.0.0.0:8100

# These will be set at runtime via environment variables:
# KONG_CLUSTER_CONTROL_PLANE - Control plane endpoint
# KONG_CLUSTER_SERVER_NAME - Control plane server name
# KONG_CLUSTER_TELEMETRY_ENDPOINT - Telemetry endpoint
# KONG_CLUSTER_TELEMETRY_SERVER_NAME - Telemetry server name
# KONG_CLUSTER_CERT - Path to TLS certificate
# KONG_CLUSTER_CERT_KEY - Path to TLS private key

# Create directory for certificates
RUN mkdir -p /etc/kong/certs

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
  CMD kong health || exit 1

# Expose proxy ports
EXPOSE 8000 8443 8100

# Default command
CMD ["kong", "docker-start"]
