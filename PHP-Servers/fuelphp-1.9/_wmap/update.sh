#!/bin/sh
composer update

# have the route & controller
yes|cp -r _wmap/fuel/* ./

# some enhancements
composer config allow-plugins.composer/installers true
composer install --no-dev -o
rm ./public/.htaccess