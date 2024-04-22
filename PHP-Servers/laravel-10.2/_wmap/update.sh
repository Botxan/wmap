#!/bin/sh
composer update

# have the route & controller
yes|cp -rf _wmap/laravel/. ./

# some enhancements
composer install --optimize-autoloader --no-dev
chmod -R o+w storage

rm ./public/.htaccess