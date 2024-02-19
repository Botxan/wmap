FROM php:8.2-fpm

# Install system dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libicu-dev \
        libzip-dev \
        unzip \
        git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install PHP extensions
RUN docker-php-ext-install pdo_mysql intl zip

# Install Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

WORKDIR "/var/www/cakephp-app"