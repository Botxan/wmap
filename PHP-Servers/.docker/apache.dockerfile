FROM php:8.3-apache

RUN apt-get update \
    && apt-get install -y libicu-dev

RUN mkdir /var/www/html/wmap

# Need it in a lot of frameworks
RUN docker-php-ext-install intl

# Optional opcache (recommended)
#RUN docker-php-ext-install opcache

ENV PORT 80
ENTRYPOINT []
CMD sed -i "s/80/$PORT/g" /etc/apache2/sites-available/000-default.conf /etc/apache2/ports.conf && docker-php-entrypoint apache2-foreground
