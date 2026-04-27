1. Al crear la imagen no tenia openssl el debian o no lo encuentra:
    - Se añadio: RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
    esto para que rust pueda detectar el openSSL