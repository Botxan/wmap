FROM php:8.3-apache

RUN apt-get update && apt-get install -y libicu-dev curl git unzip && rm -rf /var/lib/apt/lists/*

RUN curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer

RUN mkdir /var/www/html/wmap

# Need it in a lot of frameworks
RUN docker-php-ext-install intl

# Optional opcache (recommended)
# RUN docker-php-ext-install opcache

COPY . /app
WORKDIR /app

RUN chmod +x setup.sh base/option_target.sh

RUN ./setup.sh

ENV PORT 80
ENTRYPOINT []
CMD sed -i "s/80/$PORT/g" /etc/apache2/sites-available/000-default.conf /etc/apache2/ports.conf && docker-php-entrypoint apache2-foreground
